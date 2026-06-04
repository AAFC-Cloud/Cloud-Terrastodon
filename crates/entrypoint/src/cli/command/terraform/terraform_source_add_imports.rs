use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubnetId;
use cloud_terrastodon_azure::VirtualNetworkId;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_hcl::AsHclString;
use cloud_terrastodon_hcl::DataBlockReference;
use cloud_terrastodon_hcl::HclBlock;
use cloud_terrastodon_hcl::HclDataBlock;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclProviderReference;
use cloud_terrastodon_hcl::HclResourceBlock;
use cloud_terrastodon_hcl::ProviderKind;
use cloud_terrastodon_hcl::ResourceBlockReference;
use cloud_terrastodon_hcl::edit::expr::Expression;
use cloud_terrastodon_hcl::edit::expr::TraversalOperator;
use cloud_terrastodon_hcl::list_blocks_for_dir;
use eyre::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use tracing::warn;

/// Add Terraform import blocks for existing resource blocks.
#[derive(Args, Debug, Clone)]
pub struct TerraformSourceAddImportsArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    #[arg(long, default_value = ".")]
    pub work_dir: PathBuf,
}

impl TerraformSourceAddImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching resources from Azure...");
        let resources = fetch_all_resources(tenant_id)
            .await?
            .into_iter()
            .into_group_map_by(|res| res.name.clone());

        info!("Analyzing Terraform source in {:?}", self.work_dir);
        let code = list_blocks_for_dir(&self.work_dir).await?;
        info!(count=?code.len(), "Discovered HCL blocks");

        info!("Reshaping data...");
        let resource_blocks: HashMap<ResourceBlockReference, &HclResourceBlock> = {
            let mut rtn = HashMap::default();
            for v in &code {
                if let HclBlock::Resource(ref resource_block) = v.hcl_block {
                    let key = resource_block.as_resource_block_reference();
                    if let Some(x) = rtn.insert(key, resource_block) {
                        warn!(
                            found=?x.as_resource_block_reference(),
                            "Duplicate resource blocks found, using latest"
                        )
                    }
                }
            }
            rtn
        };
        let import_blocks: HashMap<&ResourceBlockReference, &HclImportBlock> = {
            let mut rtn = HashMap::default();
            for v in &code {
                if let HclBlock::Import(ref import_block) = v.hcl_block {
                    let key = &import_block.to;
                    if let Some(x) = rtn.insert(key, import_block) {
                        warn!(
                            found=?x,
                            "Duplicate import blocks found, using latest"
                        )
                    }
                }
            }
            rtn
        };
        let data_blocks: HashMap<DataBlockReference, &HclDataBlock> = {
            let mut rtn = HashMap::default();
            for v in &code {
                if let HclBlock::Data(ref data_block) = v.hcl_block {
                    let key = data_block.as_data_block_reference();
                    rtn.insert(key, data_block);
                }
            }
            rtn
        };

        info!("Identifying missing import blocks...");
        for (resource_ref, resource_block) in resource_blocks {
            if import_blocks.contains_key(&resource_ref) {
                continue;
            }
            let Some(name) = resource_block.body().get_attribute("name") else {
                warn!(
                    ?resource_ref,
                    "Resource block missing 'name' attribute, cannot create import block"
                );
                continue;
            };
            let Some(name) = name.value.as_str() else {
                warn!(
                    ?resource_ref,
                    "Resource block 'name' attribute is not a string, cannot create import block"
                );
                continue;
            };
            let Some(candidate_import_ids) = find_candidate_import_ids(
                &resource_ref,
                resource_block,
                name,
                &resources,
                &data_blocks,
            ) else {
                warn!(
                    ?resource_ref,
                    name=%name,
                    "No matching Azure resource found for Terraform resource block, cannot create import block"
                );
                continue;
            };

            for import_id in candidate_import_ids {
                let import_block = HclImportBlock {
                    provider: HclProviderReference::Inherited, // todo: check subscription?
                    to: resource_ref.clone(),
                    id: import_id,
                };
                println!("{}", import_block.as_hcl_string());
            }
        }
        Ok(())
    }
}

