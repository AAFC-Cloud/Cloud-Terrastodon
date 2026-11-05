use cloud_terrastodon_azure_types::Account;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use eyre::Result;
use eyre::WrapErr;
use std::process::Stdio;
use tokio::process::Command;

/// Read the Azure CLI's local account metadata for tenant/default selection.
///
/// This deliberately inspects the user's configured CLI profile without using
/// it to authenticate service requests. It neither refreshes subscriptions nor
/// acquires tokens, starts login, or retains general execution artifacts.
/// <https://learn.microsoft.com/cli/azure/account#az-account-list>
pub async fn read_azure_cli_local_accounts(
    cancellation: &CancellationScope,
) -> Result<Vec<Account>> {
    cancellation
        .cancellable_scope(|scope| async move {
            let program = CommandKind::AzureCLI.program().await?;
            let output = Command::new(program)
                .args(["account", "list", "--output", "json", "--only-show-errors"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .output()
                .await
                .wrap_err("reading local Azure CLI account metadata")?;
            scope.bail_if_cancelled()?;
            if !output.status.success() {
                let mut output = CommandOutput::try_from(output)?;
                // Preserve failure diagnostics without attaching account metadata.
                output.stdout.clear();
                return Err(eyre::Report::from(output))
                    .wrap_err("reading local Azure CLI account metadata");
            }
            facet_json::from_slice(&output.stdout)
                .wrap_err("decoding local Azure CLI account metadata")
        })
        .await
}
