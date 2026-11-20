use cloud_terrastodon_auth::AccountSignInPrompt;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use eyre::Result;
use teamy_cancellation::CancellationScope;

/// How to sign the user in to an existing account credential source.
pub(crate) enum AzureAccountSignInMethod {
    RegisteredSource(CloudTerrastodonCredentialSource),
    AzureCliDeviceCode,
}

impl AzureAccountSignInMethod {
    pub fn source(&self) -> CloudTerrastodonCredentialSource {
        match self {
            Self::RegisteredSource(source) => *source,
            Self::AzureCliDeviceCode => CloudTerrastodonCredentialSource::AzureCli,
        }
    }

    pub async fn sign_in(
        self,
        store: &CloudTerrastodonAccountStore,
        account_id: CloudTerrastodonAccountId,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
        cancellation: &CancellationScope,
    ) -> Result<()> {
        match self {
            Self::RegisteredSource(source) => {
                store
                    .sign_in(account_id, Some(source), None, prompt, cancellation)
                    .await?;
            }
            Self::AzureCliDeviceCode => {
                store
                    .sign_in_with_azure_cli_device_code(account_id, prompt, cancellation)
                    .await?;
            }
        }
        Ok(())
    }
}
