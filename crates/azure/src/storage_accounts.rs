use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::StorageAccount;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct StorageAccountListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_storage_accounts<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, StorageAccountListRequest<'a>> {
    executor.execute(StorageAccountListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for StorageAccountListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for StorageAccountListRequest<'a> {}

impl<'a> HasCacheKey for StorageAccountListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "storage_accounts",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for StorageAccountListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<StorageAccount>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ResourceGraphHelper::new(
                indoc! {r#"
                Resources
                | where type == "microsoft.storage/storageaccounts"
                | project id,name,kind,location,sku,properties,tags
            "#},
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .collect_all::<StorageAccount>(&executor)
            .await
        })
    }
}

#[cfg(test)]
mod test {
    use super::fetch_all_storage_accounts;
    use crate::get_test_tenant_id;
    use crate::is_storage_account_name_available;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::Slug;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli()
                    .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let storage_accounts = fetch_all_storage_accounts(&auth_context, &executor).await?;
                assert!(!storage_accounts.is_empty());
                let check_name = rand::random::<u64>() % storage_accounts.len() as u64;
                for (i, sa) in storage_accounts.into_iter().enumerate() {
                    sa.name.validate_slug()?;
                    if i == check_name as usize {
                        assert!(
                            !is_storage_account_name_available(
                                &sa.name,
                                sa.id.resource_group_id.subscription_id,
                                &auth_context,
                                &executor
                            )
                            .await?
                        );
                    }
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(StorageAccountListRequest<'static> => Vec<Arc<StorageAccount>>, effects = [Read]);
