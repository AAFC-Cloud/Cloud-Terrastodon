use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureRoleAssignmentListRequest;
use cloud_terrastodon_azure::AzureSubscriptionListRequest;
use cloud_terrastodon_azure::RoleAssignmentId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionScoped;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclProviderBlock;
use cloud_terrastodon_hcl::HclProviderReference;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_hcl::ProviderKind;
use cloud_terrastodon_hcl::Sanitizable;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;

pub async fn write_imports_for_all_role_assignments(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let cancellation = executor.cancellation();
    info!("Fetching role assignments");
    let subscriptions = AzureSubscriptionListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?
    .into_iter()
    .map(|sub| (sub.id, sub))
    .collect::<HashMap<_, _>>();
    let role_assignments = AzureRoleAssignmentListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;

    info!("Building import blocks");
    let mut providers: HashSet<HclProviderBlock> = HashSet::new();
    let mut imports: Vec<HclImportBlock> = Vec::with_capacity(role_assignments.len());
    for ra in role_assignments.iter() {
        let subscription_id = match &ra.id {
            RoleAssignmentId::Unscoped(_) => None,
            RoleAssignmentId::PortalScoped(_) => None,
            RoleAssignmentId::ServiceGroupScoped(_) => None,
            RoleAssignmentId::ManagementGroupScoped(_) => None,
            RoleAssignmentId::SubscriptionScoped(id) => Some(id.subscription_id().to_owned()),
            RoleAssignmentId::ResourceGroupScoped(id) => Some(id.subscription_id().to_owned()),
            RoleAssignmentId::ResourceScoped(id) => Some(id.subscription_id().to_owned()),
        };

        if let Some(subscription_id) = subscription_id {
            let Some(sub) = subscriptions.get(&subscription_id) else {
                bail!("could not find subscription for role assignment {ra:?}");
            };
            let sanitized_name = sub.name.sanitize();
            let mut import_block: HclImportBlock = ra.as_ref().into();
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
            let import_block: HclImportBlock = ra.as_ref().into();
            imports.push(import_block);
            providers.insert(HclProviderBlock::AzureRM {
                alias: None,
                subscription_id: None,
            });
        }
    }

    info!("Writing imports to file");
    HclWriter::new(AppDir::Imports.join("role_assignment_imports.tf"))
        .overwrite(imports, cancellation)
        .await?
        .format_file(cancellation)
        .await?;

    info!("Writing provider blocks");
    HclWriter::new(AppDir::Imports.join("boilerplate.tf"))
        .merge(providers, cancellation)
        .await?
        .format_file(cancellation)
        .await?;

    Ok(())
}
