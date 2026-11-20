use crate::azure_authorization_pages::arbitrary_authorization_scope;
use crate::azure_authorization_pages::collect_azure_authorization_pages;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureRoleDefinitionResource;
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
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// List role definitions available at the requested scope or above through
/// [Azure Authorization 2022-04-01](https://learn.microsoft.com/en-us/rest/api/authorization/role-definitions/list?view=rest-authorization-2022-04-01).
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureRoleDefinitionListAtScopeRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub scope: ScopeImpl,
}

impl<'a> Arbitrary<'a> for AzureRoleDefinitionListAtScopeRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(input)?,
            scope: arbitrary_authorization_scope(input)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureRoleDefinitionListAtScopeRequest {}

impl HasCacheKey for AzureRoleDefinitionListAtScopeRequest {
    fn cache_key(&self) -> Result<CacheKey> {
        let mut path = PathBuf::from("az/arm/tenants")
            .join(self.authentication.tenant_id.to_string())
            .join("authorization/role-definitions/2022-04-01");
        for segment in self
            .scope
            .expanded_form()
            .split('/')
            .filter(|segment| !segment.is_empty())
        {
            path.push(sanitize_windows_path_component(
                &segment.to_ascii_lowercase(),
            ));
        }
        CacheKey::from_relative_path(path)
    }
}

impl ExecutableRequest for AzureRoleDefinitionListAtScopeRequest {
    type Output = Vec<Arc<AzureRoleDefinitionResource>>;
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
                .extend(["providers", "Microsoft.Authorization", "roleDefinitions"]);
            url.query_pairs_mut()
                .append_pair("api-version", "2022-04-01");
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

cloud_terrastodon_registry::register_thing!(AzureRoleDefinitionListAtScopeRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleDefinitionListAtScopeRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureRoleDefinitionListAtScopeRequest => Vec<Arc<AzureRoleDefinitionResource>>, effects = [Read]
);
