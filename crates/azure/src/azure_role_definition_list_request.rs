use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::RoleDefinition;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

/// Fetches all AzureRM role definitions.
///
/// Uses the authorization resources table in
/// [Resource Graph 2022-10-01](https://learn.microsoft.com/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01).
/// These are Azure RBAC roles, distinct from Entra directory roles. The executor
/// shares the assembled list and records its page dependencies in memory.
#[must_use = "Bind this request to an executor and await its execution"]
#[derive(Debug, Clone, Facet, Arbitrary)]
#[facet(traits(Clone))]
pub struct AzureRoleDefinitionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureRoleDefinitionListRequest {}

impl HasCacheKey for AzureRoleDefinitionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role-definitions",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureRoleDefinitionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Arc<Vec<Arc<RoleDefinition>>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let selection = self.authentication.into();
            let fingerprint = NonEmptyString::new(facet_json::to_string(&self)?)?;
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    self.cache_key()?,
                    CacheOperationOwner::of::<Self>(),
                    &selection,
                    fingerprint,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        debug!("Fetching role definitions");
                        let mut helper = ResourceGraphHelper::new(
                            r#"authorizationresources
| where type =~ "microsoft.authorization/roledefinitions"
| project id, properties
| extend
    assignable_scopes = properties.assignableScopes,
    description = properties.description,
    permissions = properties.permissions,
    display_name = properties.roleName,
    ['kind'] = properties.type
| project-away properties"#,
                            Some(self.cache_key()?),
                            self.authentication,
                        );
                        let role_definitions = helper.collect_all(&child).await?;
                        debug!("Found {} role definitions", role_definitions.len());
                        Ok(WorkOutput::new(
                            role_definitions,
                            ArtifactObservation::now(),
                        ))
                    },
                )?
                .await
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureRoleDefinitionListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureRoleDefinitionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureRoleDefinitionListRequest => Arc<Vec<Arc<RoleDefinition>>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureRoleDefinitionListRequest,
    name = "role-definition-list",
    cached
);
