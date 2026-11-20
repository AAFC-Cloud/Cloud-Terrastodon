use crate::AzureRoleAssignmentListRequest;
use crate::azure_authorization_pages::arbitrary_authorization_scope;
use crate::azure_authorization_pages::collect_azure_authorization_pages;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RoleAssignmentResource;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::ScopeImpl;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use facet::Facet;
use std::sync::Arc;
use url::Url;

/// List role assignments that apply at the requested scope or above through
/// [Azure Authorization 2022-04-01](https://learn.microsoft.com/en-us/rest/api/authorization/role-assignments/list-for-scope?view=rest-authorization-2022-04-01).
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureRoleAssignmentListAtScopeRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub scope: ScopeImpl,
}

impl<'a> Arbitrary<'a> for AzureRoleAssignmentListAtScopeRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(input)?,
            scope: arbitrary_authorization_scope(input)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureRoleAssignmentListAtScopeRequest {}

impl HasCacheKey for AzureRoleAssignmentListAtScopeRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        // Assignment writers already invalidate the tenant-wide assignment
        // inventory. Keep these scoped reads beneath its invalidation prefix.
        let mut key = AzureRoleAssignmentListRequest {
            authentication: self.authentication,
        }
        .cache_key()?
        .join("arm/at-scope/2022-04-01")?;
        for segment in self
            .scope
            .expanded_form()
            .split('/')
            .filter(|segment| !segment.is_empty())
        {
            key = key.join(sanitize_windows_path_component(
                &segment.to_ascii_lowercase(),
            ))?;
        }
        Ok(key)
    }
}

impl ExecutableRequest for AzureRoleAssignmentListAtScopeRequest {
    type Output = Vec<Arc<RoleAssignmentResource>>;
    type Error = eyre::Report;

    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let scope = self.scope.expanded_form();
            let path = scope
                .strip_prefix('/')
                .ok_or_else(|| eyre!("the selected ARM scope must begin with '/'"))?;
            ensure!(
                path.is_empty()
                    || path
                        .split('/')
                        .all(|segment| !matches!(segment, "" | "." | "..")),
                "the selected ARM scope has an invalid path segment"
            );
            let mut url = Url::parse("https://management.azure.com")?;
            url.path_segments_mut()
                .map_err(|_| eyre!("ARM endpoint cannot contain path segments"))?
                .extend(path.split('/').filter(|segment| !segment.is_empty()))
                .extend(["providers", "Microsoft.Authorization", "roleAssignments"]);
            url.query_pairs_mut()
                .append_pair("api-version", "2022-04-01")
                .append_pair("$filter", "atScope()");
            collect_azure_authorization_pages(
                url,
                self.cache_key()?,
                self.authentication,
                &executor,
                CacheOperationOwner::of::<Self>(),
            )
            .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureRoleAssignmentListAtScopeRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleAssignmentListAtScopeRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureRoleAssignmentListAtScopeRequest => Vec<Arc<RoleAssignmentResource>>, effects = [Read]
);
