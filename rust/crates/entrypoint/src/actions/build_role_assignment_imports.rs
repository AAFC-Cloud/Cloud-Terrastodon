use anyhow::anyhow;
use anyhow::Result;
use azure::prelude::fetch_all_role_assignments;
use pathing_types::IgnoreDir;
use std::collections::HashSet;
use tofu::prelude::Sanitizable;
use tofu::prelude::TofuImportBlock;
use tofu::prelude::TofuProviderKind;
use tofu::prelude::TofuProviderReference;
use tofu::prelude::TofuWriter;
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
        return Err(anyhow!("Imports should not be empty"));
    }

    info!("Writing imports to file");
    TofuWriter::new(IgnoreDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(import_blocks)
        .await?
        .format()
        .await?;

    info!("Writing providers to boilerplate");
    let providers = used_subscriptions
        .into_iter()
        .map(|sub| sub.into_provider_block())
        .collect();
    TofuWriter::new(IgnoreDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
