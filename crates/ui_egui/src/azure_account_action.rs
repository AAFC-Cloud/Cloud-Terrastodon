use crate::azure_account_registration::AzureAccountRegistration;
use crate::azure_account_sign_in_method::AzureAccountSignInMethod;
use crate::gui_work_cache::GuiWorkCache;
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
    Rename {
        account_id: CloudTerrastodonAccountId,
        alias: CloudTerrastodonAccountAlias,
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
        method: AzureAccountSignInMethod,
    },
    SavePersonalAccessToken {
        account_id: CloudTerrastodonAccountId,
        token: AzureDevOpsPersonalAccessToken,
    },
    Reauthenticate {
        account_id: CloudTerrastodonAccountId,
        method: AzureAccountSignInMethod,
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
        Self::Login {
            account_id,
            method: AzureAccountSignInMethod::RegisteredSource(source),
        }
    }

    pub fn login_target(
        &self,
    ) -> Option<(CloudTerrastodonAccountId, CloudTerrastodonCredentialSource)> {
        match self {
            Self::Login { account_id, method } | Self::Reauthenticate { account_id, method } => {
                Some((*account_id, method.source()))
            }
            Self::ImportLegacy { account_id, source } => Some((*account_id, *source)),
            Self::SavePersonalAccessToken { account_id, .. } => Some((
                *account_id,
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
        work_cache: &GuiWorkCache,
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
            Self::Rename { account_id, alias } => {
                store.rename(account_id, alias, cancellation).await?;
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
            Self::Login { account_id, method } => {
                method
                    .sign_in(store, account_id, prompt, cancellation)
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
            Self::Reauthenticate { account_id, method } => {
                {
                    let retirement = work_cache.job_executor(executor)?;
                    store
                        .retire_for_reauthentication(account_id, method.source(), &retirement)
                        .await?;
                }
                // The sign-in prompt can stay open independently of cache ownership.
                method
                    .sign_in(store, account_id, prompt, cancellation)
                    .await?;
            }
            Self::Logout { account_id, source } => {
                let executor = work_cache.job_executor(executor)?;
                store
                    .logout_with_executor(account_id, source, &executor)
                    .await?;
            }
            Self::Remove(account_id) => {
                let executor = work_cache.job_executor(executor)?;
                store.remove_with_executor(account_id, &executor).await?;
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
