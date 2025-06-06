use cloud_terrastodon_azure::prelude::Subscription;
use cloud_terrastodon_azure::prelude::SubscriptionId;
use cloud_terrastodon_azure::prelude::fetch_all_resource_groups;
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
use tracing::info;

pub async fn write_imports_for_all_resource_groups() -> Result<()> {
    info!("Writing imports for all resource groups");

    info!("Fetching resource groups");
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id, sub))
        .collect::<HashMap<SubscriptionId, Subscription>>();

    let resource_groups = fetch_all_resource_groups().await?;

    info!("Building import blocks");
    let mut used_subscriptions = HashSet::new();
    let mut imports = Vec::with_capacity(resource_groups.len());
    for rg in resource_groups {
        let sub = subscriptions
            .get(&rg.subscription_id)
            .ok_or_else(|| eyre!("could not find subscription for resource group {rg:?}"))?;
        let mut block: HCLImportBlock = rg.into();
        block.provider = HCLProviderReference::Alias {
            kind: ProviderKind::AzureRM,
            name: sub.name.sanitize(),
        };
        imports.push(block);

        used_subscriptions.insert(sub);
    }
    if imports.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    info!("Writing import blocks");
    HCLWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(imports)
        .await?
        .format_file()
        .await?;

    info!("Writing provider blocks");
    let mut providers = Vec::new();
    for sub in used_subscriptions {
        let provider = sub.clone().into_provider_block();
        providers.push(provider);
    }
    HCLWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format_file()
        .await?;

    Ok(())
}
