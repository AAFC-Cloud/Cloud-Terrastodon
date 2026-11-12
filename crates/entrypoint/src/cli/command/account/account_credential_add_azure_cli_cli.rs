use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Register Azure CLI authentication for an existing account. Login verifies
/// that the selected user matches its established tenant-local identity.
#[derive(Debug, Facet)]
pub struct AccountCredentialAddAzureCliArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
    #[facet(figue::named)]
    pub tenant: AzureTenantArgument<'static>,
}
impl AccountCredentialAddAzureCliArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            CloudTerrastodonAccountStore::configured()
                .register_credential(
                    self.account_id,
                    CloudTerrastodonAccountCredential::AzureCli {
                        tenant_id: self.tenant.resolve(cancellation).await?,
                    },
                    cancellation,
                )
                .await?,
        ))
    }
}
