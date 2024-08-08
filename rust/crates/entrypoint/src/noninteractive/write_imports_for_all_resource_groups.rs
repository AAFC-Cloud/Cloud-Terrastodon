use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_all_resource_groups;
use azure::prelude::fetch_all_subscriptions;
use azure::prelude::Subscription;
use azure::prelude::SubscriptionId;
use pathing::AppDir;
use std::collections::HashMap;
use std::collections::HashSet;
use tofu::prelude::Sanitizable;
use tofu::prelude::TofuImportBlock;
use tofu::prelude::TofuProviderKind;
use tofu::prelude::TofuProviderReference;
use tofu::prelude::TofuWriter;
use tracing::info;

pub async fn write_imports_for_all_resource_groups() -> Result<()> {
    info!("Writing imports for all resource groups");

    info!("Fetching resource groups");
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id.clone(), sub))
        .collect::<HashMap<SubscriptionId, Subscription>>();

    let resource_groups = fetch_all_resource_groups().await?;

    info!("Building import blocks");
    let mut used_subscriptions = HashSet::new();
    let mut imports = Vec::with_capacity(resource_groups.len());
    for rg in resource_groups {
        let sub = subscriptions
            .get(&rg.subscription_id)
            .ok_or_else(|| anyhow!("could not find subscription for resource group {rg:?}"))?;
        let mut block: TofuImportBlock = rg.into();
        block.provider = TofuProviderReference::Alias {
            kind: TofuProviderKind::AzureRM,
            name: sub.name.sanitize(),
        };
        imports.push(block);

        used_subscriptions.insert(sub);
    }
    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    info!("Writing import blocks");
    TofuWriter::new(AppDir::Imports.join("resource_group_imports.tf"))
        .overwrite(imports)
        .await?
        .format()
        .await?;

    info!("Writing provider blocks");
    let mut providers = Vec::new();
    for sub in used_subscriptions {
        let provider = sub.clone().into_provider_block();
        providers.push(provider);
    }
    TofuWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
