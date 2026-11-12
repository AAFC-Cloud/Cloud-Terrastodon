use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ManagementGroup;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use indoc::indoc;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;
use tracing::info;

pub async fn fetch_root_management_group(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<Arc<ManagementGroup>> {
    info!("Fetching root management group");
    let found = AzureManagementGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?
    .into_iter()
    .find(|mg| mg.name() == mg.tenant_id.to_string());
    match found {
        Some(management_group) => {
            info!("Found root management group");
            Ok(management_group)
        }
        None => {
            let msg = "Failed to find a management group with name matching the tenant ID";
            error!(msg);
            bail!(msg);
        }
    }
}

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureManagementGroupListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureManagementGroupListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureManagementGroupListRequest {}

impl HasCacheKey for AzureManagementGroupListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "management_groups",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureManagementGroupListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ManagementGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!("Fetching management groups");
            let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Management/managementGroups"
        | project 
            tenant_id=tenantId,
            id,
            display_name=properties.displayName,
            management_group_ancestors_chain=properties.details.managementGroupAncestorsChain
    "#};

            let management_groups =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<ManagementGroup>(&executor)
                    .await?;
            let count = management_groups.len();
            if count == 0 {
                bail!(
                    "No management groups found for tenant '{}'",
                    self.authentication.tenant_id
                );
            }
            info!("Found {count} management groups");
            Ok(management_groups)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureManagementGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureManagementGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureManagementGroupListRequest => Vec<Arc<ManagementGroup>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureManagementGroupListRequest,
    name = "management-group-list",
    cached
);
