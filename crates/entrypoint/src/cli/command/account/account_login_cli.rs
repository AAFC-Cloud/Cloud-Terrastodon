use super::bmc_helix_password_prompt::BmcHelixPasswordPrompt;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::AccountSignInPrompt;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::io::IsTerminal;
use std::io::Read;
/// Provision an account mechanism. BMC Helix prompts for a password and PATs
/// are accepted through a pipe.
#[derive(Debug, Facet)]
pub struct AccountLoginArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    /// Credential mechanism to provision; omission uses the account preference.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
    /// BMC Helix server name. Omit to use the configured default server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
    #[facet(figue::named, default)]
    pub pat_stdin: bool,
    /// Use device-code sign-in for an Azure CLI registration.
    #[facet(figue::named, default)]
    pub device_code: bool,
    /// Retire the current Entra or Helix session before signing in again.
    #[facet(figue::named, default)]
    pub reauthenticate: bool,
}
impl AccountLoginArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        if self.server.is_some() || self.source == Some(CloudTerrastodonCredentialSource::BmcHelix)
        {
            return self.login_helix(&store, executor).await;
        }
        let catalogue = store.load()?;
        let account = catalogue.resolve(Some(&self.account), None)?;
        let source = self.source.unwrap_or(account.preferred_source);
        if source == CloudTerrastodonCredentialSource::BmcHelix {
            return self.login_helix(&store, executor).await;
        }
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

    async fn login_helix(
        &self,
        store: &CloudTerrastodonAccountStore,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CliOutput> {
        eyre::ensure!(
            self.source
                .is_none_or(|source| source == CloudTerrastodonCredentialSource::BmcHelix),
            "--server can only be combined with --source helix"
        );
        eyre::ensure!(!self.pat_stdin, "--pat-stdin is only for Azure DevOps PATs");
        ensure!(
            !self.device_code,
            "--device-code is only for Azure CLI sign-in"
        );
        let server = BmcHelixServerStore::configured().get(self.server.as_ref())?;
        let account = store.resolve_bmc_helix(&server.url, Some(&self.account))?;
        let active = matches!(
            store.load()?.bmc_helix_session(account.id, &server.url),
            CloudTerrastodonAccountSession::Active { .. }
        );
        ensure!(
            self.reauthenticate || !active,
            "selected account already has an active BMC Helix session on {}; sign out before signing in again",
            server.name
        );
        let password = BmcHelixPasswordPrompt::read(&server, executor.cancellation()).await?;
        if active {
            store
                .logout_bmc_helix_with_executor(account.id, &server.url, executor)
                .await?;
        }
        let session = store
            .sign_in_to_bmc_helix(account.id, &server.url, &password, executor.cancellation())
            .await?;
        Ok(CliOutput::facet(session))
    }
}
