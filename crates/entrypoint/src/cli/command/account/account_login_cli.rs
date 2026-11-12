use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::io::IsTerminal;
use std::io::Read;
/// Provision an account mechanism. Entra reauthentication preserves its verified
/// principal after explicit logout. PATs are accepted through a pipe.
#[derive(Debug, Facet)]
pub struct AccountLoginArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
    /// Credential mechanism to provision; omission uses the account preference.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
    #[facet(figue::named, default)]
    pub pat_stdin: bool,
}
impl AccountLoginArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let mut token = String::new();
        if self.pat_stdin {
            eyre::ensure!(
                !std::io::stdin().is_terminal(),
                "--pat-stdin requires piped input so credentials are not echoed"
            );
            std::io::stdin().read_to_string(&mut token)?;
        }
        let profile = CloudTerrastodonAccountStore::configured()
            .login(
                self.account_id,
                self.source,
                self.pat_stdin.then(|| token.trim_end()),
                cancellation,
            )
            .await?;
        Ok(CliOutput::facet(profile))
    }
}
