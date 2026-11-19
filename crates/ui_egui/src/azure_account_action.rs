use crate::azure_account_registration::AzureAccountRegistration;
use cloud_terrastodon_auth::AccountSignInPrompt;
use cloud_terrastodon_auth::AzureDevOpsPersonalAccessToken;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;

/// Account work submitted from a UI form. The PAT submission owns its credential.
pub(crate) enum AzureAccountAction {
    Reload,
    Add {
        alias: CloudTerrastodonAccountAlias,
        registration: AzureAccountRegistration,
    },
    RegisterCredential {
        account_id: CloudTerrastodonAccountId,
        credential: CloudTerrastodonAccountCredential,
    },
    Prefer {
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    },
    SetDefault(CloudTerrastodonAccountId),
    ClearDefault(CloudTerrastodonAccountId),
    Login {
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    },
    SavePersonalAccessToken {
        account_id: CloudTerrastodonAccountId,
        token: AzureDevOpsPersonalAccessToken,
    },
    Reauthenticate {
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    },
    Logout {
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
    },
    Remove(CloudTerrastodonAccountId),
    ImportLegacy {
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    },
}

impl AzureAccountAction {
    pub fn sign_in(
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    ) -> Self {
        Self::Login { account_id, source }
    }

    pub fn login_target(
        &self,
    ) -> Option<(CloudTerrastodonAccountId, CloudTerrastodonCredentialSource)> {
        match *self {
            Self::Login { account_id, source }
            | Self::Reauthenticate { account_id, source }
            | Self::ImportLegacy { account_id, source } => Some((account_id, source)),
            Self::SavePersonalAccessToken { account_id, .. } => Some((
                account_id,
                CloudTerrastodonCredentialSource::PersonalAccessToken,
            )),
            _ => None,
        }
    }

    /// Apply the requested change and return the ID of a newly registered account.
    pub async fn run(
        self,
        store: &CloudTerrastodonAccountStore,
        executor: &CloudTerrastodonExecutor,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
    ) -> Result<Option<CloudTerrastodonAccountId>> {
        let cancellation = executor.cancellation();
        match self {
            Self::Reload => {}
            Self::Add {
                alias,
                registration,
            } => {
                let credential = registration.resolve(cancellation).await?;
                let account = store.add(alias, credential, cancellation).await?;
                return Ok(Some(account.id));
            }
            Self::RegisterCredential {
                account_id,
                credential,
            } => {
                store
                    .register_credential(account_id, credential, cancellation)
                    .await?;
            }
            Self::Prefer { account_id, source } => {
                store
                    .set_preferred_source(account_id, source, cancellation)
                    .await?;
            }
            Self::SetDefault(account_id) => {
                store.set_default(account_id, cancellation).await?;
            }
            Self::ClearDefault(account_id) => {
                store.clear_default(account_id, cancellation).await?;
            }
            Self::Login { account_id, source } => {
                store
                    .sign_in(account_id, Some(source), None, prompt, cancellation)
                    .await?;
            }
            Self::SavePersonalAccessToken { account_id, token } => {
                store
                    .sign_in(
                        account_id,
                        Some(CloudTerrastodonCredentialSource::PersonalAccessToken),
                        Some(token.as_ref()),
                        prompt,
                        cancellation,
                    )
                    .await?;
            }
            Self::Reauthenticate { account_id, source } => {
                store
                    .reauthenticate(account_id, source, prompt, executor)
                    .await?;
            }
            Self::Logout { account_id, source } => {
                store
                    .logout_with_executor(account_id, source, executor)
                    .await?;
            }
            Self::Remove(account_id) => {
                store.remove_with_executor(account_id, executor).await?;
            }
            Self::ImportLegacy { account_id, source } => {
                store
                    .import_legacy(account_id, Some(source), cancellation)
                    .await?;
            }
        }
        Ok(None)
    }
}
