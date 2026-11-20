use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Register Azure CLI authentication for an existing account. Login verifies
/// that the selected user matches its established tenant-local identity.
#[derive(Debug, Facet)]
pub struct AccountCredentialAddAzureCliArgs {
    /// Account UUID or alias in the selected tenant.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    #[facet(figue::named)]
    pub tenant: AzureTenantArgument<'static>,
}
impl AccountCredentialAddAzureCliArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let tenant_id = self.tenant.resolve(cancellation).await?;
        let account_id = store
            .load()?
            .resolve(
                Some(&self.account),
                Some(&CloudTerrastodonAccountNamespace::Entra { tenant_id }),
            )?
            .id;
        Ok(CliOutput::facet(
            store
                .register_credential(
                    account_id,
                    CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                    cancellation,
                )
                .await?,
        ))
    }
}
