use cloud_terrastodon_azure_types::prelude::KeyVaultId;
use cloud_terrastodon_azure_types::prelude::KeyVaultSecret;
use cloud_terrastodon_command::{CacheKey, CacheableCommand};
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::impl_cacheable_into_future;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

pub struct KeyVaultSecretsListRequest<'a> {
    key_vault_id: &'a KeyVaultId,
}

pub fn fetch_key_vault_secrets<'a>(key_vault_id: &'a KeyVaultId) -> KeyVaultSecretsListRequest<'a> {
    KeyVaultSecretsListRequest { key_vault_id }
}

#[async_trait]
impl<'a> CacheableCommand for KeyVaultSecretsListRequest<'a> {
    type Output = Vec<KeyVaultSecret>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "keyvault",
            "secret",
            "list",
            self.key_vault_id.key_vault_name.as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "keyvault",
            "secret",
            "list",
            "--vault-name",
            self.key_vault_id.key_vault_name.as_str(),
            "--subscription",
            self
                .key_vault_id
                .resource_group_id
                .subscription_id
                .to_string()
                .as_str(),
            "--output",
            "json",
        ]);
        cmd.cache(self.cache_key());
        let secrets = cmd.run().await?;
        Ok(secrets)
    }
}

impl_cacheable_into_future!(KeyVaultSecretsListRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[ignore] // network + requires az login and existing vault
    async fn fetch_some() -> eyre::Result<()> {
        // Provide a real key vault id via env var for manual testing
        if let Ok(expanded) = std::env::var("TEST_KEY_VAULT_ID") {
            let id: KeyVaultId = expanded.parse()?;
                let secrets = fetch_key_vault_secrets(&id).await?;
            println!("Fetched {} secrets", secrets.len());
        }
        Ok(())
    }
}
