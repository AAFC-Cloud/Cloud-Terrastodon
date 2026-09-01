use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::StorageAccount;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct StorageAccountListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_storage_accounts<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> StorageAccountListRequest<'a> {
    StorageAccountListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for StorageAccountListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for StorageAccountListRequest<'a> {
    type Output = Vec<StorageAccount>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "storage_accounts",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        ResourceGraphHelper::new(
            indoc! {r#"
                Resources
                | where type == "microsoft.storage/storageaccounts"
                | project id,name,kind,location,sku,properties,tags
            "#},
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .collect_all::<StorageAccount>()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(StorageAccountListRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use super::fetch_all_storage_accounts;
    use crate::get_test_tenant_id;
    use crate::is_storage_account_name_available;
    use cloud_terrastodon_azure_types::Slug;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let storage_accounts = fetch_all_storage_accounts(
            &AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?,
        )
        .await?;
        assert!(!storage_accounts.is_empty());
        let check_name = rand::random::<u64>() % storage_accounts.len() as u64;
        for (i, sa) in storage_accounts.into_iter().enumerate() {
            sa.name.validate_slug()?;
            if i == check_name as usize {
                assert!(!is_storage_account_name_available(&sa.name).await?);
            }
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(StorageAccountListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(StorageAccountListRequest<'static> => Vec<StorageAccount>);
