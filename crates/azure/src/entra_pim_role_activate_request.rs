use crate::EntraPimRoleAssignmentsListMineRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentRequest;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use facet::Facet;
use http::Method;
use std::time::Duration;
use url::Url;

/// Submit a self-activation through the account's delegated Graph credential.
///
/// Preserves the existing legacy `privilegedAccess/aadroles` workflow. Microsoft's
/// [official legacy request documentation](https://github.com/microsoftgraph/microsoft-graph-docs-contrib/blob/main/api-reference/beta/api/governanceroleassignmentrequest-post.md)
/// marks this API deprecated; replacing it with PIM v3 requires migrating the
/// eligibility, settings, and activation contracts together. Success acknowledges
/// submission rather than completion of any required approval process.
#[derive(Arbitrary, Facet)]
pub struct EntraPimRoleActivateRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub principal_id: EntraPrincipalId,
    pub role_definition_id: Uuid,
    pub linked_eligible_role_assignment_id: String,
    pub justification: String,
    pub duration: Duration,
}

impl HasCacheKey for EntraPimRoleActivateRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_components([
                "graph",
                "beta",
                "tenants",
                &self.authentication.tenant_id.to_string(),
                "pim",
                "principals",
                &self.principal_id.to_string(),
                "activate",
                &self.linked_eligible_role_assignment_id,
            ])?,
            valid_for: Duration::ZERO,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraPimRoleActivateRequest {}

impl ExecutableRequest for EntraPimRoleActivateRequest {
    type Error = eyre::Report;
    type Output = ();

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let url = Url::parse(
                "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignmentRequests",
            )?;
            let body = facet_json::to_string_pretty(&RoleAssignmentRequest::new_self_activation(
                self.principal_id,
                self.authentication.tenant_id,
                self.role_definition_id,
                self.linked_eligible_role_assignment_id,
                self.justification,
                self.duration,
            ))?;
            executor
                .deferred(
                    "activating an Entra role and invalidating its assignment inventory",
                    |child| async move {
                        let execution = RestRequest::from_method_and_url(Method::POST, url)?
                            .authentication(self.authentication.into())
                            .cache(key)
                            .body(body)
                            .receive_raw_with_decoder(
                                |response| {
                                    response.error_for_status()?;
                                    Ok(())
                                },
                                &child,
                            )
                            .await;
                        // A server-side mutation can succeed before response/artifact handling
                        // reports an error, so retain invalidation for either outcome.
                        let invalidation = EntraPimRoleAssignmentsListMineRequest {
                            authentication: self.authentication,
                        }
                        .invalidate(&child)
                        .await;
                        match (execution, invalidation) {
                            (Ok(()), Ok(())) => Ok(()),
                            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                            (Err(error), Err(invalidation)) => Err(error.wrap_err(format!(
                                "PIM assignment invalidation also failed: {invalidation:#}"
                            ))),
                        }
                    },
                )
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraPimRoleActivateRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraPimRoleActivateRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraPimRoleActivateRequest => (), effects = [Write]);
