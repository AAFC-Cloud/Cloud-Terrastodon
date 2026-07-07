use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::KeyVaultId;
use cloud_terrastodon_azure_types::KeyVaultSecret;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(facet::Facet)]
pub struct KeyVaultSecretsListRequest<'a> {
    pub key_vault_id: Cow<'a, KeyVaultId>,
}

pub fn fetch_key_vault_secrets<'a>(key_vault_id: &'a KeyVaultId) -> KeyVaultSecretsListRequest<'a> {
    KeyVaultSecretsListRequest {
        key_vault_id: Cow::Borrowed(key_vault_id),
    }
}

impl<'a> Arbitrary<'a> for KeyVaultSecretsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            key_vault_id: Cow::Owned(KeyVaultId::arbitrary(u)?),
        })
    }
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
            self.key_vault_id
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

cloud_terrastodon_command::impl_cacheable_into_future!(KeyVaultSecretsListRequest<'a>, 'a);

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
            assert!(
                secrets
                    .iter()
                    .all(|secret| !secret.id.to_string().is_empty())
            );
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(KeyVaultSecretsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultSecretsListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(KeyVaultSecretsListRequest<'static> => Vec<KeyVaultSecret>);
