use crate::AzureEligibleChildResourcesAtScopeRequest;
use crate::AzureManagementGroupListRequest;
use crate::AzureResourceGroupListRequest;
use crate::FetchChildrenBehaviour;
use crate::azure_management_group_list_request::fetch_root_management_group;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AsScope;
use cloud_terrastodon_azure_types::EligibleChildResource;
use cloud_terrastodon_azure_types::EligibleChildResourceKind;
use cloud_terrastodon_azure_types::ManagementGroupId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use facet::Facet;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureEligibleChildResourceListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureEligibleChildResourceListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl CacheInvalidationKeys for AzureEligibleChildResourceListRequest {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        // The root-group lookup and both inventory reads contribute to this composition.
        Ok(vec![
            AzureManagementGroupListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
            AzureEligibleChildResourcesAtScopeRequest {
                scope: ManagementGroupId::from_name(&self.authentication.tenant_id.to_string())
                    .as_scope_impl(),
                behaviour: FetchChildrenBehaviour::GetAllChildren,
                authentication: self.authentication,
            }
            .cache_key()?,
            AzureResourceGroupListRequest {
                authentication: self.authentication,
            }
            .cache_key()?,
        ])
    }
}

impl ExecutableRequest for AzureEligibleChildResourceListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<EligibleChildResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let root_mg = fetch_root_management_group(&self.authentication, &executor).await?;
            let scope = root_mg.as_scope();
            let mut resource_containers = AzureEligibleChildResourcesAtScopeRequest {
                scope: (scope).as_scope_impl(),
                behaviour: (FetchChildrenBehaviour::GetAllChildren),
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?;
            // this contains management groups and subscriptions

            let rgs = AzureResourceGroupListRequest {
                authentication: self.authentication,
            }
            .execute(&executor)
            .await?
            .into_iter()
            .map(|x| {
                Arc::new(EligibleChildResource {
                    name: x.name.to_string(),
                    kind: EligibleChildResourceKind::ResourceGroup,
                    id: x.as_scope().as_scope_impl(),
                })
            });
            resource_containers.extend(rgs);
            // extend to include resource groups

            Ok(resource_containers)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureEligibleChildResourceListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureEligibleChildResourceListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureEligibleChildResourceListRequest => Vec<Arc<EligibleChildResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureEligibleChildResourceListRequest,
    name = "eligible-child-resource-list",
    cached
);
