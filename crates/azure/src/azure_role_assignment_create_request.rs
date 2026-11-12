use crate::AzureRoleAssignmentListRequest;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::RoleAssignmentCreateBody;
use cloud_terrastodon_azure_types::RoleAssignmentCreateProperties;
use cloud_terrastodon_azure_types::RoleAssignmentId;
use cloud_terrastodon_azure_types::RoleAssignmentResource;
use cloud_terrastodon_azure_types::RoleDefinitionId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Create an Azure RBAC assignment at its explicit, serializable resource ID.
/// The caller chooses its assignment UUID before execution so reviewing or
/// retrying the same request does not silently create a different assignment.
/// [Authorization 2022-04-01](https://learn.microsoft.com/rest/api/authorization/role-assignments/create?view=rest-authorization-2022-04-01).
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureRoleAssignmentCreateRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub id: Cow<'a, RoleAssignmentId>,
    pub role_definition_id: Cow<'a, RoleDefinitionId>,
    pub principal_id: EntraPrincipalId,
}
impl<'a> Arbitrary<'a> for AzureRoleAssignmentCreateRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            id: Cow::Owned(Arbitrary::arbitrary(u)?),
            role_definition_id: Cow::Owned(Arbitrary::arbitrary(u)?),
            principal_id: Arbitrary::arbitrary(u)?,
        })
    }
}

impl HasCacheKey for AzureRoleAssignmentCreateRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        let mut path = PathBuf::from("rest/arm/tenants")
            .join(self.authentication.tenant_id.to_string())
            .join("role-assignments/create");
        for component in self.id.expanded_form().trim_start_matches('/').split('/') {
            path.push(sanitize_windows_path_component(component));
        }
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(path)?,
            valid_for: Duration::ZERO,
        })
    }
}
impl InvalidatesOwnCacheKey for AzureRoleAssignmentCreateRequest<'_> {}
impl ExecutableRequest for AzureRoleAssignmentCreateRequest<'_> {
    type Output = RoleAssignmentId;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let key = self.cache_key()?;
            let mut url = Url::parse("https://management.azure.com")?;
            url.set_path(&self.id.expanded_form());
            url.query_pairs_mut()
                .append_pair("api-version", "2022-04-01");
            let body = RoleAssignmentCreateBody {
                properties: RoleAssignmentCreateProperties {
                    role_definition_id: self.role_definition_id.into_owned(),
                    principal_id: self.principal_id,
                },
            };
            executor.deferred("creating an RBAC assignment and invalidating its inventory", async |child| {
                let execution = RestRequest::from_method_and_url(Method::PUT, url)?
                    .authentication(self.authentication.into())
                    .body(facet_json::to_string(&body)?)
                    .cache(key)
                    .receive::<RoleAssignmentResource>(&child)
                    .await;
                // A failed response or artifact write does not prove the remote
                // mutation failed. Refresh the inventory after every attempt.
                let invalidation = AzureRoleAssignmentListRequest { authentication: self.authentication }
                    .execute(&child).invalidate().await;
                match (execution, invalidation) {
                    (Ok(response), Ok(())) => Ok(response.id),
                    (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                    (Err(error), Err(invalidation)) => Err(error.wrap_err(format!("role-assignment inventory invalidation also failed: {invalidation:#}"))),
                }
            }).await
        })
    }
}
cloud_terrastodon_registry::register_thing!(AzureRoleAssignmentCreateRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleAssignmentCreateRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRoleAssignmentCreateRequest<'static> => RoleAssignmentId, effects = [Write]);
