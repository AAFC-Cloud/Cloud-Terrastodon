use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_azure_types::StorageAccountBlobContainerName;
use cloud_terrastodon_azure_types::StorageAccountId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

/// This can fail due to network rules on the storage account
#[derive(facet::Facet)]
pub struct StorageAccountBlobContainerNamesListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub storage_account_id: Cow<'a, StorageAccountId>,
}

pub fn fetch_storage_account_blob_container_names<'a>(
    storage_account_id: &'a StorageAccountId,
    cancellation: &CancellationScope,
) -> StorageAccountBlobContainerNamesListRequest<'a> {
    StorageAccountBlobContainerNamesListRequest {
        cancellation: cancellation.clone(),
        storage_account_id: Cow::Borrowed(storage_account_id),
    }
}

impl<'a> Arbitrary<'a> for StorageAccountBlobContainerNamesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: Arbitrary::arbitrary(u)?,
            storage_account_id: Cow::Owned(StorageAccountId::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand
    for StorageAccountBlobContainerNamesListRequest<'a>
{
    type Output = HashSet<StorageAccountBlobContainerName>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "storage_accounts",
            self.storage_account_id.expanded_form().as_ref(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.cancellation_safe();
        let subscription = self
            .storage_account_id
            .resource_group_id
            .subscription_id
            .as_hyphenated()
            .to_string();
        cmd.args([
            "storage",
            "container",
            "list",
            "--account-name",
            &self.storage_account_id.storage_account_name,
            "--subscription",
            subscription.as_ref(),
            "--query",
            "[].name",
            "--output",
            "json",
            "--auth-mode",
            "login",
        ]);
        cmd.cache(self.cache_key());

        let rtn = cmd.run(&self.cancellation).await?;
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(
    StorageAccountBlobContainerNamesListRequest<'a>,
    'a
);

#[cfg(test)]
mod test {
    use crate::fetch_all_storage_accounts;
    use crate::fetch_storage_account_blob_container_names;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use eyre::bail;

    #[tokio::test]
    pub async fn blob_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let storage_accounts = fetch_all_storage_accounts(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &cancellation,
                )
                .await?;
                for sa in storage_accounts.into_iter() {
                    if let Ok(blob_containers) =
                        fetch_storage_account_blob_container_names(&sa.id, &cancellation).await
                    {
                        assert!(!sa.name.is_empty());
                        assert!(
                            blob_containers
                                .iter()
                                .all(|name| !name.to_string().is_empty())
                        );
                        return Ok(());
                    }
                }
                bail!("Failed to get any blob containers D:");
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerNamesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(
    StorageAccountBlobContainerNamesListRequest<'static>
);
cloud_terrastodon_registry::register_into_future!(StorageAccountBlobContainerNamesListRequest<'static> => HashSet<StorageAccountBlobContainerName>, effects = [Read]);
