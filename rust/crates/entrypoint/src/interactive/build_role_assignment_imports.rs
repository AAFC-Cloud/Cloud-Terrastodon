use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::Sanitizable;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderKind;
use cloud_terrastodon_core_tofu::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use eyre::eyre;
use eyre::Result;
use itertools::Itertools;
use std::collections::HashSet;
use tracing::info;

pub async fn build_role_assignment_imports() -> Result<()> {
    info!("Fetching role assignments");
    let found = fetch_all_role_assignments().await?;

    info!("Building import blocks");
    let mut used_subscriptions = HashSet::new();
    let mut seen_ids = HashSet::new();
    let mut import_blocks = Vec::new();
    for (subscription, role_assignments) in found {
        let provider_alias = TofuProviderReference::Alias {
            kind: TofuProviderKind::AzureRM,
            name: subscription.name.sanitize(),
        };
        for role_assignment in role_assignments {
            if seen_ids.insert(role_assignment.id.to_owned()) {
                let mut block: TofuImportBlock = role_assignment.into();
                provider_alias.clone_into(&mut block.provider);
                import_blocks.push(block);
            }
        }
        used_subscriptions.insert(subscription);
    }

    if import_blocks.is_empty() {
        return Err(eyre!("Imports should not be empty"));
    }

    info!("Writing imports to file");
    TofuWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(import_blocks)
        .await?
        .format()
        .await?;

    info!("Writing providers to boilerplate");
    let providers = used_subscriptions
        .into_iter()
        .map(|sub| sub.into_provider_block())
        .collect_vec();
    TofuWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
