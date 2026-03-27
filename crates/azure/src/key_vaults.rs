use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::KeyVault;
use cloud_terrastodon_azure_types::KeyVaultName;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct KeyVaultListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_key_vaults(tenant_id: AzureTenantId) -> KeyVaultListRequest {
    KeyVaultListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for KeyVaultListRequest {
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
        );
        query.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(KeyVaultListRequest);

#[deprecated(note = "https://github.com/Azure/azure-cli/issues/31178")]
pub async fn is_key_vault_name_available(name: &KeyVaultName) -> eyre::Result<bool> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["keyvault", "check-name", "--name", name]);
    #[derive(Deserialize)]
    #[allow(unused)]
    struct Response {
        message: String,
        #[serde(rename = "nameAvailable")]
        name_available: bool,
        reason: Value,
    }
    let response = cmd.run::<Response>().await?;
    Ok(response.name_available)
}

#[cfg(test)]
mod test {
    use crate::fetch_all_key_vaults;
    use crate::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let key_vaults = fetch_all_key_vaults(get_test_tenant_id().await?).await?;
        assert!(!key_vaults.is_empty());
        Ok(())
    }
}
