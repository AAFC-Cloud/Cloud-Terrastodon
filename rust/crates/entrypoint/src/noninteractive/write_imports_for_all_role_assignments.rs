use anyhow::bail;
use anyhow::Result;
use azure::prelude::fetch_all_role_assignments_v2;
use azure::prelude::fetch_all_subscriptions;
use azure::prelude::Scope;
use azure::prelude::Subscription;
use azure::prelude::SubscriptionId;
use azure::prelude::SubscriptionScoped;
use pathing::AppDir;
use std::collections::HashMap;
use std::collections::HashSet;
use tofu::prelude::Sanitizable;
use tofu::prelude::TofuImportBlock;
use tofu::prelude::TofuProviderBlock;
use tofu::prelude::TofuProviderKind;
use tofu::prelude::TofuProviderReference;
use tofu::prelude::TofuWriter;
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
            azure::prelude::RoleAssignmentId::Unscoped(_) => None,
            azure::prelude::RoleAssignmentId::ManagementGroupScoped(_) => None,
            azure::prelude::RoleAssignmentId::SubscriptionScoped(id) => {
                Some(id.subscription_id().to_owned())
            }
            azure::prelude::RoleAssignmentId::ResourceGroupScoped(id) => {
                Some(id.subscription_id().to_owned())
            }
            azure::prelude::RoleAssignmentId::ResourceScoped(id) => {
                Some(id.subscription_id().to_owned())
            }
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