fn find_candidate_import_ids(
    resource_ref: &ResourceBlockReference,
    resource_block: &HclResourceBlock,
    name: &str,
    resources_by_name: &HashMap<String, Vec<Resource>>,
    data_blocks: &HashMap<DataBlockReference, &HclDataBlock>,
) -> Option<Vec<String>> {
    if !is_azapi_subnet_resource(resource_ref, resource_block) {
        let candidate_resources = resources_by_name.get(name)?;
        return Some(
            candidate_resources
                .iter()
                .map(|resource| resource.id.expanded_form())
                .collect(),
        );
    }

    let expected_parent = resolve_subnet_parent_context(resource_block, data_blocks)?;
    let matching = resources_by_name
        .values()
        .flat_map(|resources| resources.iter())
        .filter(|resource| subnet_matches_context(resource, name, &expected_parent))
        .map(|resource| resource.id.expanded_form())
        .collect::<Vec<_>>();

    if !matching.is_empty() {
        return Some(matching);
    }

    let vnet_resource =
        find_matching_virtual_network_resource(resources_by_name, &expected_parent)?;
    let ScopeImpl::VirtualNetwork(vnet_id) = &vnet_resource.id else {
        return None;
    };
    let subnet_id = SubnetId::try_new(vnet_id.clone(), name).ok()?;
    Some(vec![subnet_id.expanded_form()])
}

fn is_azapi_subnet_resource(
    resource_ref: &ResourceBlockReference,
    resource_block: &HclResourceBlock,
) -> bool {
    matches!(
        resource_ref,
        ResourceBlockReference::Other {
            provider: ProviderKind::Other(provider),
            kind,
            ..
        } if provider == "azapi" && kind == "resource"
    ) && resource_block
        .body()
        .get_attribute("type")
        .and_then(|attr| attr.value.as_str())
        .is_some_and(|resource_type| {
            resource_type
                .split_once('@')
                .map(|(prefix, _)| prefix)
                .unwrap_or(resource_type)
                .eq_ignore_ascii_case("Microsoft.Network/virtualNetworks/subnets")
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ExpectedSubnetParentContext {
    virtual_network_name: String,
    resource_group_name: String,
}

fn resolve_subnet_parent_context(
    resource_block: &HclResourceBlock,
    data_blocks: &HashMap<DataBlockReference, &HclDataBlock>,
) -> Option<ExpectedSubnetParentContext> {
    let parent_id = resource_block.body().get_attribute("parent_id")?;
    if let Some(parent_id) = parent_id.value.as_str()
        && let Ok(vnet_id) = parent_id.parse::<VirtualNetworkId>()
    {
        return Some(ExpectedSubnetParentContext {
            virtual_network_name: vnet_id.virtual_network_name.to_string(),
            resource_group_name: vnet_id.resource_group_id.resource_group_name.to_string(),
        });
    }

    let vnet_ref = data_block_reference_from_expression(&parent_id.value, &["id"])?;
    let vnet_block = data_blocks.get(&vnet_ref)?;
    if vnet_block.provider_kind() != ProviderKind::AzureRM || vnet_ref.kind() != "virtual_network" {
        return None;
    }

    let virtual_network_name = string_attribute(vnet_block.body(), "name")?.to_owned();
    let resource_group_name = resolve_resource_group_name(vnet_block, data_blocks)?;
    Some(ExpectedSubnetParentContext {
        virtual_network_name,
        resource_group_name,
    })
}

fn resolve_resource_group_name(
    vnet_block: &HclDataBlock,
    data_blocks: &HashMap<DataBlockReference, &HclDataBlock>,
) -> Option<String> {
    let resource_group_name = vnet_block.body().get_attribute("resource_group_name")?;
    if let Some(name) = resource_group_name.value.as_str() {
        return Some(name.to_owned());
    }

    let resource_group_ref =
        data_block_reference_from_expression(&resource_group_name.value, &["name", "id"])?;
    let resource_group_block = data_blocks.get(&resource_group_ref)?;
    if resource_group_block.provider_kind() != ProviderKind::AzureRM
        || resource_group_ref.kind() != "resource_group"
    {
        return None;
    }
    string_attribute(resource_group_block.body(), "name").map(str::to_owned)
}

fn subnet_matches_context(
    resource: &Resource,
    subnet_name: &str,
    expected_parent: &ExpectedSubnetParentContext,
) -> bool {
    let ScopeImpl::Subnet(subnet_id) = &resource.id else {
        return false;
    };

    subnet_id
        .subnet_name
        .to_string()
        .eq_ignore_ascii_case(subnet_name)
        && subnet_id
            .virtual_network_id
            .virtual_network_name
            .to_string()
            .eq_ignore_ascii_case(&expected_parent.virtual_network_name)
        && subnet_id
            .virtual_network_id
            .resource_group_id
            .resource_group_name
            .to_string()
            .eq_ignore_ascii_case(&expected_parent.resource_group_name)
}

fn find_matching_virtual_network_resource<'a>(
    resources_by_name: &'a HashMap<String, Vec<Resource>>,
    expected_parent: &ExpectedSubnetParentContext,
) -> Option<&'a Resource> {
    resources_by_name
        .get(&expected_parent.virtual_network_name)?
        .iter()
        .find(|resource| match &resource.id {
            ScopeImpl::VirtualNetwork(vnet_id) => vnet_id
                .resource_group_id
                .resource_group_name
                .to_string()
                .eq_ignore_ascii_case(&expected_parent.resource_group_name),
            _ => false,
        })
}

fn string_attribute<'a>(
    body: &'a cloud_terrastodon_hcl::edit::structure::Body,
    key: &str,
) -> Option<&'a str> {
    body.get_attribute(key)?.value.as_str()
}

