use crate::MyEntraPimRoleAssignmentsListRequest;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_azure_types::RoleAssignmentRequest;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use http::Method;
use std::time::Duration;
use url::Url;

/// Activate an Entra role through the executor's delegated Graph credential.
/// See [PIM role-assignment requests](https://learn.microsoft.com/graph/api/resources/governanceroleassignmentrequest?view=graph-rest-beta).
pub async fn activate_pim_entra_role(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    principal_id: impl Into<EntraPrincipalId>,
    role_assignment: &GovernanceRoleAssignment,
    justification: String,
    duration: Duration,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let principal_id = principal_id.into();
    let url = Url::parse(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignmentRequests",
    )?;
    let body = facet_json::to_string_pretty(&RoleAssignmentRequest::new_self_activation(
        principal_id,
        tenant_id,
        role_assignment,
        justification,
        duration,
    ))?;
    let key = CacheKey {
        scope: CacheScope::try_from_components([
            "graph",
            "beta",
            "tenants",
            &tenant_id.to_string(),
            "pim",
            "principals",
            &principal_id.to_string(),
            "activate",
            &role_assignment.id,
        ])?,
        valid_for: Duration::ZERO,
    };
    executor
        .deferred(
            "activating an Entra role and invalidating its assignment inventory",
            |child| async move {
                let execution = RestRequest::from_method_and_url(Method::POST, url)?
                    .authentication(AuthenticationSelection::AzureBearer {
                        auth_source,
                        tenant_id,
                    })
                    .cache(key)
                    .body(body)
                    .receive_raw(&child)
                    .await
                    .and_then(|response| response.error_for_status().map_err(Into::into));
                // A server-side mutation can succeed before response/artifact handling
                // reports an error, so retain invalidation for either outcome.
                let invalidation = MyEntraPimRoleAssignmentsListRequest {
                    tenant_id,
                    auth_source,
                }
                .invalidate(&child)
                .await;
                match (execution, invalidation) {
                    (Ok(_), Ok(())) => Ok(()),
                    (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                    (Err(error), Err(invalidation)) => Err(error.wrap_err(format!(
                        "PIM assignment invalidation also failed: {invalidation:#}"
                    ))),
                }
            },
        )
        .await
}
