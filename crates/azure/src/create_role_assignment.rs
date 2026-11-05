use crate::fetch_all_role_assignments;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentCreateBody;
use cloud_terrastodon_azure_types::RoleAssignmentCreateProperties;
use cloud_terrastodon_azure_types::RoleAssignmentId;
use cloud_terrastodon_azure_types::RoleAssignmentResource;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
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
    auth_context: &AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> Result<RoleAssignmentId> {
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
        path: PathBuf::from("rest/arm/tenants")
            .join(auth_context.tenant_id.to_string())
            .join("role-assignments/create")
            .join(assignment_id.short_form()),
        valid_for: Duration::ZERO,
    };
    cancellation
        .deferred(
            "creating an RBAC assignment and invalidating its inventory",
            |scope| async move {
                let execution = RestRequest::from_method_and_url(Method::PUT, url, &scope)?
                    .azure_auth_context(auth_context)
                    .body(facet_json::to_string(&body)?)
                    .cache(key)
                    .receive::<RoleAssignmentResource>()
                    .await;
                // A failed response or artifact write does not prove the remote
                // mutation failed. Refresh the inventory after every attempt.
                let invalidation = fetch_all_role_assignments(auth_context, &scope)
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
