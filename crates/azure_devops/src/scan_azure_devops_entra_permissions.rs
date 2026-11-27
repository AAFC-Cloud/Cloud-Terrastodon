use crate::AzureDevOpsEntraPermissionIdentity;
use crate::AzureDevOpsEntraPermissionOrganization;
use crate::AzureDevOpsEntraPermissionSubject;
use crate::AzureDevOpsIdentityPermissionsRequest;
use crate::AzureDevOpsOrganizationEntraUsersListRequest;
use crate::AzureDevOpsOrganizationGroupsListRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsDescriptor;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGraphUser;
use cloud_terrastodon_azure_devops_types::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::TrackedAzureDevOpsOrganization;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Report;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use std::collections::HashSet;
use std::num::NonZeroUsize;

/// Finds an Entra principal's direct and parent-group Azure DevOps grants in
/// tracked organizations associated with the authenticated tenant.
///
/// Correlates [Graph users](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/users/list?view=azure-devops-rest-7.1)
/// and [Graph groups](https://learn.microsoft.com/en-us/rest/api/azure/devops/graph/groups/list?view=azure-devops-rest-7.1)
/// by their Entra object IDs. Individual lookup failures are returned alongside
/// successful reports so callers can inspect partial results.
pub async fn scan_azure_devops_entra_permissions(
    organizations: Vec<TrackedAzureDevOpsOrganization>,
    authentication: AzureBearerAuthenticationSelection,
    subject: AzureDevOpsEntraPermissionSubject,
    refresh: bool,
    executor: &CloudTerrastodonExecutor,
) -> Result<Vec<AzureDevOpsEntraPermissionOrganization<Report>>> {
    ensure!(
        organizations
            .iter()
            .all(|org| org.tenant_id == Some(authentication.tenant_id)),
        "Azure DevOps organizations must be associated with the authenticated tenant"
    );
    organizations
        .into_iter()
        .map_concurrent(
            executor,
            NonZeroUsize::MIN.saturating_add(2),
            async |executor, organization| {
                let result = scan_organization(
                    &executor,
                    &organization.url,
                    authentication,
                    subject,
                    refresh,
                )
                .await
                .wrap_err_with(|| format!("Looking up identities in {}", organization.url.name()));
                executor.cancellation().bail_if_cancelled()?;
                eyre::Ok(AzureDevOpsEntraPermissionOrganization {
                    organization: organization.url,
                    result,
                })
            },
        )
        .await
}

async fn scan_organization(
    executor: &CloudTerrastodonExecutor,
    organization: &AzureDevOpsOrganizationUrl,
    authentication: AzureBearerAuthenticationSelection,
    subject: AzureDevOpsEntraPermissionSubject,
    refresh: bool,
) -> Result<Vec<AzureDevOpsEntraPermissionIdentity<Report>>> {
    let identities = match subject {
        AzureDevOpsEntraPermissionSubject::Group(group_id) => {
            AzureDevOpsOrganizationGroupsListRequest {
                organization: organization.clone(),
                authentication: authentication.into(),
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await?
            .into_iter()
            .filter(|group| matches_directory_group(group, authentication.tenant_id, group_id))
            .map(|group| (group.display_name.clone(), group.descriptor.clone()))
            .collect::<Vec<_>>()
        }
        AzureDevOpsEntraPermissionSubject::User(user_id) => {
            AzureDevOpsOrganizationEntraUsersListRequest {
                organization: organization.clone(),
                authentication: authentication.into(),
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await?
            .into_iter()
            .filter(|user| matches_directory_user(user, authentication.tenant_id, user_id))
            .map(|user| (user.display_name.to_string(), user.descriptor.clone()))
            .collect()
        }
    };
    let mut seen = HashSet::new();
    let mut reports = Vec::new();
    for (display_name, descriptor) in identities {
        executor.cancellation().bail_if_cancelled()?;
        if !seen.insert(descriptor.clone()) {
            continue;
        }
        let report = AzureDevOpsIdentityPermissionsRequest {
            organization: organization.clone(),
            subject: descriptor.clone(),
            authentication: authentication.into(),
        }
        .execute(executor)
        .with_invalidation(refresh)
        .await;
        executor.cancellation().bail_if_cancelled()?;
        let result = report.wrap_err_with(|| {
            format!(
                "Checking explicit permissions for {descriptor} in {}",
                organization.name()
            )
        });
        reports.push(AzureDevOpsEntraPermissionIdentity {
            display_name,
            descriptor,
            result,
        });
    }
    Ok(reports)
}

fn matches_directory_group(
    group: &AzureDevOpsGroup,
    tenant_id: AzureTenantId,
    group_id: EntraGroupId,
) -> bool {
    // Graph can report an AAD group's DevOps IdentityDomain URI in the domain.
    // The tracked organization's tenant supplies the tenant scope in that case.
    if let Ok(domain) = group.domain.parse::<AzureTenantId>()
        && domain != tenant_id
    {
        return false;
    }
    group.origin.eq_ignore_ascii_case("aad")
        && matches!(group.descriptor, AzureDevOpsDescriptor::EntraGroup(_))
        && group.is_deleted != Some(true)
        && group.origin_id.parse::<EntraGroupId>().ok() == Some(group_id)
}

fn matches_directory_user(
    user: &AzureDevOpsGraphUser,
    tenant_id: AzureTenantId,
    user_id: EntraUserId,
) -> bool {
    // The organization is already scoped to the requested tenant. An explicit
    // conflicting tenant still rules out correlation, even with a matching GUID.
    if let Some(domain) = user
        .domain
        .as_deref()
        .and_then(|domain| domain.parse::<AzureTenantId>().ok())
        && domain != tenant_id
    {
        return false;
    }
    user.origin
        .as_deref()
        .is_some_and(|origin| origin.eq_ignore_ascii_case("aad"))
        && matches!(user.descriptor, AzureDevOpsDescriptor::EntraUser(_))
        && user.is_deleted_in_origin != Some(true)
        && user.origin_id.as_deref().and_then(|id| id.parse().ok()) == Some(user_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlate_user_by_origin_and_reject_conflicting_tenant() -> eyre::Result<()> {
        let tenant_id = "00000000-0000-0000-0000-000000000001".parse()?;
        let user_id: EntraUserId = "00000000-0000-0000-0000-000000000002".parse()?;
        let mut user = AzureDevOpsGraphUser {
            links: None,
            descriptor: "aad.synthetic-user".parse()?,
            directory_alias: None,
            display_name: "Example user".into(),
            domain: Some("00000000-0000-0000-0000-000000000003".to_owned()),
            is_deleted_in_origin: None,
            legacy_descriptor: None,
            mail_address: None,
            meta_type: None,
            metadata_update_date: None,
            origin: Some("aad".to_owned()),
            origin_id: Some(user_id.to_string()),
            principal_name: "example@example.test".to_owned(),
            subject_kind: Some("user".to_owned()),
            url: None,
        };
        assert!(!matches_directory_user(&user, tenant_id, user_id));

        user.domain = None;
        assert!(matches_directory_user(&user, tenant_id, user_id));

        user.origin_id = Some("00000000-0000-0000-0000-000000000004".to_owned());
        assert!(!matches_directory_user(&user, tenant_id, user_id));
        Ok(())
    }
}
