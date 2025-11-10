use cloud_terrastodon_azure::prelude::RoleAssignmentId;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::Subscription;
use cloud_terrastodon_azure::prelude::SubscriptionId;
use cloud_terrastodon_azure::prelude::SubscriptionScoped;
use cloud_terrastodon_azure::prelude::fetch_all_role_assignments;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_hcl::prelude::HclImportBlock;
use cloud_terrastodon_hcl::prelude::HclProviderBlock;
use cloud_terrastodon_hcl::prelude::HclProviderReference;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_hcl::prelude::ProviderKind;
use cloud_terrastodon_hcl::prelude::Sanitizable;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;

pub async fn write_imports_for_all_role_assignments() -> Result<()> {
    info!("Fetching role assignments");
    let subscriptions = fetch_all_subscriptions()
        .await?
        .into_iter()
        .map(|sub| (sub.id, sub))
        .collect::<HashMap<SubscriptionId, Subscription>>();
    let role_assignments = fetch_all_role_assignments().await?;

    info!("Building import blocks");
    let mut providers: HashSet<HclProviderBlock> = HashSet::new();
    let mut imports: Vec<HclImportBlock> = Vec::with_capacity(role_assignments.len());
    for ra in role_assignments {
        let subscription_id = match &ra.id {
            RoleAssignmentId::Unscoped(_) => None,
            RoleAssignmentId::PortalScoped(_) => None,
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
            let mut import_block: HclImportBlock = ra.into();
            import_block.provider = HclProviderReference::Alias {
                kind: ProviderKind::AzureRM,
                name: sanitized_name.to_owned(),
            };
            imports.push(import_block);
            providers.insert(HclProviderBlock::AzureRM {
                alias: Some(sanitized_name),
                subscription_id: Some(subscription_id.short_form().to_owned()),
            });
        } else {
            let import_block: HclImportBlock = ra.into();
            imports.push(import_block);
            providers.insert(HclProviderBlock::AzureRM {
                alias: None,
                subscription_id: None,
            });
        }
    }

    info!("Writing imports to file");
    HclWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(imports)
        .await?
        .format_file()
        .await?;

    info!("Writing provider blocks");
    HclWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers)
        .await?
        .format_file()
        .await?;

    Ok(())
}
