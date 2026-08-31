use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::KeyVault;
use cloud_terrastodon_azure_types::KeyVaultName;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct KeyVaultListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_key_vaults<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> KeyVaultListRequest<'a> {
    KeyVaultListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for KeyVaultListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for KeyVaultListRequest<'a> {
    type Output = Vec<KeyVault>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "key_vaults",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut query = ResourceGraphHelper::new(
            self.tenant_id,
            r#"
resources
| where type =~ "microsoft.keyvault/vaults"
| project id,name,location,properties,tags
        "#,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        query.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(KeyVaultListRequest<'a>, 'a);

#[deprecated(note = "https://github.com/Azure/azure-cli/issues/31178")]
pub async fn is_key_vault_name_available(name: &KeyVaultName) -> eyre::Result<bool> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["keyvault", "check-name", "--name", name]);
    #[derive(facet::Facet)]
    #[allow(unused)]
    struct Response {
        message: String,
        #[facet(rename = "nameAvailable")]
        name_available: bool,
        reason: facet_json::RawJson<'static>,
    }
    let response = cmd.run::<Response>().await?;
    Ok(response.name_available)
}

#[cfg(test)]
mod test {
    use crate::fetch_all_key_vaults;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let key_vaults =
            fetch_all_key_vaults(get_test_tenant_id().await?, &AuthContext::default()).await?;
        assert!(!key_vaults.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(KeyVaultListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(KeyVaultListRequest<'static> => Vec<KeyVault>);
