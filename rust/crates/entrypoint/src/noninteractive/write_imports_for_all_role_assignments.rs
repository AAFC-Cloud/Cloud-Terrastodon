use anyhow::bail;
use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure::prelude::RoleAssignmentId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use cloud_terrastodon_core_azure::prelude::SubscriptionScoped;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_core_tofu::prelude::Sanitizable;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderKind;
use cloud_terrastodon_core_tofu::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu::prelude::TofuWriter;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;

pub async fn write_imports_for_all_role_assignments() -> Result<()> {
    info!("Fetching role assignments");
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id.clone(), sub))
        .collect::<HashMap<SubscriptionId, Subscription>>();
    let role_assignments = fetch_all_role_assignments_v2().await?;

    info!("Building import blocks");
    let mut providers: HashSet<TofuProviderBlock> = HashSet::new();
    let mut imports: Vec<TofuImportBlock> = Vec::with_capacity(role_assignments.len());
    for ra in role_assignments {
        let subscription_id = match &ra.id {
            RoleAssignmentId::Unscoped(_) => None,
            RoleAssignmentId::ManagementGroupScoped(_) => None,
            RoleAssignmentId::SubscriptionScoped(id) => Some(id.subscription_id().to_owned()),
            RoleAssignmentId::ResourceGroupScoped(id) => Some(id.subscription_id().to_owned()),
            RoleAssignmentId::ResourceScoped(id) => Some(id.subscription_id().to_owned()),
        };

        if let Some(subscription_id) = subscription_id {
            let Some(sub) = subscriptions.get(&subscription_id) else {
                bail!("could not find subscription for role assignment {ra:?}")
            };
            let sanitized_name = sub.name.sanitize();
            let mut import_block: TofuImportBlock = ra.into();
            import_block.provider = TofuProviderReference::Alias {
                kind: TofuProviderKind::AzureRM,
                name: sanitized_name.to_owned(),
            };
            imports.push(import_block);
            providers.insert(TofuProviderBlock::AzureRM {
                alias: Some(sanitized_name),
                subscription_id: Some(subscription_id.short_form().to_owned()),
            });
        } else {
            let import_block: TofuImportBlock = ra.into();
            imports.push(import_block);
            providers.insert(TofuProviderBlock::AzureRM {
                alias: None,
                subscription_id: None,
            });
        }
    }

    info!("Writing imports to file");
    TofuWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(imports)
        .await?
        .format()
        .await?;

    info!("Writing provider blocks");
    TofuWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format()
        .await?;

    Ok(())
}
