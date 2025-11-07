use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_hcl::prelude::HclImportBlock;
use cloud_terrastodon_hcl::prelude::HclProviderReference;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_hcl::prelude::ProviderKind;
use cloud_terrastodon_hcl::prelude::Sanitizable;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::bail;
use eyre::eyre;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;

pub async fn build_role_assignment_imports() -> Result<()> {
    info!("Fetching role assignments");
    let role_assignments = fetch_all_role_assignments().await?;

    info!("Building import blocks");
    let mut used_subscriptions = HashSet::new();
    let mut seen_ids = HashSet::new();
    let mut import_blocks = Vec::new();

    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id, sub))
        .collect::<HashMap<_, _>>();

    for role_assignment in role_assignments {
        if !seen_ids.insert(role_assignment.id.clone()) {
            continue; // already did this one
        }
        let subscription_id = role_assignment.id.subscription_id();
        let mut import_block: HclImportBlock = role_assignment.into();
        if let Some(subscription_id) = subscription_id {
            let Some(subscription) = subscriptions.get(&subscription_id) else {
                bail!("Failed to find subscription with id {subscription_id}");
            };
            import_block.provider = HclProviderReference::Alias {
                kind: ProviderKind::AzureRM,
                name: subscription.name.sanitize(),
            };
            used_subscriptions.insert(subscription);
        }
        import_blocks.push(import_block);
    }

    if import_blocks.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    info!("Writing imports to file");
    HclWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(import_blocks)
        .await?
        .format_file()
        .await?;

    info!("Writing providers to boilerplate");
    let providers = used_subscriptions
        .into_iter()
        .map(|sub| sub.into_provider_block())
        .collect_vec();
    HclWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format_file()
        .await?;

    Ok(())
}
