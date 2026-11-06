use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ManagementGroup;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::error;
use tracing::info;

pub async fn fetch_root_management_group(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<Arc<ManagementGroup>> {
    info!("Fetching root management group");
    let found = fetch_all_management_groups(auth_context, executor)
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
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct ManagementGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_management_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ManagementGroupListRequest<'a>> {
    executor.execute(ManagementGroupListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for ManagementGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for ManagementGroupListRequest<'a> {}

impl<'a> HasCacheKey for ManagementGroupListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "management_groups",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for ManagementGroupListRequest<'a> {
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

            let management_groups = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .collect_all::<ManagementGroup>(&executor)
            .await?;
            let count = management_groups.len();
            if count == 0 {
                bail!(
                    "No management groups found for tenant '{}'",
                    self.auth_context.tenant_id
                );
            }
            info!("Found {count} management groups");
            Ok(management_groups)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let result = fetch_all_management_groups(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                assert!(!result.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ManagementGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ManagementGroupListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ManagementGroupListRequest<'static> => Vec<Arc<ManagementGroup>>, effects = [Read]);