fn data_block_reference_from_expression(
    expression: &Expression,
    terminal_attrs: &[&str],
) -> Option<DataBlockReference> {
    let traversal = expression.as_traversal()?;
    let Expression::Variable(root) = &traversal.expr else {
        return None;
    };
    if root.as_str() != "data" || traversal.operators.len() != 3 {
        return None;
    }

    let kind = traversal_attr_name(&traversal.operators[0])?;
    let name = traversal_attr_name(&traversal.operators[1])?.to_owned();
    let terminal_attr = traversal_attr_name(&traversal.operators[2])?;
    if !terminal_attrs
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(terminal_attr))
    {
        return None;
    }

    if let Some(kind) = kind.strip_prefix("azurerm_") {
        return Some(DataBlockReference::Other {
            provider: ProviderKind::AzureRM,
            kind: kind.to_owned(),
            name,
        });
    }

    let (provider, kind) = kind.split_once('_')?;
    Some(DataBlockReference::Other {
        provider: ProviderKind::Other(provider.to_owned()),
        kind: kind.to_owned(),
        name,
    })
}

fn traversal_attr_name(operator: &TraversalOperator) -> Option<&str> {
    match operator {
        TraversalOperator::GetAttr(attr) => Some(attr.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::ResourceType;
    use cloud_terrastodon_hcl::IntoHclBlocks;
    use indoc::indoc;
    use std::str::FromStr;

    fn parse_hcl_block(input: &str) -> HclBlock {
        let body: cloud_terrastodon_hcl::edit::structure::Body = input.parse().unwrap();
        body.try_into_hcl_blocks()
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    }

    #[test]
    fn finds_matching_subnet_resource_for_azapi_resource() {
        let resource_ref = ResourceBlockReference::Other {
            provider: ProviderKind::Other("azapi".to_owned()),
            kind: "resource".to_owned(),
            name: "vm-subnet".to_owned(),
        };
        let HclBlock::Resource(resource_block) = parse_hcl_block(indoc! {r#"
            resource "azapi_resource" "vm-subnet" {
              type      = "Microsoft.Network/virtualNetworks/subnets@2022-05-01"
              name      = "my-subnet"
              parent_id = data.azurerm_virtual_network.main.id
            }
        "#}) else {
            panic!("expected resource block");
        };
        let HclBlock::Data(vnet_block) = parse_hcl_block(indoc! {r#"
            data "azurerm_virtual_network" "main" {
              name                = "my-vnet"
              resource_group_name = data.azurerm_resource_group.main.name
            }
        "#}) else {
            panic!("expected vnet data block");
        };
        let HclBlock::Data(rg_block) = parse_hcl_block(indoc! {r#"
            data "azurerm_resource_group" "main" {
              name = "my-resource-group"
            }
        "#}) else {
            panic!("expected rg data block");
        };

        let data_blocks = HashMap::from([
            (vnet_block.as_data_block_reference(), &vnet_block),
            (rg_block.as_data_block_reference(), &rg_block),
        ]);
        let subnet_resource = Resource {
            id: ScopeImpl::from_str("/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/my-resource-group/providers/Microsoft.Network/virtualNetworks/my-vnet/subnets/my-subnet").unwrap(),
            kind: ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
            name: "my-vnet/my-subnet".to_owned(),
            tags: HashMap::new(),
            properties: HashMap::new(),
        };
        let wrong_parent_resource = Resource {
            id: ScopeImpl::from_str("/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/other-resource-group/providers/Microsoft.Network/virtualNetworks/other-vnet/subnets/my-subnet").unwrap(),
            kind: ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
            name: "other-vnet/my-subnet".to_owned(),
            tags: HashMap::new(),
            properties: HashMap::new(),
        };
        let resources = HashMap::from([
            (
                "other-vnet/my-subnet".to_owned(),
                vec![wrong_parent_resource],
            ),
            ("my-vnet/my-subnet".to_owned(), vec![subnet_resource]),
        ]);

        let import_ids = find_candidate_import_ids(
            &resource_ref,
            &resource_block,
            "my-subnet",
            &resources,
            &data_blocks,
        )
        .unwrap();

        assert_eq!(import_ids.len(), 1);
        assert_eq!(
            import_ids[0],
            "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/my-resource-group/providers/Microsoft.Network/virtualNetworks/my-vnet/subnets/my-subnet"
        );
    }

    #[test]
    fn resolves_subnet_parent_context_from_vnet_and_rg_data_blocks() {
        let HclBlock::Resource(resource_block) = parse_hcl_block(indoc! {r#"
            resource "azapi_resource" "vm-subnet" {
              type      = "Microsoft.Network/virtualNetworks/subnets@2022-05-01"
              name      = "my-subnet"
              parent_id = data.azurerm_virtual_network.main.id
            }
        "#}) else {
            panic!("expected resource block");
        };
        let HclBlock::Data(vnet_block) = parse_hcl_block(indoc! {r#"
            data "azurerm_virtual_network" "main" {
              name                = "my-vnet"
              resource_group_name = data.azurerm_resource_group.main.name
            }
        "#}) else {
            panic!("expected vnet data block");
        };
        let HclBlock::Data(rg_block) = parse_hcl_block(indoc! {r#"
            data "azurerm_resource_group" "main" {
              name = "my-resource-group"
            }
        "#}) else {
            panic!("expected rg data block");
        };

        let data_blocks = HashMap::from([
            (vnet_block.as_data_block_reference(), &vnet_block),
            (rg_block.as_data_block_reference(), &rg_block),
        ]);

        let context = resolve_subnet_parent_context(&resource_block, &data_blocks).unwrap();
        assert_eq!(
            context,
            ExpectedSubnetParentContext {
                virtual_network_name: "my-vnet".to_owned(),
                resource_group_name: "my-resource-group".to_owned(),
            }
        );
    }

    #[test]
    fn finds_subnet_import_id_from_matching_vnet_when_subnet_resource_is_absent() {
        let resource_ref = ResourceBlockReference::Other {
            provider: ProviderKind::Other("azapi".to_owned()),
            kind: "resource".to_owned(),
            name: "vm-subnet".to_owned(),
        };
        let HclBlock::Resource(resource_block) = parse_hcl_block(indoc! {r#"
            resource "azapi_resource" "vm-subnet" {
              type      = "Microsoft.Network/virtualNetworks/subnets@2022-05-01"
              name      = "my-subnet"
              parent_id = data.azurerm_virtual_network.main.id
            }
        "#}) else {
            panic!("expected resource block");
        };
        let HclBlock::Data(vnet_block) = parse_hcl_block(indoc! {r#"
            data "azurerm_virtual_network" "main" {
              name                = "my-vnet"
              resource_group_name = data.azurerm_resource_group.main.name
            }
        "#}) else {
            panic!("expected vnet data block");
        };
        let HclBlock::Data(rg_block) = parse_hcl_block(indoc! {r#"
            data "azurerm_resource_group" "main" {
              name = "my-resource-group"
            }
        "#}) else {
            panic!("expected rg data block");
        };

        let data_blocks = HashMap::from([
            (vnet_block.as_data_block_reference(), &vnet_block),
            (rg_block.as_data_block_reference(), &rg_block),
        ]);
        let vnet_resource = Resource {
            id: ScopeImpl::from_str("/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/my-resource-group/providers/Microsoft.Network/virtualNetworks/my-vnet").unwrap(),
            kind: ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
            name: "my-vnet".to_owned(),
            tags: HashMap::new(),
            properties: HashMap::new(),
        };
        let resources = HashMap::from([("my-vnet".to_owned(), vec![vnet_resource])]);

        let import_ids = find_candidate_import_ids(
            &resource_ref,
            &resource_block,
            "my-subnet",
            &resources,
            &data_blocks,
        )
        .unwrap();

        assert_eq!(
            import_ids,
            vec!["/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/my-resource-group/providers/Microsoft.Network/virtualNetworks/my-vnet/subnets/my-subnet".to_owned()]
        );
    }

    #[test]
    fn parses_data_block_reference_from_traversal_expression() {
        let expression: Expression = "data.azurerm_virtual_network.main.id".parse().unwrap();

        let reference = data_block_reference_from_expression(&expression, &["id"]).unwrap();

        assert_eq!(
            reference,
            DataBlockReference::Other {
                provider: ProviderKind::AzureRM,
                kind: "virtual_network".to_owned(),
                name: "main".to_owned(),
            }
        );
    }
}
