use crate::AccountSignInPrompt;
use crate::AuthContext;
use crate::AzureCliSessions;
use crate::BmcHelixAccountSession;
use crate::BrowserSession;
use crate::CloudTerrastodonAccountCatalogue;
use crate::CloudTerrastodonAccountDefault;
use crate::CredentialAuthenticationProvider;
use crate::ResolvedAuthContext;
use crate::WorkloadIdentityConfig;
use crate::account_credential_provisioning::AccountCredentialProvisioning;
use crate::auth_logout_error::AuthLogoutError;
use crate::azure_cli_session::AzureCliSession;
use crate::azure_cli_session_store::AzureCliSessionStore;
use crate::azure_cli_sign_in_mode::AzureCliSignInMode;
use crate::cloud_terrastodon_account_secret_store::CloudTerrastodonAccountSecretStore;
use crate::entra_authenticated_user_identity::EntraAuthenticatedUserIdentity;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::BmcHelixAuthorization;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountPrincipal;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialRegistration;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::GiteaCommandSession;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use eyre::ensure;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Owns explicit account registrations, defaults, and isolated credential storage.
///
/// Reading this catalogue never imports ambient Azure CLI, browser, or PAT state.
/// Runtime providers capture selected credentials once and do not consult this
/// store again to resolve request aliases or substitute another account.
#[derive(Clone, Debug)]
pub struct CloudTerrastodonAccountStore {
    root: PathBuf,
}

impl CloudTerrastodonAccountStore {
    pub fn configured() -> Self {
        Self {
            root: AppDir::Config.join("accounts"),
        }
    }

