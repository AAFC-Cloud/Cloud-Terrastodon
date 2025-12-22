use cloud_terrastodon_azure_types::prelude::KeyVaultId;
use cloud_terrastodon_azure_types::prelude::KeyVaultSecret;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

/// Fetch all secrets (metadata only) for a given Key Vault using Azure CLI.
/// Equivalent CLI: az keyvault secret list --vault-name <name>
/// This intentionally does NOT fetch secret values (which would require additional calls per secret).
pub async fn fetch_key_vault_secrets(key_vault_id: &KeyVaultId) -> Result<Vec<KeyVaultSecret>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "keyvault",
        "secret",
        "list",
        "--vault-name",
        key_vault_id.key_vault_name.as_str(),
        "--subscription",
        key_vault_id
            .resource_group_id
            .subscription_id
            .to_string()
            .as_str(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "keyvault",
            "secret",
            "list",
            key_vault_id.key_vault_name.as_str(),
        ]),
        valid_for: Duration::MAX,
    });
    let secrets = cmd.run().await?;
    Ok(secrets)
}

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
