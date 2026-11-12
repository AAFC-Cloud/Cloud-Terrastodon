use crate::AzureRoleAssignmentListRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentCreateBody;
use cloud_terrastodon_azure_types::RoleAssignmentCreateProperties;
use cloud_terrastodon_azure_types::RoleAssignmentId;
use cloud_terrastodon_azure_types::RoleAssignmentResource;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Create an Azure RBAC assignment using the selected tenant's credentials.
///
/// [Authorization 2022-04-01](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01).
/// Each invocation uses a fresh assignment UUID. Completion includes invalidating
/// the selected tenant's existing role-assignment inventory.
pub async fn create_role_assignment(
    scope: &impl Scope,
    role_definition_id: &RoleDefinitionId,
    principal_object_id: &Uuid,
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<RoleAssignmentId> {
    let executor = &executor.with_authentication_lease(&(*authentication).into())?;
    let assignment_id: RoleAssignmentId = format!(
        "{}/providers/Microsoft.Authorization/roleAssignments/{}",
        scope.expanded_form().trim_end_matches('/'),
        Uuid::new_v4()
    )
    .parse()?;
    let mut url = Url::parse("https://management.azure.com")?;
    url.set_path(&assignment_id.expanded_form());
    url.query_pairs_mut()
        .append_pair("api-version", "2022-04-01");
    let body = RoleAssignmentCreateBody {
        properties: RoleAssignmentCreateProperties {
            role_definition_id: role_definition_id.clone(),
            principal_id: EntraPrincipalId::new(*principal_object_id),
        },
    };
    let key = CacheKey {
        scope: CacheScope::try_from_relative_path(
            PathBuf::from("rest/arm/tenants")
                .join(authentication.tenant_id.to_string())
                .join("role-assignments/create")
                .join(assignment_id.short_form()),
        )?,
        valid_for: Duration::ZERO,
    };
    executor
        .deferred(
            "creating an RBAC assignment and invalidating its inventory",
            |executor| async move {
                let execution = RestRequest::from_method_and_url(Method::PUT, url)?
                    .authentication((*authentication).into())
                    .body(facet_json::to_string(&body)?)
                    .cache(key)
                    .receive::<RoleAssignmentResource>(&executor)
                    .await;
                // A failed response or artifact write does not prove the remote
                // mutation failed. Refresh the inventory after every attempt.
                let invalidation = AzureRoleAssignmentListRequest {
                    authentication: *authentication,
                }
                .execute(&executor)
                .invalidate()
                .await;
                match (execution, invalidation) {
                    (Ok(response), Ok(())) => Ok(response.id),
                    (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                    (Err(error), Err(invalidation)) => Err(error.wrap_err(format!(
                        "role-assignment inventory invalidation also failed: {invalidation:#}"
                    ))),
                }
            },
        )
        .await
}
