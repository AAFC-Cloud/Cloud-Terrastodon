use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AccountSignInPrompt;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::io::IsTerminal;
use std::io::Read;
/// Provision an account mechanism. Entra reauthentication preserves its verified
/// principal after explicit logout. PATs are accepted through a pipe.
#[derive(Debug, Facet)]
pub struct AccountLoginArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    /// Credential mechanism to provision; omission uses the account preference.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
    #[facet(figue::named, default)]
    pub pat_stdin: bool,
    /// Use device-code sign-in for an Azure CLI registration.
    #[facet(figue::named, default)]
    pub device_code: bool,
    /// Retire the current Entra session before signing in again.
    #[facet(figue::named, default)]
    pub reauthenticate: bool,
}
impl AccountLoginArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let catalogue = store.load()?;
        let account = catalogue.resolve(Some(&self.account), None)?;
        let source = self.source.unwrap_or(account.preferred_source);
        if self.device_code {
            ensure!(
                source == CloudTerrastodonCredentialSource::AzureCli && !self.pat_stdin,
                "--device-code requires an Azure CLI registration and cannot be combined with --pat-stdin"
            );
        }
        if self.reauthenticate {
            ensure!(
                !self.pat_stdin,
                "--reauthenticate cannot be combined with --pat-stdin"
            );
            store
                .retire_for_reauthentication(account.id, source, executor)
                .await?;
        }
        if self.device_code {
            let profile = store
                .sign_in_with_azure_cli_device_code(
                    account.id,
                    &AccountSignInPrompt::print,
                    executor.cancellation(),
                )
                .await?;
            return Ok(CliOutput::facet(profile));
        }
        let mut token = String::new();
        if self.pat_stdin {
            eyre::ensure!(
                !std::io::stdin().is_terminal(),
                "--pat-stdin requires piped input so credentials are not echoed"
            );
            std::io::stdin().read_to_string(&mut token)?;
        }
        let profile = store
            .login(
                account.id,
                Some(source),
                self.pat_stdin.then(|| token.trim_end()),
                executor.cancellation(),
            )
            .await?;
        Ok(CliOutput::facet(profile))
    }
}
