use crate::AzureLocationName;
use crate::AzureTenantId;
use crate::ResourceGroupId;
use crate::ResourceGroupName;
use crate::SubscriptionName;
use crate::SubscriptionScoped;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use std::collections::HashMap;
#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct ResourceGroup {
    pub id: ResourceGroupId,
    pub subscription_name: SubscriptionName,
    pub tenant_id: AzureTenantId,
    pub location: AzureLocationName,
    pub managed_by: Option<String>,
    pub name: ResourceGroupName,
    pub properties: HashMap<String, String>,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
}

impl AsScope for ResourceGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &ResourceGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for ResourceGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
impl From<ResourceGroup> for HclImportBlock {
    fn from(resource_group: ResourceGroup) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: resource_group.id.to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::ResourceGroup,
                name: format!(
                    "{}__{}",
                    resource_group.name,
                    resource_group.id.subscription_id().as_hyphenated()
                )
                .sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceGroupName;
    use crate::SubscriptionId;
    use crate::slug::Slug;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let id = ResourceGroupId::new(
            SubscriptionId::new(Uuid::nil()),
            ResourceGroupName::try_new("bruh")?,
        );
        let expanded = id.expanded_form();
        let x: ResourceGroupId = facet_json::from_str(&facet_json::to_string(&expanded)?)?;
        assert_eq!(x, id);
        assert_eq!(x.expanded_form(), expanded);
        assert_eq!(x.to_string(), expanded);

        Ok(())
    }

    #[test]
    fn null_tags_default_to_empty_map() -> Result<()> {
        let id = ResourceGroupId::new(
            SubscriptionId::new(Uuid::nil()),
            ResourceGroupName::try_new("bruh")?,
        );
        let json = format!(
            r#"{{
                "id": "{}",
                "subscription_name": "Example Subscription",
                "tenant_id": "00000000-0000-0000-0000-000000000000",
                "location": "eastus",
                "managed_by": null,
                "name": "bruh",
                "properties": {{}},
                "tags": null
            }}"#,
            id.expanded_form()
        );

        let resource_group = facet_json::from_str::<ResourceGroup>(&json)?;
        assert!(resource_group.tags.is_empty());
        let reparsed =
            facet_json::from_str::<ResourceGroup>(&facet_json::to_string(&resource_group)?)?;
        assert_eq!(resource_group, reparsed);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ResourceGroup);
cloud_terrastodon_registry::register_arbitrary!(ResourceGroup);

cloud_terrastodon_registry::register_arbitrary!(Vec<ResourceGroup>);
