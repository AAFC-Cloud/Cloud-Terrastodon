#![cfg(test)]

mod support;

use bstr::ByteSlice;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::sync::Arc;
use std::time::Duration;

struct AzureCliVersion;

#[tokio::test]
#[ignore = "requires a configured local Azure CLI executable; performs no service request"]
async fn local_cli_metadata_uses_explicit_execution() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let result = CommandBuilder::new(CommandKind::AzureCLI)
                .args(["--version"])
                .run_raw(&scope)
                .await?;
            assert!(result.success());
            assert!(!result.stdout.trim().is_empty());
            Ok(())
        })
        .await
}

#[tokio::test]
#[ignore = "requires a configured local Azure CLI executable; performs no service request"]
async fn local_cli_metadata_can_share_a_canonical_raw_result() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = support::executor(scope)?;
            let mut command = CommandBuilder::new(CommandKind::AzureCLI);
            command.args(["--version"]);
            command.cache(support::key(Duration::from_secs(60)));
            let owner = CacheOperationOwner::of::<AzureCliVersion>();
            let first = command.run_raw_cached(&executor, owner).await?;
            let second = command.run_raw_cached(&executor, owner).await?;
            assert!(first.success());
            assert!(Arc::ptr_eq(&first, &second));
            Ok(())
        })
        .await
}