    /// Explicit storage ownership for embedding and isolated tests.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        ensure!(root.is_absolute(), "account store root must be absolute");
        Ok(Self { root })
    }

    pub fn load(&self) -> Result<CloudTerrastodonAccountCatalogue> {
        let content = match std::fs::read(self.root.join("accounts.json")) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(CloudTerrastodonAccountCatalogue::default());
            }
            Err(error) => return Err(error).wrap_err("reading the account catalogue"),
        };
        let mut catalogue: CloudTerrastodonAccountCatalogue =
            facet_json::from_slice(&content).wrap_err(
                "decoding the account catalogue; pre-cutover single-credential records require explicit migration or new verified registrations; existing account IDs and credential files have not been changed",
            )?;
        catalogue.migrate_bmc_helix_sessions();
        catalogue.validate()?;
        Ok(catalogue)
    }

    fn prepare_directory(path: &Path) -> Result<()> {
        std::fs::create_dir_all(path).wrap_err("creating the account storage directory")?;
        #[cfg(unix)]
        std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o700))?;
        Ok(())
    }

    async fn lock(&self, cancellation: &CancellationScope) -> Result<std::fs::File> {
        Self::prepare_directory(&self.root)?;
        AzureCliSession::lock(&self.root.join("accounts.lock"), cancellation)
            .await
            .wrap_err("acquiring the account catalogue mutation lease")
    }

    fn save(&self, catalogue: &CloudTerrastodonAccountCatalogue) -> Result<()> {
        catalogue.validate()?;
        Self::prepare_directory(&self.root)?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root)?;
        temporary.write_all(&facet_json::to_vec(catalogue)?)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(self.root.join("accounts.json"))
            .map_err(|error| error.error)
            .wrap_err("publishing the account catalogue")?;
        Ok(())
    }

    pub async fn add(
        &self,
        alias: CloudTerrastodonAccountAlias,
        credential: CloudTerrastodonAccountCredential,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        ensure!(
            !catalogue.accounts.iter().any(
                |account| account.alias == alias && account.namespace == credential.namespace()
            ),
            "account alias already exists in this tenant or service"
        );
        if let CloudTerrastodonAccountCredential::Gitea { login, .. } = &credential {
            ensure!(
                !login.trim().is_empty() && !login.chars().any(char::is_control),
                "tea login profile must be nonempty and contain no control characters"
            );
        }
        let profile = CloudTerrastodonAccountProfile::new(
            CloudTerrastodonAccountId::new(Uuid::new_v4()),
            alias,
            credential,
        );
        catalogue.accounts.push(profile.clone());
        cancellation.with_deferral("registering an account", |_| self.save(&catalogue))?;
        Ok(profile)
    }

    /// Change an account's alias within its existing namespace.
    pub async fn rename(
        &self,
        account_id: CloudTerrastodonAccountId,
        alias: CloudTerrastodonAccountAlias,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let Some(index) = catalogue
            .accounts
            .iter()
            .position(|account| account.id == account_id)
        else {
            bail!("account {account_id} is not registered");
        };
        let account = &catalogue.accounts[index];
        if account.alias == alias {
            return Ok(account.clone());
        }
        ensure!(
            !catalogue
                .accounts
                .iter()
                .any(|other| { other.namespace == account.namespace && other.alias == alias }),
            "account alias {alias:?} already exists in this tenant or service",
            alias = alias.as_str(),
        );

        catalogue.accounts[index].alias = alias;
        cancellation.with_deferral("renaming an account", |_| self.save(&catalogue))?;
        Ok(catalogue.accounts.swap_remove(index))
    }

    pub async fn set_default(
        &self,
        account_id: CloudTerrastodonAccountId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let namespace = catalogue.by_id(account_id)?.namespace.clone();
        ensure!(
            !matches!(namespace, CloudTerrastodonAccountNamespace::BmcHelix { .. }),
            "specify a server when setting a Helix account default"
        );
        catalogue
            .defaults
            .retain(|value| value.namespace != namespace);
        catalogue.defaults.push(CloudTerrastodonAccountDefault {
            namespace,
            account_id,
        });
        cancellation.with_deferral("updating an account default", |_| self.save(&catalogue))
    }

    pub async fn clear_default(
        &self,
        account_id: CloudTerrastodonAccountId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        ensure!(
            !matches!(
                catalogue.by_id(account_id)?.namespace,
                CloudTerrastodonAccountNamespace::BmcHelix { .. }
            ),
            "specify a server when clearing a Helix account default"
        );
        catalogue
            .defaults
            .retain(|value| value.account_id != account_id);
        cancellation.with_deferral("clearing an account default", |_| self.save(&catalogue))
    }

    pub async fn set_bmc_helix_default(
        &self,
        server: &BmcHelixServerUrl,
        account_id: CloudTerrastodonAccountId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        catalogue.bmc_helix_account(account_id)?;
        let namespace = CloudTerrastodonAccountNamespace::BmcHelix {
            server: Some(server.clone()),
        };
        catalogue
            .defaults
            .retain(|value| value.namespace != namespace);
        catalogue.defaults.push(CloudTerrastodonAccountDefault {
            namespace,
            account_id,
        });
        cancellation.with_deferral("updating a Helix server's account default", |_| {
            self.save(&catalogue)
        })
    }

    pub async fn clear_bmc_helix_default(
        &self,
        server: &BmcHelixServerUrl,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let namespace = CloudTerrastodonAccountNamespace::BmcHelix {
            server: Some(server.clone()),
        };
        catalogue
            .defaults
            .retain(|value| value.namespace != namespace);
        cancellation.with_deferral("clearing a Helix server's account default", |_| {
            self.save(&catalogue)
        })
    }

    fn credential_store(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        source: CloudTerrastodonCredentialSource,
        generation: Uuid,
    ) -> CloudTerrastodonAccountSecretStore {
        CloudTerrastodonAccountSecretStore {
            directory: self
                .root
                .join("credentials")
                .join(profile.id.to_string())
                .join(source.to_string())
                .join(generation.to_string()),
            account_id: profile.id,
            generation,
        }
    }

    async fn cleanup_generation(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        source: CloudTerrastodonCredentialSource,
        generation: Uuid,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let store = self.credential_store(profile, source, generation);
        Self::mark_credential_retired(&store)?;
        match &profile.registration(source)?.credential {
            CloudTerrastodonAccountCredential::AzureCli { .. } => {
                AzureCliSessionStore::at(store.directory.join("azure-cli"))
                    .logout_all(cancellation)
                    .await
            }
            CloudTerrastodonAccountCredential::WorkloadIdentity { .. } => Ok(()),
            _ => store.delete(),
        }
    }

    fn mark_credential_retired(store: &CloudTerrastodonAccountSecretStore) -> Result<()> {
        Self::prepare_directory(&store.directory)?;
        std::fs::write(
            store.directory.join("retired"),
            b"explicit account retirement",
        )
        .wrap_err("marking the account credential retired")
    }

    async fn cleanup_bmc_helix_session(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        server: &BmcHelixServerUrl,
        generation: Uuid,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let store = self.credential_store(
            profile,
            CloudTerrastodonCredentialSource::BmcHelix,
            generation,
        );
        Self::mark_credential_retired(&store)?;
        let revocation = match store.read_if_present() {
            Ok(Some(token)) => crate::bmc_helix_session::logout(server, &token, cancellation).await,
            Ok(None) => Ok(()),
            Err(error) => Err(error),
        };
        if let Err(error) = revocation {
            // The catalogue has retired this session. Clear the local token
            // even when the server is unavailable.
            tracing::warn!(%error, "Could not revoke the Helix token on the server");
        }
        store.delete()
    }

    /// Attach another mechanism without changing the stable account identity.
    /// Activation still requires login and verified principal continuity.
    pub async fn register_credential(
        &self,
        account_id: CloudTerrastodonAccountId,
        credential: CloudTerrastodonAccountCredential,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let profile = catalogue
            .accounts
            .iter_mut()
            .find(|profile| profile.id == account_id)
            .ok_or_else(|| eyre::eyre!("account {account_id} is not registered"))?;
        ensure!(
            profile.namespace == credential.namespace(),
            "credential belongs to a different tenant or service"
        );
        ensure!(
            profile.registration(credential.source()).is_err(),
            "this credential source is already registered"
        );
        profile
            .credentials
            .push(CloudTerrastodonCredentialRegistration {
                credential,
                session: CloudTerrastodonAccountSession::Unprovisioned,
            });
        profile.validate()?;
        let profile = profile.clone();
        cancellation.with_deferral("registering an account credential", |_| {
            self.save(&catalogue)
        })?;
        Ok(profile)
    }

    pub async fn set_preferred_source(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let profile = catalogue
            .accounts
            .iter_mut()
            .find(|profile| profile.id == account_id)
            .ok_or_else(|| eyre::eyre!("account {account_id} is not registered"))?;
        profile.registration(source)?;
        profile.preferred_source = source;
        let profile = profile.clone();
        cancellation.with_deferral("changing an account's preferred credential source", |_| {
            self.save(&catalogue)
        })?;
        Ok(profile)
    }

    /// Provision the selected mechanism. Existing active sessions require an
    /// explicit logout first; a replacement Entra session must prove the same
    /// tenant-local principal before publication. PAT/tea replacements still
    /// require a new account because those flows have no identity proof here.
    pub async fn login(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        personal_access_token: Option<&str>,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        self.sign_in(
            account_id,
            source,
            personal_access_token,
            &AccountSignInPrompt::print,
            cancellation,
        )
        .await
    }

    /// Sign in using the application's interactive prompt renderer.
    ///
    /// Browser sign-in opens the system browser. The callback receives a URL
    /// when browser launch fails or a code when Azure CLI uses device sign-in.
    /// Prompts must be scoped to the invoking application and kept out of logs.
    pub async fn sign_in(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        personal_access_token: Option<&str>,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        self.provision(
            account_id,
            source,
            AccountCredentialProvisioning::SignIn {
                personal_access_token,
                prompt,
                azure_cli_mode: AzureCliSignInMode::Automatic,
            },
            cancellation,
        )
        .await
    }

    /// Sign in to a registered Azure CLI credential using an explicit device code.
    ///
    /// The application renders the code from `prompt` for this sign-in. The
    /// resulting session must still match the registered tenant and identity.
    /// See [`az login --use-device-code`](https://learn.microsoft.com/cli/azure/reference-index?view=azure-cli-latest#az-login).
    pub async fn sign_in_with_azure_cli_device_code(
        &self,
        account_id: CloudTerrastodonAccountId,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        self.provision(
            account_id,
            Some(CloudTerrastodonCredentialSource::AzureCli),
            AccountCredentialProvisioning::SignIn {
                personal_access_token: None,
                prompt,
                azure_cli_mode: AzureCliSignInMode::DeviceCode,
            },
            cancellation,
        )
        .await
    }

    /// Exchange a password for a server-bound [AR System JWT](https://docs.helixops.ai/bin/Service-Management/Innovation-Suite/AR-System/ars213/Developing-an-API-program/Integrating-AR-System-forms-with-a-third-party-application-by-using-the-REST-API/Access-and-authentication-for-the-REST-API/).
    /// Only the issued token is persisted in the account secret store.
    pub async fn sign_in_to_bmc_helix(
        &self,
        account_id: CloudTerrastodonAccountId,
        server: &BmcHelixServerUrl,
        password: &str,
        cancellation: &CancellationScope,
    ) -> Result<BmcHelixAccountSession> {
        ensure!(
            !password.is_empty(),
            "Helix login requires a nonempty password"
        );
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let profile = catalogue.bmc_helix_account(account_id)?.clone();
        let source = CloudTerrastodonCredentialSource::BmcHelix;
        let CloudTerrastodonAccountCredential::BmcHelix { username, .. } =
            &profile.registration(source)?.credential
        else {
            bail!("the selected Helix account has no username");
        };
        let previous = catalogue.bmc_helix_session(account_id, server);
        ensure!(
            !matches!(previous, CloudTerrastodonAccountSession::Active { .. }),
            "the Helix account is already signed in on this server; sign out here before signing in again"
        );
        if let CloudTerrastodonAccountSession::Provisioning { generation }
        | CloudTerrastodonAccountSession::Retired { generation } = previous
        {
            cancellation
                .finish_required("cleaning the prior Helix server session", async |scope| {
                    self.cleanup_bmc_helix_session(&profile, server, generation, &scope)
                        .await
                })
                .await?;
        }

        cancellation
            .deferred(
                "issuing and saving the Helix server session",
                async |scope| {
                    let generation = Uuid::new_v4();
                    let store = self.credential_store(&profile, source, generation);
                    Self::prepare_directory(&store.directory)?;
                    catalogue.bmc_helix_sessions.retain(|record| {
                        record.account_id != account_id || record.server != *server
                    });
                    let index = catalogue.bmc_helix_sessions.len();
                    catalogue.bmc_helix_sessions.push(BmcHelixAccountSession {
                        account_id,
                        server: server.clone(),
                        session: CloudTerrastodonAccountSession::Provisioning { generation },
                    });
                    self.save(&catalogue)?;

                    let provision = async {
                        let token =
                            crate::bmc_helix_session::login(server, username, password, &scope)
                                .await?;
                        if let Err(error) = store.write(&token) {
                            if let Err(cleanup) =
                                crate::bmc_helix_session::logout(server, &token, &scope).await
                            {
                                return Err(error.wrap_err(format!(
                                    "could not revoke the unsaved Helix credential: {cleanup:#}"
                                )));
                            }
                            return Err(error);
                        }
                        catalogue.bmc_helix_sessions[index].session =
                            CloudTerrastodonAccountSession::Active { generation };
                        self.save(&catalogue)
                    }
                    .await;
                    if let Err(error) = provision {
                        let cleanup = async {
                            self.cleanup_bmc_helix_session(&profile, server, generation, &scope)
                                .await?;
                            catalogue.bmc_helix_sessions.remove(index);
                            self.save(&catalogue)
                        }
                        .await;
                        return match cleanup {
                            Ok(()) => Err(error),
                            Err(cleanup) => Err(error.wrap_err(format!(
                                "Helix sign-in cleanup also failed: {cleanup:#}"
                            ))),
                        };
                    }
                    Ok(catalogue.bmc_helix_sessions[index].clone())
                },
            )
            .await
    }

    /// Retire the selected Entra session, then sign in and verify that its
    /// replacement belongs to the same tenant-local principal.
    ///
    /// If sign-in fails after retirement, this credential remains signed out.
    /// Install a fresh provider for later work after a session change.
    /// With a job-scoped cache, use [`Self::retire_for_reauthentication`] and
    /// release the executor before waiting for sign-in.
    pub async fn reauthenticate(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
        prompt: &(dyn Fn(AccountSignInPrompt) + Send + Sync),
        executor: &CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonAccountProfile> {
        self.retire_for_reauthentication(account_id, source, executor)
            .await?;
        self.sign_in(
            account_id,
            Some(source),
            None,
            prompt,
            executor.cancellation(),
        )
        .await
    }

    /// Finish retiring an active Entra session before interactive sign-in.
    ///
    /// The executor owns cache invalidation and drains admitted work. Callers
    /// with a job-scoped cache can release that executor before calling
    /// [`Self::sign_in`] with the same account and source. The catalogue
    /// preserves the verified principal so sign-in can check the replacement.
    pub async fn retire_for_reauthentication(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let previous = {
            let catalogue = self.load()?;
            let profile = catalogue.by_id(account_id)?;
            ensure!(
                matches!(
                    profile.namespace,
                    CloudTerrastodonAccountNamespace::Entra { .. }
                ),
                "reauthentication requires a registered Entra credential"
            );
            profile.registration(source)?.session
        };
        if let CloudTerrastodonAccountSession::Active { generation } = previous {
            executor
                .cancellation()
                .deferred(
                    "retiring the Entra session before signing in again",
                    async |scope| {
                        let executor = executor.with_cancellation_scope(scope);
                        // Drain admitted work before taking the catalogue lock.
                        // Recheck the observed generation under that lock so we
                        // cannot retire a newer session published by another owner.
                        executor.retire_account(account_id, Some(source)).await?;
                        self.logout_generation(
                            account_id,
                            Some(source),
                            Some(generation),
                            None,
                            executor.cancellation(),
                        )
                        .await
                    },
                )
                .await?;
            // Ownership of the old session has ended. A new interactive
            // operation can now be skipped if its caller was cancelled.
            executor.cancellation().bail_if_cancelled()?;
        }
        Ok(())
    }

    /// Explicitly adopt application-owned legacy credentials. Never imports an
    /// ambient Azure CLI login; Entra identities are verified before activation.
    pub async fn import_legacy(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        self.provision(
            account_id,
            source,
            AccountCredentialProvisioning::ImportLegacy,
            cancellation,
        )
        .await
    }

    async fn provision(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        provisioning: AccountCredentialProvisioning<'_>,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let mut profile = catalogue.by_id(account_id)?.clone();
        let source = source.unwrap_or(profile.preferred_source);
        ensure!(
            source != CloudTerrastodonCredentialSource::BmcHelix,
            "Helix login requires a server and an interactive password"
        );
        let previous = profile.registration(source)?.session;
        let personal_access_token = match &provisioning {
            AccountCredentialProvisioning::SignIn {
                personal_access_token,
                ..
            } => *personal_access_token,
            AccountCredentialProvisioning::ImportLegacy => None,
        };
        ensure!(
            personal_access_token.is_none()
                || source == CloudTerrastodonCredentialSource::PersonalAccessToken,
            "only PAT accounts accept a personal access token"
        );
        if source == CloudTerrastodonCredentialSource::PersonalAccessToken
            && !matches!(provisioning, AccountCredentialProvisioning::ImportLegacy)
        {
            ensure!(
                personal_access_token.is_some_and(|token| !token.trim().is_empty()),
                "PAT login requires a nonempty credential"
            );
        }
        ensure!(
            !matches!(previous, CloudTerrastodonAccountSession::Active { .. }),
            "the selected credential is already active; explicitly log out before replacement login"
        );
        if matches!(previous, CloudTerrastodonAccountSession::Retired { .. }) {
            ensure!(
                matches!(
                    profile.namespace,
                    CloudTerrastodonAccountNamespace::Entra { .. }
                ),
                "replacement PAT or tea credentials require a new account registration"
            );
        }
        if let CloudTerrastodonAccountSession::Provisioning { generation }
        | CloudTerrastodonAccountSession::Retired { generation } = previous
        {
            cancellation
                .finish_required(
                    "cleaning interrupted or retired account credentials",
                    async |protected| {
                        self.cleanup_generation(&profile, source, generation, &protected)
                            .await
                    },
                )
                .await?;
        }
        let generation = Uuid::new_v4();
        let store = self.credential_store(&profile, source, generation);
        Self::prepare_directory(&store.directory)?;
        profile.registration_mut(source)?.session =
            CloudTerrastodonAccountSession::Provisioning { generation };
        *catalogue
            .accounts
            .iter_mut()
            .find(|entry| entry.id == account_id)
            .ok_or_else(|| eyre::eyre!("account disappeared during provisioning"))? =
            profile.clone();
        cancellation.with_deferral("recording account provisioning", |_| self.save(&catalogue))?;
        let result = self
            .acquire_credential(&profile, source, &store, provisioning, cancellation)
            .await;
        let result = result.and_then(|principal| {
            if let Some(principal) = principal {
                profile.verify_principal(principal)?;
            }
            Ok(())
        });
        if let Err(error) = result {
            // Even a principal mismatch must leave no usable candidate session.
            let cleanup = cancellation
                .finish_required(
                    "retiring failed credential provisioning",
                    async |protected| {
                        // This candidate has never been admitted. Keep its provisioning
                        // generation until cleanup succeeds so interrupted cleanup can
                        // be retried, including for a first PAT/tea login.
                        profile.registration_mut(source)?.session =
                            CloudTerrastodonAccountSession::Provisioning { generation };
                        *catalogue
                            .accounts
                            .iter_mut()
                            .find(|entry| entry.id == account_id)
                            .ok_or_else(|| {
                                eyre::eyre!("account disappeared during provisioning cleanup")
                            })? = profile.clone();
                        self.save(&catalogue)?;
                        self.cleanup_generation(&profile, source, generation, &protected)
                            .await?;
                        profile.registration_mut(source)?.session =
                            CloudTerrastodonAccountSession::Unprovisioned;
                        *catalogue
                            .accounts
                            .iter_mut()
                            .find(|entry| entry.id == account_id)
                            .ok_or_else(|| {
                                eyre::eyre!("account disappeared after provisioning cleanup")
                            })? = profile.clone();
                        self.save(&catalogue)
                    },
                )
                .await;
            return match cleanup {
                Ok(()) => Err(error),
                Err(cleanup_error) => Err(error).wrap_err(format!(
                    "candidate credential was not activated; cleanup also failed: {cleanup_error}"
                )),
            };
        }
        profile.registration_mut(source)?.session =
            CloudTerrastodonAccountSession::Active { generation };
        profile.validate()?;
        *catalogue
            .accounts
            .iter_mut()
            .find(|entry| entry.id == account_id)
            .ok_or_else(|| eyre::eyre!("account disappeared during provisioning"))? =
            profile.clone();
        cancellation.finish_required_sync("publishing the verified account session", |_| {
            self.save(&catalogue)
        })?;
        Ok(profile)
    }

    async fn acquire_credential(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        source: CloudTerrastodonCredentialSource,
        store: &CloudTerrastodonAccountSecretStore,
        provisioning: AccountCredentialProvisioning<'_>,
        cancellation: &CancellationScope,
    ) -> Result<Option<CloudTerrastodonAccountPrincipal>> {
        let (personal_access_token, sign_in) = match provisioning {
            AccountCredentialProvisioning::SignIn {
                personal_access_token,
                prompt,
                azure_cli_mode,
            } => (personal_access_token, Some((prompt, azure_cli_mode))),
            AccountCredentialProvisioning::ImportLegacy => (None, None),
        };
        let import = sign_in.is_none();
        match &profile.registration(source)?.credential {
            CloudTerrastodonAccountCredential::AzureCli { tenant_id } => {
                let sessions = AzureCliSessionStore::at(store.directory.join("azure-cli"));
                let session = if let Some((prompt, azure_cli_mode)) = sign_in {
                    sessions
                        .login(*tenant_id, azure_cli_mode, prompt, cancellation)
                        .await?
                } else {
                    sessions.import_legacy(*tenant_id, cancellation).await?;
                    sessions.load(*tenant_id, cancellation).await?
                };
                let token = session
                    .fetch_token(AzureCredentialResource::MicrosoftGraph, cancellation)
                    .await?;
                Ok(Some(CloudTerrastodonAccountPrincipal::EntraUser(
                    EntraAuthenticatedUserIdentity::verify(&token.access_token, cancellation)
                        .await?,
                )))
            }
            CloudTerrastodonAccountCredential::Browser {
                tenant_id,
                client_id,
            } => {
                let (session, principal) = if let Some((prompt, _)) = sign_in {
                    crate::browser_access_token::acquire_browser_session(
                        *tenant_id,
                        *client_id,
                        prompt,
                        cancellation,
                    )
                    .await?
                } else {
                    let session = crate::browser_access_token::load_browser_session()?
                        .ok_or_else(|| eyre::eyre!("no legacy CT browser credential exists"))?;
                    ensure!(
                        session.tenant_id == *tenant_id && session.client_id == *client_id,
                        "legacy browser credential belongs to a different tenant or application"
                    );
                    let mut cache = crate::BrowserTokenCache::from_session(Some(&session));
                    cache.credential_store = Some(store.clone());
                    let token = crate::browser_access_token::fetch_browser_access_token(
                        &cache,
                        *tenant_id,
                        *client_id,
                        AzureCredentialResource::MicrosoftGraph,
                        false,
                        cancellation,
                    )
                    .await?;
                    let principal =
                        EntraAuthenticatedUserIdentity::verify(&token.access_token, cancellation)
                            .await?;
                    // Refresh may rotate the credential. Retain the successor
                    // which the token cache published instead of the old import.
                    // Decoder excerpts can contain the refresh credential;
                    // intentionally exclude that source from diagnostics.
                    let session = facet_json::from_str(&store.read()?).map_err(|_| {
                        eyre::eyre!("decoding refreshed imported credentials failed")
                    })?;
                    (session, principal)
                };
                // Keep serialization failures from exposing session credentials.
                let encoded = facet_json::to_string(&session).map_err(|_| {
                    eyre::eyre!("serializing the browser account credential failed")
                })?;
                cancellation.with_deferral("persisting the account browser credential", |_| {
                    store.write(&encoded)
                })?;
                Ok(Some(CloudTerrastodonAccountPrincipal::EntraUser(principal)))
            }
            CloudTerrastodonAccountCredential::WorkloadIdentity {
                tenant_id,
                client_id,
                assertion_file,
            } => {
                ensure!(
                    !import,
                    "workload identity uses explicit acquisition configuration; use account login"
                );
                let config = WorkloadIdentityConfig {
                    tenant_id: *tenant_id,
                    client_id: *client_id,
                    federated_token: std::fs::read_to_string(assertion_file)
                        .wrap_err("reading the selected workload assertion")?,
                };
                // Successful tenant-specific client-credentials authentication
                // proves the configured application. No JWT claims are trusted.
                crate::workload_identity::exchange_workload_identity_assertion(
                    &reqwest::Client::new(),
                    config.token_endpoint()?,
                    &config,
                    AzureCredentialResource::AzureResourceManager,
                    cancellation,
                )
                .await?;
                Ok(Some(CloudTerrastodonAccountPrincipal::EntraApplication(
                    *client_id,
                )))
            }
            CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken { .. } => {
                let token = if import {
                    crate::azure_devops_pat::read_legacy_azure_devops_personal_access_token()
                        .await?
                        .as_str()
                        .to_owned()
                } else {
                    personal_access_token
                        .ok_or_else(|| eyre::eyre!("PAT login requires a credential"))?
                        .to_owned()
                };
                ensure!(
                    !token.trim().is_empty(),
                    "personal access token must not be empty"
                );
                cancellation
                    .with_deferral("persisting the account PAT", |_| store.write(&token))?;
                Ok(None)
            }
            CloudTerrastodonAccountCredential::Gitea {
                instance,
                login,
                configuration,
            } => {
                ensure!(
                    !import,
                    "tea credentials use explicit configuration; use account login"
                );
                let content = std::fs::read_to_string(configuration)
                    .wrap_err("reading the explicitly selected tea credential configuration")?;
                let session = GiteaCommandSession::new(login.clone(), &content)?;
                let output = CommandBuilder::new(CommandKind::Gitea)
                    .cancellation_safe()
                    .artifact_retention(ArtifactRetention::Disabled)
                    .env(
                        "XDG_CONFIG_HOME",
                        session
                            .configuration_home()
                            .to_str()
                            .ok_or_else(|| eyre::eyre!("tea configuration path is not Unicode"))?,
                    )
                    .env_remove("TEA_CONFIG")
                    .env_remove_prefix("GITEA_")
                    .args(["logins", "list", "--output", "json"])
                    .run_raw(cancellation)
                    .await?;
                // Decoding diagnostics may quote credential configuration or
                // account details from the isolated tea session.
                let logins: Vec<crate::gitea_account_login_metadata::GiteaAccountLoginMetadata> =
                    facet_json::from_slice(&output.stdout).map_err(|_| {
                        eyre::eyre!(
                            "tea could not describe the selected isolated credential configuration"
                        )
                    })?;
                ensure!(
                    logins.len() == 1 && logins[0].name == *login && logins[0].url == *instance,
                    "the supplied tea configuration must contain exactly one login matching the selected name and instance"
                );
                cancellation.with_deferral("persisting the isolated tea credential", |_| {
                    store.write(&content)
                })?;
                Ok(None)
            }
            CloudTerrastodonAccountCredential::BmcHelix { .. } => {
                bail!("Helix login requires a server and an interactive password");
            }
        }
    }

    /// Drain the executor's admitted work before removing the selected credentials.
    pub async fn logout_with_executor(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        executor
            .cancellation()
            .deferred("retiring and removing account credentials", async |scope| {
                let executor = executor.with_cancellation_scope(scope);
                executor.retire_account(account_id, source).await?;
                self.logout_source(account_id, source, executor.cancellation())
                    .await
            })
            .await
    }

    /// Sign out the accounts currently registered, retaining their registrations
    /// and configured defaults. Let admitted retirement finish before stopping
    /// on cancellation and attempt the remaining accounts after a cleanup error.
    pub async fn logout_all_with_executor(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<CloudTerrastodonAccountId>> {
        let account_ids = self
            .load()?
            .accounts
            .into_iter()
            .map(|account| account.id)
            .collect::<Vec<_>>();
        let mut errors = Vec::new();
        for &id in &account_ids {
            executor.cancellation().bail_if_cancelled()?;
            if let Err(error) = self
                .logout_with_executor(id, None, executor)
                .await
                .wrap_err_with(|| format!("signing out account {id}"))
            {
                errors.push(error);
            }
        }
        if !errors.is_empty() {
            return Err(AuthLogoutError { errors }.into());
        }
        Ok(account_ids)
    }

    /// Drain the executor's admitted work before deleting the account registration.
    pub async fn remove_with_executor(
        &self,
        account_id: CloudTerrastodonAccountId,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        executor
            .cancellation()
            .deferred("retiring and removing an account", async |scope| {
                let executor = executor.with_cancellation_scope(scope);
                executor.retire_account(account_id, None).await?;
                self.remove(account_id, executor.cancellation()).await
            })
            .await
    }

    /// Retire every source before cleanup. One damaged credential must not leave
    /// another source active, and every cleanup failure is retained for retry.
    pub async fn logout(
        &self,
        account_id: CloudTerrastodonAccountId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.logout_source(account_id, None, cancellation).await
    }

    pub async fn logout_source(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.logout_generation(account_id, source, None, None, cancellation)
            .await
    }

    /// Retire only this server's JWT and leave other sessions usable.
    pub async fn logout_bmc_helix_with_executor(
        &self,
        account_id: CloudTerrastodonAccountId,
        server: &BmcHelixServerUrl,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        self.load()?.bmc_helix_account(account_id)?;
        executor
            .cancellation()
            .deferred("retiring a Helix server session", async |scope| {
                let executor = executor.with_cancellation_scope(scope);
                executor
                    .retire_bmc_helix_session(account_id, server.id())
                    .await?;
                self.logout_generation(
                    account_id,
                    Some(CloudTerrastodonCredentialSource::BmcHelix),
                    None,
                    Some(server),
                    executor.cancellation(),
                )
                .await
            })
            .await
    }

    async fn logout_generation(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        expected_generation: Option<Uuid>,
        helix_server: Option<&BmcHelixServerUrl>,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let mut profile = catalogue.by_id(account_id)?.clone();
        if let Some(source) = source {
            let session = profile.registration(source)?.session;
            if let Some(expected_generation) = expected_generation {
                ensure!(
                    matches!(
                        session,
                        CloudTerrastodonAccountSession::Active { generation }
                            | CloudTerrastodonAccountSession::Retired { generation }
                            if generation == expected_generation
                    ),
                    "the account session changed during reauthentication; reload and retry"
                );
            }
        }
        cancellation
            .deferred("retiring account credential sessions", async |protected| {
                let mut cleanup = Vec::new();
                let mut helix_cleanup = Vec::new();
                for slot in &mut profile.credentials {
                    if source.is_some_and(|source| source != slot.credential.source()) {
                        continue;
                    }
                    if let CloudTerrastodonAccountSession::Active { generation }
                    | CloudTerrastodonAccountSession::Provisioning { generation }
                    | CloudTerrastodonAccountSession::Retired { generation } = slot.session
                    {
                        slot.session = CloudTerrastodonAccountSession::Retired { generation };
                        cleanup.push((slot.credential.source(), generation));
                    }
                }
                if source.is_none_or(|source| source == CloudTerrastodonCredentialSource::BmcHelix)
                {
                    for record in &mut catalogue.bmc_helix_sessions {
                        if record.account_id == account_id
                            && helix_server.is_none_or(|server| record.server == *server)
                            && let CloudTerrastodonAccountSession::Active { generation }
                            | CloudTerrastodonAccountSession::Provisioning { generation }
                            | CloudTerrastodonAccountSession::Retired { generation } =
                                record.session
                        {
                            record.session = CloudTerrastodonAccountSession::Retired { generation };
                            helix_cleanup.push((record.server.clone(), generation));
                        }
                    }
                }
                *catalogue
                    .accounts
                    .iter_mut()
                    .find(|entry| entry.id == account_id)
                    .ok_or_else(|| eyre::eyre!("account disappeared during logout"))? =
                    profile.clone();
                self.save(&catalogue)?;
                let mut errors = Vec::new();
                for (source, generation) in cleanup {
                    if let Err(error) = self
                        .cleanup_generation(&profile, source, generation, &protected)
                        .await
                    {
                        errors.push(error);
                    }
                }
                for (server, generation) in helix_cleanup {
                    if let Err(error) = self
                        .cleanup_bmc_helix_session(&profile, &server, generation, &protected)
                        .await
                    {
                        errors.push(error);
                    }
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(crate::auth_logout_error::AuthLogoutError { errors }.into())
                }
            })
            .await
    }
    pub async fn remove(
        &self,
        account_id: CloudTerrastodonAccountId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.logout(account_id, cancellation).await?;
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        ensure!(
            catalogue
                .by_id(account_id)?
                .credentials
                .iter()
                .all(|slot| matches!(
                    slot.session,
                    CloudTerrastodonAccountSession::Unprovisioned
                        | CloudTerrastodonAccountSession::Retired { .. }
                )),
            "account acquired another credential session during removal; retire it and retry"
        );
        ensure!(
            catalogue
                .bmc_helix_sessions
                .iter()
                .all(|record| record.account_id != account_id
                    || matches!(
                        record.session,
                        CloudTerrastodonAccountSession::Unprovisioned
                            | CloudTerrastodonAccountSession::Retired { .. }
                    )),
            "account acquired another Helix session during removal; retire it and retry"
        );
        catalogue.accounts.retain(|value| value.id != account_id);
        catalogue
            .bmc_helix_sessions
            .retain(|record| record.account_id != account_id);
        catalogue
            .defaults
            .retain(|value| value.account_id != account_id);
        cancellation.with_deferral("removing an account registration", |_| {
            self.save(&catalogue)
        })
    }

    /// Resolve aliases/defaults into approval-safe data without opening secret
    /// storage, acquiring a token, or constructing a provider/session.
    pub fn resolve_azure_selection(
        &self,
        tenant_id: AzureTenantId,
        account: Option<&str>,
        source: AuthSource,
    ) -> Result<AzureBearerAuthenticationSelection> {
        ensure!(
            source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        let catalogue = self.load()?;
        let account = account
            .map(str::parse::<CloudTerrastodonAccountArgument>)
            .transpose()?;
        let profile = catalogue.resolve(
            account.as_ref(),
            Some(&CloudTerrastodonAccountNamespace::Entra { tenant_id }),
        )?;
        profile.azure_bearer_selection(source)
    }

    /// Resolve an explicit Entra account, using its registered tenant rather
    /// than a configured tenant default. This reads account metadata only.
    pub fn resolve_azure_account_selection(
        &self,
        account: &CloudTerrastodonAccountArgument,
        source: AuthSource,
    ) -> Result<AzureBearerAuthenticationSelection> {
        ensure!(
            source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        let catalogue = self.load()?;
        let profile = catalogue.resolve_in_namespaces(
            account,
            |namespace| matches!(namespace, CloudTerrastodonAccountNamespace::Entra { .. }),
            " among Entra accounts",
        )?;
        profile.azure_bearer_selection(source)
    }

    /// Resolve an explicit Entra tenant, an account's tenant, or a PAT account
    /// registered for the requested organization.
    pub fn resolve_devops_selection(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
        tenant_id: Option<AzureTenantId>,
        account: Option<&str>,
        source: AuthSource,
    ) -> Result<AzureDevOpsAuthSelection> {
        ensure!(
            source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        if let Some(tenant_id) = tenant_id {
            ensure!(
                source != AuthSource::PersonalAccessToken,
                "PAT accounts use an organization namespace and do not accept a tenant"
            );
            return Ok(AzureDevOpsAuthSelection::Bearer(
                self.resolve_azure_selection(tenant_id, account, source)?,
            ));
        }
        ensure!(
            account.is_some()
                || matches!(source, AuthSource::Auto | AuthSource::PersonalAccessToken),
            "Entra authentication requires an explicit account or tenant"
        );
        let catalogue = self.load()?;
        let account = account
            .map(str::parse::<CloudTerrastodonAccountArgument>)
            .transpose()?;
        let profile = match &account {
            Some(account) if source != AuthSource::PersonalAccessToken => {
                let qualifier = if source == AuthSource::Auto {
                    " among Entra accounts or PAT accounts for the requested organization"
                } else {
                    " among Entra accounts"
                };
                catalogue.resolve_in_namespaces(
                    account,
                    |namespace| match namespace {
                        CloudTerrastodonAccountNamespace::Entra { .. } => true,
                        CloudTerrastodonAccountNamespace::AzureDevOps {
                            organization: registered,
                        } => source == AuthSource::Auto && registered == organization,
                        CloudTerrastodonAccountNamespace::Gitea { .. }
                        | CloudTerrastodonAccountNamespace::BmcHelix { .. } => false,
                    },
                    qualifier,
                )?
            }
            _ => catalogue.resolve(
                account.as_ref(),
                Some(&CloudTerrastodonAccountNamespace::AzureDevOps {
                    organization: organization.clone(),
                }),
            )?,
        };
        if matches!(
            profile.namespace,
            CloudTerrastodonAccountNamespace::Entra { .. }
        ) {
            return Ok(AzureDevOpsAuthSelection::Bearer(
                profile.azure_bearer_selection(source)?,
            ));
        }
        ensure!(
            profile.select_source(source)? == CloudTerrastodonCredentialSource::PersonalAccessToken,
            "selected account is not a PAT account"
        );
        Ok(AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: profile.id,
        })
    }
    pub fn resolve_gitea(
        &self,
        instance: &str,
        account: Option<&str>,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let catalogue = self.load()?;
        let namespace = CloudTerrastodonAccountNamespace::Gitea {
            instance: instance.parse()?,
        };
        let account = account
            .map(str::parse::<CloudTerrastodonAccountArgument>)
            .transpose()?;
        Ok(catalogue
            .resolve(account.as_ref(), Some(&namespace))?
            .clone())
    }

    /// Resolve an explicit Helix account or the selected server's default.
    pub fn resolve_bmc_helix(
        &self,
        server: &BmcHelixServerUrl,
        account: Option<&CloudTerrastodonAccountArgument>,
    ) -> Result<CloudTerrastodonAccountProfile> {
        Ok(self.load()?.resolve_bmc_helix(server, account)?.clone())
    }

    pub(crate) fn provider_for_bmc_helix(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        server_id: BmcHelixServerId,
    ) -> Result<CredentialAuthenticationProvider> {
        let catalogue = self.load()?;
        ensure!(
            catalogue.bmc_helix_account(profile.id)? == profile,
            "account profile changed; resolve its current binding explicitly"
        );
        let Some(record) = catalogue
            .bmc_helix_sessions
            .iter()
            .find(|record| record.account_id == profile.id && record.server.id() == server_id)
        else {
            bail!("log in to the selected Helix account on this server first");
        };
        let CloudTerrastodonAccountSession::Active { generation } = record.session else {
            bail!("log in to the selected Helix account on this server first");
        };
        let store = self.credential_store(
            profile,
            CloudTerrastodonCredentialSource::BmcHelix,
            generation,
        );
        ensure!(
            !store.directory.join("retired").try_exists()?,
            "selected Helix account session is retired"
        );
        Ok(CredentialAuthenticationProvider::for_bmc_helix(
            profile.id,
            record.server.clone(),
            BmcHelixAuthorization::new(&store.read()?)?,
            generation,
        ))
    }

    pub fn provider_for(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        source: CloudTerrastodonCredentialSource,
    ) -> Result<CredentialAuthenticationProvider> {
        ensure!(
            self.load()?.by_id(profile.id)? == profile,
            "account profile changed; resolve its current binding explicitly"
        );
        let registration = profile.registration(source)?;
        if let CloudTerrastodonAccountCredential::Gitea {
            instance, login, ..
        } = &registration.credential
        {
            let CloudTerrastodonAccountSession::Active { generation } = registration.session else {
                bail!("log in to the selected Gitea account first");
            };
            let store = self.credential_store(profile, source, generation);
            ensure!(
                !store.directory.join("retired").try_exists()?,
                "selected account session is retired"
            );
            let session = Arc::new(GiteaCommandSession::new(login.clone(), &store.read()?)?);
            return CredentialAuthenticationProvider::for_gitea(
                profile.id,
                instance.clone(),
                session,
                generation,
            );
        }
        CredentialAuthenticationProvider::from_accounts([self.context_for(
            profile,
            source
                .auth_source()
                .ok_or_else(|| eyre::eyre!("the account uses a service-specific provider"))?,
        )?])
    }

    /// Capture one admitted session. No future request consults aliases/defaults.
    pub fn context_for(
        &self,
        profile: &CloudTerrastodonAccountProfile,
        source: AuthSource,
    ) -> Result<AuthContext> {
        ensure!(
            self.load()?.by_id(profile.id)? == profile,
            "account profile changed; resolve its current binding explicitly"
        );
        let credential_source = profile.select_source(source)?;
        let registration = profile.registration(credential_source)?;
        let CloudTerrastodonAccountSession::Active { generation } = registration.session else {
            bail!("selected account is not logged in; explicitly provision its credential session");
        };
        let store = self.credential_store(profile, credential_source, generation);
        ensure!(
            !store.directory.join("retired").try_exists()?,
            "selected account session is retired"
        );
        let source = registration
            .credential
            .auth_source()
            .ok_or_else(|| eyre::eyre!("the account uses a service-specific provider"))?;
        let mut state =
            ResolvedAuthContext::new(source, None, crate::is_headless_auth_context(), None);
        state.account_id = profile.id;
        state.tenant_id = match profile.namespace {
            CloudTerrastodonAccountNamespace::Entra { tenant_id } => Some(tenant_id),
            _ => None,
        };
        state.credential_store = Some(store.clone());
        match &registration.credential {
            CloudTerrastodonAccountCredential::AzureCli { .. } => {
                state.azure_cli_sessions = AzureCliSessions::at(store.directory.join("azure-cli"));
            }
            CloudTerrastodonAccountCredential::Browser {
                tenant_id,
                client_id,
            } => {
                // Decoder excerpts can include the persisted refresh credential.
                let session: BrowserSession = facet_json::from_str(&store.read()?)
                    .map_err(|_| eyre::eyre!("decoding the selected browser credential failed"))?;
                ensure!(
                    session.tenant_id == *tenant_id && session.client_id == *client_id,
                    "stored browser credential belongs to a different tenant or application"
                );
                state.browser_token_cache = crate::BrowserTokenCache::from_session(Some(&session));
                state.browser_token_cache.credential_store = Some(store);
                state.browser_session = Some(session);
            }
            CloudTerrastodonAccountCredential::WorkloadIdentity {
                tenant_id,
                client_id,
                assertion_file,
            } => {
                state.workload_identity = Some(WorkloadIdentityConfig {
                    tenant_id: *tenant_id,
                    client_id: *client_id,
                    federated_token: std::fs::read_to_string(assertion_file)
                        .wrap_err("reading the selected workload assertion")?,
                });
            }
            CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken { organization } => {
                state.personal_access_token.set(Arc::new(
                    crate::AzureDevOpsPersonalAccessToken::new(store.read()?),
                ))?;
                state.organization = Some(organization.clone());
            }
            CloudTerrastodonAccountCredential::Gitea { .. } => {
                bail!("Gitea accounts use the tea provider");
            }
            CloudTerrastodonAccountCredential::BmcHelix { .. } => {
                bail!("BMC Helix accounts use the AR System provider");
            }
        }
        Ok(AuthContext::Resolved(Arc::new(state)))
    }
}

#[cfg(test)]
#[path = "named_azure_cache_reuse_tests.rs"]
mod named_azure_cache_reuse_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure_types::EntraApplicationClientId;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use facet::Facet;

    #[tokio::test]
    async fn rename_preserves_account_identity_and_enforces_namespace_uniqueness() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let credential = CloudTerrastodonAccountCredential::AzureCli { tenant_id };
                let account = store
                    .add("operator".parse()?, credential.clone(), &scope)
                    .await?;
                store.add("reserved".parse()?, credential, &scope).await?;
                let other_tenant = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli {
                            tenant_id: AzureTenantId::new(Uuid::from_u128(2)),
                        },
                        &scope,
                    )
                    .await?;
                store.set_default(account.id, &scope).await?;
                let mut catalogue = store.load()?;
                let profile = &mut catalogue.accounts[0];
                profile.verify_principal(CloudTerrastodonAccountPrincipal::EntraUser(
                    cloud_terrastodon_azure_types::EntraUserId::new(Uuid::from_u128(3)),
                ))?;
                profile
                    .registration_mut(CloudTerrastodonCredentialSource::AzureCli)?
                    .session = CloudTerrastodonAccountSession::Active {
                    generation: Uuid::from_u128(4),
                };
                let mut expected = profile.clone();
                store.save(&catalogue)?;

                let duplicate = store.rename(account.id, "reserved".parse()?, &scope).await;
                assert!(
                    duplicate
                        .unwrap_err()
                        .to_string()
                        .contains("already exists")
                );
                assert_eq!(store.load()?.by_id(account.id)?, &expected);

                expected.alias = "admin".parse()?;
                assert_eq!(
                    store
                        .rename(account.id, expected.alias.clone(), &scope)
                        .await?,
                    expected
                );
                let saved = store.load()?;
                assert_eq!(saved.resolve(None, Some(&account.namespace))?, &expected);
                assert_eq!(
                    saved.resolve(Some(&"admin".parse()?), Some(&account.namespace))?,
                    &expected
                );
                assert!(
                    saved
                        .resolve(Some(&"operator".parse()?), Some(&account.namespace))
                        .is_err()
                );
                assert_eq!(saved.by_id(other_tenant.id)?, &other_tenant);
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn helix_server_defaults_are_independent_for_shared_accounts() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let production = "https://helix.example.invalid".parse()?;
        let sandbox = "https://helix-test.example.invalid".parse()?;
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "operator".parse()?,
                        CloudTerrastodonAccountCredential::BmcHelix {
                            server: None,
                            username: "test-operator".parse()?,
                        },
                        &scope,
                    )
                    .await?;
                assert!(store.resolve_bmc_helix(&production, None).is_err());
                store
                    .set_bmc_helix_default(&production, account.id, &scope)
                    .await?;
                assert!(store.resolve_bmc_helix(&sandbox, None).is_err());
                assert_eq!(
                    store
                        .resolve_bmc_helix(&sandbox, Some(&"operator".parse()?))?
                        .id,
                    account.id
                );
                store
                    .set_bmc_helix_default(&sandbox, account.id, &scope)
                    .await?;
                store.clear_bmc_helix_default(&production, &scope).await?;
                assert!(store.resolve_bmc_helix(&production, None).is_err());
                assert_eq!(store.resolve_bmc_helix(&sandbox, None)?.id, account.id);
                eyre::Ok(())
            })
            .await
    }

    #[test]
    fn helix_legacy_catalogue_preserves_account_default_and_generation() -> Result<()> {
        #[derive(Facet)]
        struct LegacyCatalogue {
            accounts: Vec<CloudTerrastodonAccountProfile>,
            defaults: Vec<CloudTerrastodonAccountDefault>,
        }

        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let server: BmcHelixServerUrl = "https://helix.example.invalid".parse()?;
        let mut account = CloudTerrastodonAccountProfile::new(
            CloudTerrastodonAccountId::new(Uuid::from_u128(1)),
            "operator".parse()?,
            CloudTerrastodonAccountCredential::BmcHelix {
                server: Some(server.clone()),
                username: "test-operator".parse()?,
            },
        );
        let source = CloudTerrastodonCredentialSource::BmcHelix;
        let session = CloudTerrastodonAccountSession::Active {
            generation: Uuid::from_u128(2),
        };
        account.registration_mut(source)?.session = session;
        let id = account.id;
        let legacy = LegacyCatalogue {
            defaults: vec![CloudTerrastodonAccountDefault {
                namespace: account.namespace.clone(),
                account_id: id,
            }],
            accounts: vec![account],
        };
        CloudTerrastodonAccountStore::prepare_directory(&store.root)?;
        std::fs::write(
            store.root.join("accounts.json"),
            facet_json::to_vec(&legacy)?,
        )?;
        let catalogue = store.load()?;
        assert_eq!(catalogue.resolve_bmc_helix(&server, None)?.id, id);
        assert_eq!(catalogue.bmc_helix_session(id, &server), session);
        assert_eq!(
            catalogue.bmc_helix_session(id, &"https://other.example.invalid".parse()?),
            CloudTerrastodonAccountSession::Unprovisioned
        );
        store.save(&catalogue)?;
        assert_eq!(
            store.load()?.bmc_helix_sessions,
            catalogue.bmc_helix_sessions
        );
        Ok(())
    }

    #[tokio::test]
    async fn device_code_sign_in_requires_registered_azure_cli_credentials() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let profile = store
                    .add(
                        "synthetic".parse()?,
                        CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                            organization: "synthetic".parse()?,
                        },
                        &scope,
                    )
                    .await?;
                let error = store
                    .sign_in_with_azure_cli_device_code(
                        profile.id,
                        &|_| panic!("an unregistered source must never issue a prompt"),
                        &scope,
                    )
                    .await
                    .unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("no azure-cli credential registration")
                );
                assert_eq!(store.load()?.by_id(profile.id)?, &profile);
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn metadata_resolution_keeps_account_identity_and_explicit_source_without_credentials()
    -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        let other_tenant = AzureTenantId::new(Uuid::from_u128(2));
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                        &scope,
                    )
                    .await?;
                let other = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli {
                            tenant_id: other_tenant,
                        },
                        &scope,
                    )
                    .await?;
                assert!(
                    store
                        .resolve_azure_selection(tenant_id, None, AuthSource::Auto)
                        .is_err()
                );
                store.set_default(account.id, &scope).await?;
                let selection = store.resolve_azure_selection(tenant_id, None, AuthSource::Auto)?;
                assert_eq!(selection.account_id, account.id);
                assert_eq!(selection.tenant_id, tenant_id);
                assert_eq!(selection.auth_source, AzureBearerAuthSource::AzureCli);
                let ambiguous = store
                    .resolve_azure_account_selection(&"admin".parse()?, AuthSource::Auto)
                    .unwrap_err()
                    .to_string();
                assert!(ambiguous.contains("matches multiple registrations"));
                assert!(ambiguous.contains(&account.id.to_string()));
                assert!(ambiguous.contains(&other.id.to_string()));
                assert_eq!(
                    store.resolve_azure_account_selection(
                        &CloudTerrastodonAccountArgument::Id(other.id),
                        AuthSource::Auto,
                    )?,
                    AzureBearerAuthenticationSelection {
                        tenant_id: other_tenant,
                        account_id: other.id,
                        auth_source: AzureBearerAuthSource::AzureCli,
                    },
                );
                assert!(
                    store
                        .resolve_azure_selection(
                            tenant_id,
                            Some(&other.id.to_string()),
                            AuthSource::Auto
                        )
                        .is_err()
                );
                assert!(
                    store
                        .resolve_azure_selection(tenant_id, Some("admin"), AuthSource::Browser)
                        .is_err()
                );
                store
                    .register_credential(
                        account.id,
                        CloudTerrastodonAccountCredential::Browser {
                            tenant_id,
                            client_id: EntraApplicationClientId::new(Uuid::from_u128(3)),
                        },
                        &scope,
                    )
                    .await?;
                store
                    .set_preferred_source(
                        account.id,
                        CloudTerrastodonCredentialSource::Browser,
                        &scope,
                    )
                    .await?;
                let browser =
                    store.resolve_azure_selection(tenant_id, Some("admin"), AuthSource::Auto)?;
                assert_eq!(browser.account_id, selection.account_id);
                assert_eq!(browser.auth_source, AzureBearerAuthSource::Browser);
                assert_eq!(
                    store.resolve_azure_selection(tenant_id, None, AuthSource::AzureCli)?,
                    selection
                );
                assert!(
                    !store.root.join("credentials").exists(),
                    "metadata resolution must not open credential storage"
                );
                let current = store.load()?.by_id(account.id)?.clone();
                assert!(store.context_for(&current, AuthSource::AzureCli).is_err());
                assert!(store.context_for(&current, AuthSource::Browser).is_err());
                // Explicit source overrides never rewrite the persisted preference.
                assert_eq!(
                    current.preferred_source,
                    CloudTerrastodonCredentialSource::Browser
                );
                store.remove(account.id, &scope).await?;
                let replacement = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                        &scope,
                    )
                    .await?;
                assert_ne!(selection.account_id, replacement.id);
                assert!(
                    store
                        .resolve_azure_selection(
                            tenant_id,
                            Some(&selection.account_id.to_string()),
                            AuthSource::Auto
                        )
                        .is_err()
                );
                eyre::Ok(())
            })
            .await
    }

    #[test]
    fn account_tenant_inference_uses_eligible_namespaces() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        let organization: AzureDevOpsOrganizationUrl = "synthetic".parse()?;
        let entra_profile = CloudTerrastodonAccountProfile::new(
            CloudTerrastodonAccountId::new(Uuid::from_u128(2)),
            "personal".parse()?,
            CloudTerrastodonAccountCredential::AzureCli { tenant_id },
        );
        let pat_profile = CloudTerrastodonAccountProfile::new(
            CloudTerrastodonAccountId::new(Uuid::from_u128(3)),
            "personal".parse()?,
            CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                organization: organization.clone(),
            },
        );
        let other_pat_profile = CloudTerrastodonAccountProfile::new(
            CloudTerrastodonAccountId::new(Uuid::from_u128(4)),
            "personal".parse()?,
            CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                organization: "another".parse()?,
            },
        );
        let mut catalogue = CloudTerrastodonAccountCatalogue {
            accounts: vec![other_pat_profile, entra_profile.clone()],
            ..Default::default()
        };
        store.save(&catalogue)?;
        let entra = AzureBearerAuthenticationSelection {
            account_id: entra_profile.id,
            auth_source: AzureBearerAuthSource::AzureCli,
            tenant_id,
        };
        assert_eq!(
            store.resolve_devops_selection(
                &organization,
                None,
                Some("personal"),
                AuthSource::Auto
            )?,
            AzureDevOpsAuthSelection::Bearer(entra),
        );

        catalogue.defaults.push(CloudTerrastodonAccountDefault {
            namespace: pat_profile.namespace.clone(),
            account_id: pat_profile.id,
        });
        catalogue.accounts.push(pat_profile.clone());
        store.save(&catalogue)?;
        let pat = AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: pat_profile.id,
        };
        assert_eq!(
            store.resolve_azure_account_selection(&"personal".parse()?, AuthSource::Auto)?,
            entra,
        );
        let ambiguous = store
            .resolve_devops_selection(&organization, None, Some("personal"), AuthSource::Auto)
            .unwrap_err()
            .to_string();
        assert!(ambiguous.contains(&entra_profile.id.to_string()));
        assert!(ambiguous.contains(&pat_profile.id.to_string()));
        assert_eq!(
            store.resolve_devops_selection(
                &organization,
                None,
                Some("personal"),
                AuthSource::AzureCli,
            )?,
            AzureDevOpsAuthSelection::Bearer(entra),
        );
        assert_eq!(
            store.resolve_devops_selection(
                &organization,
                None,
                Some("personal"),
                AuthSource::PersonalAccessToken,
            )?,
            pat,
        );
        assert_eq!(
            store.resolve_devops_selection(&organization, None, None, AuthSource::Auto)?,
            pat,
        );
        Ok(())
    }

    #[tokio::test]
    async fn logout_retires_all_sources_even_when_each_cleanup_fails() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                        &scope,
                    )
                    .await?;
                store
                    .register_credential(
                        account.id,
                        CloudTerrastodonAccountCredential::Browser {
                            tenant_id,
                            client_id: EntraApplicationClientId::new(Uuid::from_u128(3)),
                        },
                        &scope,
                    )
                    .await?;
                let mut catalogue = store.load()?;
                let profile = &mut catalogue.accounts[0];
                profile.verify_principal(CloudTerrastodonAccountPrincipal::EntraUser(
                    cloud_terrastodon_azure_types::EntraUserId::new(Uuid::from_u128(4)),
                ))?;
                for (index, registration) in profile.credentials.iter_mut().enumerate() {
                    registration.session = CloudTerrastodonAccountSession::Active {
                        generation: Uuid::from_u128(index as u128 + 10),
                    };
                }
                for registration in &profile.credentials {
                    let CloudTerrastodonAccountSession::Active { generation } =
                        registration.session
                    else {
                        unreachable!()
                    };
                    // Force cleanup to fail before any operating-system credential
                    // store or Azure CLI invocation. The durable catalogue must
                    // nevertheless retire both sources and retain both failures.
                    std::fs::create_dir_all(
                        store
                            .credential_store(profile, registration.credential.source(), generation)
                            .directory
                            .join("retired"),
                    )?;
                }
                store.save(&catalogue)?;
                let error = store.logout(account.id, &scope).await.unwrap_err();
                assert_eq!(
                    error
                        .downcast_ref::<crate::auth_logout_error::AuthLogoutError>()
                        .unwrap()
                        .errors
                        .len(),
                    2
                );
                let catalogue = store.load()?;
                let profile = catalogue.by_id(account.id)?;
                assert!(profile.credentials.iter().all(|registration| matches!(
                    registration.session,
                    CloudTerrastodonAccountSession::Retired { .. }
                )));
                assert!(store.context_for(profile, AuthSource::AzureCli).is_err());
                assert!(store.context_for(profile, AuthSource::Browser).is_err());
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn bulk_logout_keeps_registrations_and_continues_after_a_cleanup_failure() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        let client_id = EntraApplicationClientId::new(Uuid::from_u128(2));
        let source = CloudTerrastodonCredentialSource::WorkloadIdentity;
        let generation = Uuid::from_u128(3);
        CancellationToken::new()
            .with_async_scope(async |scope| {
                for alias in ["first", "second"] {
                    store
                        .add(
                            alias.parse()?,
                            CloudTerrastodonAccountCredential::WorkloadIdentity {
                                tenant_id,
                                client_id,
                                assertion_file: directory
                                    .path()
                                    .join("unused-assertion")
                                    .try_into()?,
                            },
                            &scope,
                        )
                        .await?;
                }
                let mut catalogue = store.load()?;
                for profile in &mut catalogue.accounts {
                    profile.verify_principal(
                        CloudTerrastodonAccountPrincipal::EntraApplication(client_id),
                    )?;
                    profile.registration_mut(source)?.session =
                        CloudTerrastodonAccountSession::Active { generation };
                }
                let first = &catalogue.accounts[0];
                std::fs::create_dir_all(
                    store
                        .credential_store(first, source, generation)
                        .directory
                        .join("retired"),
                )?;
                store.save(&catalogue)?;
                store.set_default(first.id, &scope).await?;
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(CredentialAuthenticationProvider::from_store(store.clone())),
                    scope,
                );

                let error = store.logout_all_with_executor(&executor).await.unwrap_err();
                let failures = &error.downcast_ref::<AuthLogoutError>().unwrap().errors;
                assert_eq!(failures.len(), 1);
                assert!(failures[0].to_string().contains(&first.id.to_string()));
                let after = store.load()?;
                assert_eq!(after.defaults[0].account_id, first.id);
                for original in &catalogue.accounts {
                    let retained = after.by_id(original.id)?;
                    assert_eq!(retained.alias, original.alias);
                    assert_eq!(
                        retained.registration(source)?.session,
                        CloudTerrastodonAccountSession::Retired { generation }
                    );
                }
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn reauthentication_retires_one_source_and_releases_cache_before_sign_in() -> Result<()> {
        use cloud_terrastodon_auth_types::AuthenticationSelection;
        use cloud_terrastodon_cache::WorkCache;
        use std::future::ready;
        use std::time::Duration;
        use tokio::time::timeout;

        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let cache_root = directory.path().join("work");
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        let source = CloudTerrastodonCredentialSource::AzureCli;
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                        &scope,
                    )
                    .await?;
                store
                    .register_credential(
                        account.id,
                        CloudTerrastodonAccountCredential::Browser {
                            tenant_id,
                            client_id: EntraApplicationClientId::new(Uuid::from_u128(3)),
                        },
                        &scope,
                    )
                    .await?;
                let mut catalogue = store.load()?;
                let profile = &mut catalogue.accounts[0];
                let principal = CloudTerrastodonAccountPrincipal::EntraUser(
                    cloud_terrastodon_azure_types::EntraUserId::new(Uuid::from_u128(4)),
                );
                profile.verify_principal(principal)?;
                profile
                    .registration_mut(CloudTerrastodonCredentialSource::AzureCli)?
                    .session = CloudTerrastodonAccountSession::Active {
                    generation: Uuid::from_u128(10),
                };
                profile
                    .registration_mut(CloudTerrastodonCredentialSource::Browser)?
                    .session = CloudTerrastodonAccountSession::Active {
                    generation: Uuid::from_u128(11),
                };
                store.save(&catalogue)?;
                let stale = store
                    .logout_generation(
                        account.id,
                        Some(CloudTerrastodonCredentialSource::AzureCli),
                        Some(Uuid::from_u128(9)),
                        None,
                        &scope,
                    )
                    .await
                    .unwrap_err();
                assert!(
                    stale
                        .to_string()
                        .contains("changed during reauthentication")
                );
                assert_eq!(
                    store
                        .load()?
                        .by_id(account.id)?
                        .registration(CloudTerrastodonCredentialSource::AzureCli)?
                        .session,
                    CloudTerrastodonAccountSession::Active {
                        generation: Uuid::from_u128(10),
                    }
                );
                // The synthetic CLI slot has no nested sessions, so cleanup requires
                // only temporary files and never launches a process.
                {
                    let executor = CloudTerrastodonExecutor::new(
                        WorkCache::new(&cache_root)?,
                        Arc::new(CredentialAuthenticationProvider::from_store(store.clone())),
                        scope.clone(),
                    );
                    // Register the partition so retirement exercises the cache.
                    executor.admit(&AuthenticationSelection::AzureBearer(
                        AzureBearerAuthenticationSelection {
                            account_id: account.id,
                            auth_source: AzureBearerAuthSource::AzureCli,
                            tenant_id,
                        },
                    ))?;
                    store.retire_for_reauthentication(account.id, source, &executor).await?;
                }
                assert!(cache_root.exists(), "retirement did not open the cache");

                // Keep sign-in waiting at the catalogue lock. The invalid PAT
                // argument also guarantees it cannot launch Azure CLI if the
                // lock ever stops excluding this second catalogue owner.
                let _catalogue_lease = store.lock(&scope).await?;
                let (cancel, sign_in) = scope.with_async_child_scope(async |sign_in_scope| {
                    store
                        .sign_in(
                            account.id,
                            Some(source),
                            Some("synthetic-invalid-pat-argument"),
                            &|_| unreachable!("the fixture cannot acquire credentials"),
                            &sign_in_scope,
                        )
                        .await
                });
                tokio::pin!(sign_in);
                tokio::select! {
                    biased;
                    result = &mut sign_in => panic!("sign-in did not wait for the catalogue: {result:?}"),
                    () = ready(()) => {}
                }
                timeout(
                    Duration::from_secs(10),
                    WorkCache::new(&cache_root)?.acquire(&scope),
                )
                .await??;
                cancel.request_cancel("the synthetic sign-in wait is complete");
                assert!(sign_in.await.is_err());

                let catalogue = store.load()?;
                let profile = catalogue.by_id(account.id)?;
                assert!(matches!(
                    profile
                        .registration(CloudTerrastodonCredentialSource::AzureCli)?
                        .session,
                    CloudTerrastodonAccountSession::Retired { .. }
                ));
                assert!(matches!(
                    profile
                        .registration(CloudTerrastodonCredentialSource::Browser)?
                        .session,
                    CloudTerrastodonAccountSession::Active { .. }
                ));
                assert_eq!(profile.principal, Some(principal));
                eyre::Ok(())
            })
            .await
    }
    #[tokio::test]
    async fn concurrent_lazy_admission_initializes_one_source_session_and_retains_it() -> Result<()>
    {
        use cloud_terrastodon_auth_types::AuthenticationSelection;
        use cloud_terrastodon_cache::WorkCache;
        use cloud_terrastodon_execution::AuthenticationProvider;
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let assertion = directory.path().join("synthetic-assertion");
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        let client_id = EntraApplicationClientId::new(Uuid::from_u128(2));
        let provider = CredentialAuthenticationProvider::from_store(store.clone());
        // Construction and anonymous admission do not consult the absent store.
        provider.admit(&AuthenticationSelection::Anonymous)?;
        assert!(!store.root.exists());
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "automation".parse()?,
                        CloudTerrastodonAccountCredential::WorkloadIdentity {
                            tenant_id,
                            client_id,
                            assertion_file: assertion.clone().try_into()?,
                        },
                        &scope,
                    )
                    .await?;
                let mut catalogue = store.load()?;
                catalogue.accounts[0].verify_principal(
                    CloudTerrastodonAccountPrincipal::EntraApplication(client_id),
                )?;
                catalogue.accounts[0].credentials[0].session =
                    CloudTerrastodonAccountSession::Active {
                        generation: Uuid::from_u128(3),
                    };
                store.save(&catalogue)?;
                let selection =
                    AuthenticationSelection::AzureBearer(provider.resolve_azure_authentication(
                        tenant_id,
                        Some("automation"),
                        AuthSource::Auto,
                    )?);
                // Selector resolution read metadata only; no assertion exists yet.
                assert!(!assertion.exists());
                assert!(provider.admit(&selection).is_err());
                std::fs::write(&assertion, "synthetic-never-exchanged-assertion")?;
                let barrier = std::sync::Barrier::new(8);
                let partitions = std::thread::scope(|threads| {
                    let handles = (0..8)
                        .map(|_| {
                            let provider = provider.clone();
                            let barrier = &barrier;
                            threads.spawn(move || {
                                barrier.wait();
                                provider.admit(&selection)
                            })
                        })
                        .collect::<Vec<_>>();
                    handles
                        .into_iter()
                        .map(|thread| thread.join().expect("admission thread"))
                        .collect::<Result<Vec<_>>>()
                })?;
                assert!(
                    partitions
                        .iter()
                        .all(|partition| partition == &partitions[0])
                );
                std::fs::remove_file(&assertion)?;
                assert_eq!(provider.admit(&selection)?, partitions[0]);
                assert!(
                    CredentialAuthenticationProvider::from_store(store.clone())
                        .admit(&selection)
                        .is_err()
                );
                provider
                    .retire(account.id, &WorkCache::temporary()?, &scope)
                    .await?;
                assert!(provider.admit(&selection).is_err());
                eyre::Ok(())
            })
            .await
    }
    #[tokio::test]
    async fn provisioned_cli_session_can_be_admitted_headlessly_without_interactive_login()
    -> Result<()> {
        use cloud_terrastodon_auth_types::AuthenticationSelection;
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "automation".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                        &scope,
                    )
                    .await?;
                let mut catalogue = store.load()?;
                catalogue.accounts[0].verify_principal(
                    CloudTerrastodonAccountPrincipal::EntraUser(
                        cloud_terrastodon_azure_types::EntraUserId::new(Uuid::from_u128(2)),
                    ),
                )?;
                catalogue.accounts[0].credentials[0].session =
                    CloudTerrastodonAccountSession::Active {
                        generation: Uuid::from_u128(3),
                    };
                store.save(&catalogue)?;
                let selection =
                    AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                        account_id: account.id,
                        tenant_id,
                        auth_source: AzureBearerAuthSource::AzureCli,
                    });
                let context = store.context_for(&catalogue.accounts[0], AuthSource::AzureCli)?;
                let AuthContext::Resolved(mut state) = context else {
                    unreachable!()
                };
                Arc::get_mut(&mut state)
                    .expect("one freshly constructed session owner")
                    .headless = true;
                let context = AuthContext::Resolved(state);
                assert!(context.admit(&selection).is_ok());
                assert!(
                    !store.root.join("credentials").exists(),
                    "admission must not launch login or token acquisition"
                );
                let unprovisioned = AuthContext::explicit_headless(AuthSource::AzureCli);
                let unprovisioned_selection =
                    AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                        account_id: unprovisioned.account_id()?,
                        tenant_id,
                        auth_source: AzureBearerAuthSource::AzureCli,
                    });
                assert!(unprovisioned.admit(&unprovisioned_selection).is_err());
                eyre::Ok(())
            })
            .await
    }
    #[tokio::test]
    async fn captured_generation_ignores_selector_changes_but_blocks_replacement_admission()
    -> Result<()> {
        use cloud_terrastodon_auth_types::AuthenticationSelection;
        use cloud_terrastodon_execution::AuthenticationProvider;
        let directory = tempfile::tempdir()?;
        let store = CloudTerrastodonAccountStore::new(directory.path().join("accounts"))?;
        let tenant_id = AzureTenantId::new(Uuid::from_u128(1));
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let account = store
                    .add(
                        "admin".parse()?,
                        CloudTerrastodonAccountCredential::AzureCli { tenant_id },
                        &scope,
                    )
                    .await?;
                store
                    .register_credential(
                        account.id,
                        CloudTerrastodonAccountCredential::Browser {
                            tenant_id,
                            client_id: EntraApplicationClientId::new(Uuid::from_u128(2)),
                        },
                        &scope,
                    )
                    .await?;
                let mut catalogue = store.load()?;
                catalogue.accounts[0].verify_principal(
                    CloudTerrastodonAccountPrincipal::EntraUser(
                        cloud_terrastodon_azure_types::EntraUserId::new(Uuid::from_u128(3)),
                    ),
                )?;
                catalogue.accounts[0]
                    .registration_mut(CloudTerrastodonCredentialSource::AzureCli)?
                    .session = CloudTerrastodonAccountSession::Active {
                    generation: Uuid::from_u128(10),
                };
                store.save(&catalogue)?;
                let provider = CredentialAuthenticationProvider::from_store(store.clone());
                let selection =
                    AuthenticationSelection::AzureBearer(provider.resolve_azure_authentication(
                        tenant_id,
                        Some("admin"),
                        AuthSource::AzureCli,
                    )?);
                let retained = provider
                    .lease(&selection)?
                    .expect("credential execution retains a lease");
                let partition = provider.admit(&selection)?;
                // Aliases/default preferences and a sibling source are mutable
                // metadata. None may replace a selected mechanism's session owner.
                catalogue.accounts[0].alias = "administrator".parse()?;
                catalogue.accounts[0].preferred_source = CloudTerrastodonCredentialSource::Browser;
                catalogue.accounts[0]
                    .registration_mut(CloudTerrastodonCredentialSource::Browser)?
                    .session = CloudTerrastodonAccountSession::Active {
                    generation: Uuid::from_u128(11),
                };
                store.save(&catalogue)?;
                assert_eq!(provider.admit(&selection)?, partition);
                assert_eq!(retained.admit(&selection)?, partition);
                retained.validate_cache_reuse(&selection)?;
                // A durable generation change is different: fresh executions fail
                // closed, while already admitted required completion keeps its owner.
                catalogue.accounts[0]
                    .registration_mut(CloudTerrastodonCredentialSource::AzureCli)?
                    .session = CloudTerrastodonAccountSession::Active {
                    generation: Uuid::from_u128(12),
                };
                store.save(&catalogue)?;
                assert!(provider.admit(&selection).is_err());
                assert!(provider.lease(&selection).is_err());
                assert_eq!(retained.admit(&selection)?, partition);
                assert!(retained.validate_cache_reuse(&selection).is_err());
                assert!(
                    !store.root.join("credentials").exists(),
                    "this exercises metadata and lease policy without token acquisition"
                );
                eyre::Ok(())
            })
            .await
    }
}
