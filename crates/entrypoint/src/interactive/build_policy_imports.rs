use cloud_terrastodon_azure::prelude::PolicyAssignment;
use cloud_terrastodon_azure::prelude::fetch_all_policy_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_azure::prelude::fetch_all_policy_set_definitions;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_hcl::prelude::HCLImportBlock;
use cloud_terrastodon_hcl::prelude::HCLProviderReference;
use cloud_terrastodon_hcl::prelude::HCLWriter;
use cloud_terrastodon_hcl::prelude::ProviderKind;
use cloud_terrastodon_hcl::prelude::Sanitizable;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::eyre;
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::try_join;
use tracing::info;

pub async fn build_policy_imports() -> Result<()> {
    info!("Fetching information");
    let (policy_definitions, policy_set_definitions, policy_assignments, subscriptions) = try_join!(
        fetch_all_policy_definitions(),
        fetch_all_policy_set_definitions(),
        fetch_all_policy_assignments(),
        fetch_all_subscriptions(),
    )?;

    let subscriptions = subscriptions
        .into_iter()
        .map(|sub| (sub.id, sub))
        .collect::<HashMap<_, _>>();

    let mut imports: Vec<HCLImportBlock> = Default::default();
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut provider_blocks: HashSet<_> = Default::default();

    info!("Writing policy definition import blocks");
    for policy_definition in policy_definitions {
        if policy_definition.policy_type == "Custom" {
            let provider = if let Some(subscription_id) = policy_definition.id.subscription_id() {
                let subscription = subscriptions.get(&subscription_id).ok_or(eyre!(format!(
                    "Could not find subscription with id {}",
                    &subscription_id
                )))?;
                let azurerm_provider_block = subscription.into_provider_block();
                provider_blocks.insert(azurerm_provider_block.clone());
                HCLProviderReference::Alias {
                    kind: ProviderKind::AzureRM,
                    name: subscription.name.sanitize(),
                }
            } else {
                HCLProviderReference::Inherited
            };
            let mut block: HCLImportBlock = policy_definition.into();
            block.provider = provider;
            if seen_ids.insert(block.id.clone()) {
                imports.push(block);
            }
        }
    }

    info!("Writing policy set definition import blocks");
    for policy_set_definition in policy_set_definitions {
        if policy_set_definition.policy_type == "Custom" {
            let provider =
                if let Some(subscription_id) = policy_set_definition.id.subscription_id() {
                    let subscription = subscriptions.get(&subscription_id).ok_or(eyre!(
                        format!("Could not find subscription with id {}", &subscription_id)
                    ))?;
                    let azurerm_provider_block = subscription.into_provider_block();
                    provider_blocks.insert(azurerm_provider_block.clone());
                    HCLProviderReference::Alias {
                        kind: ProviderKind::AzureRM,
                        name: subscription.name.sanitize(),
                    }
                } else {
                    HCLProviderReference::Inherited
                };
            let mut block: HCLImportBlock = policy_set_definition.into();
            block.provider = provider;
            if seen_ids.insert(block.id.clone()) {
                imports.push(block);
            }
        }
    }

    info!("Writing policy assignment import blocks");
    for (management_group, policy_assignments) in policy_assignments {
        policy_assignments
            .into_iter()
            .map(|policy_assignment: PolicyAssignment| {
                //todo: filter out inherited assignments that cause the terraform block label to contain a mismatched management group name
                let import_block: HCLImportBlock = policy_assignment.into();
                let provider = import_block.provider;
                let id = import_block.id;
                let mut to = import_block.to;
                to.use_name(|name| format!("{}_{}", name, management_group.name()).sanitize());
                HCLImportBlock { provider, id, to }
            })
            .for_each(|block: HCLImportBlock| {
                if seen_ids.insert(block.id.clone()) {
                    imports.push(block);
                }
            });
    }

    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    info!("Writing boilerplate.tf");
    HCLWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .format_on_write()
        .merge(provider_blocks)
        .await?;

    info!("Writing policy_imports.tf");
    HCLWriter::new(AppDir::Imports.join("policy_imports.tf"))
        .format_on_write()
        .overwrite(imports)
        .await?;

    Ok(())
}
