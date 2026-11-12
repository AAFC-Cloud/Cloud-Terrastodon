use crate::AuthContext;
use crate::AzureCliSessions;
use crate::BrowserSession;
use crate::CloudTerrastodonAccountCatalogue;
use crate::CloudTerrastodonAccountDefault;
use crate::CredentialAuthenticationProvider;
use crate::ResolvedAuthContext;
use crate::WorkloadIdentityConfig;
use crate::azure_cli_session::AzureCliSession;
use crate::azure_cli_session_store::AzureCliSessionStore;
use crate::cloud_terrastodon_account_secret_store::CloudTerrastodonAccountSecretStore;
use crate::entra_authenticated_user_identity::EntraAuthenticatedUserIdentity;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
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
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
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
        let catalogue: CloudTerrastodonAccountCatalogue =
            facet_json::from_slice(&content).wrap_err(
                "decoding the account catalogue; pre-cutover single-credential records require explicit migration or new verified registrations; existing account IDs and credential files have not been changed",
            )?;
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

    pub async fn set_default(
        &self,
        account_id: CloudTerrastodonAccountId,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let namespace = catalogue.by_id(account_id)?.namespace.clone();
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
        catalogue.by_id(account_id)?;
        catalogue
            .defaults
            .retain(|value| value.account_id != account_id);
        cancellation.with_deferral("clearing an account default", |_| self.save(&catalogue))
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
        Self::prepare_directory(&store.directory)?;
        std::fs::write(
            store.directory.join("retired"),
            b"explicit account retirement",
        )?;
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
        self.provision(
            account_id,
            source,
            personal_access_token,
            false,
            cancellation,
        )
        .await
    }

    /// Explicitly adopt application-owned legacy credentials. Never imports an
    /// ambient Azure CLI login; Entra identities are verified before activation.
    pub async fn import_legacy(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        self.provision(account_id, source, None, true, cancellation)
            .await
    }

    async fn provision(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        personal_access_token: Option<&str>,
        import: bool,
        cancellation: &CancellationScope,
    ) -> Result<CloudTerrastodonAccountProfile> {
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let mut profile = catalogue.by_id(account_id)?.clone();
        let source = source.unwrap_or(profile.preferred_source);
        let previous = profile.registration(source)?.session;
        ensure!(
            personal_access_token.is_none()
                || source == CloudTerrastodonCredentialSource::PersonalAccessToken,
            "only PAT accounts accept a personal access token"
        );
        if source == CloudTerrastodonCredentialSource::PersonalAccessToken && !import {
            ensure!(
                personal_access_token.is_some_and(|token| !token.trim().is_empty()),
                "PAT login requires a nonempty credential supplied over standard input"
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
            .acquire_credential(
                &profile,
                source,
                &store,
                personal_access_token,
                import,
                cancellation,
            )
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
        personal_access_token: Option<&str>,
        import: bool,
        cancellation: &CancellationScope,
    ) -> Result<Option<CloudTerrastodonAccountPrincipal>> {
        match &profile.registration(source)?.credential {
            CloudTerrastodonAccountCredential::AzureCli { tenant_id } => {
                let sessions = AzureCliSessionStore::at(store.directory.join("azure-cli"));
                let session = if import {
                    sessions.import_legacy(*tenant_id, cancellation).await?;
                    sessions.load(*tenant_id, cancellation).await?
                } else {
                    sessions.login(*tenant_id, cancellation).await?
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
                let (session, principal) = if import {
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
                    let session = facet_json::from_str(&store.read()?).map_err(|_| {
                        eyre::eyre!("decoding refreshed imported credentials failed")
                    })?;
                    (session, principal)
                } else {
                    crate::browser_access_token::acquire_browser_session(
                        *tenant_id,
                        *client_id,
                        cancellation,
                    )
                    .await?
                };
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
                        .ok_or_else(|| {
                            eyre::eyre!(
                                "PAT login requires a credential supplied over standard input"
                            )
                        })?
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
        }
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
        let _lease = self.lock(cancellation).await?;
        let mut catalogue = self.load()?;
        let mut profile = catalogue.by_id(account_id)?.clone();
        if let Some(source) = source {
            profile.registration(source)?;
        }
        cancellation
            .deferred("retiring account credential sessions", async |protected| {
                let mut cleanup = Vec::new();
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
        catalogue.accounts.retain(|value| value.id != account_id);
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
        let profile = catalogue.resolve(
            &CloudTerrastodonAccountNamespace::Entra { tenant_id },
            account,
        )?;
        let source = profile
            .select_source(source)?
            .auth_source()
            .ok_or_else(|| {
                eyre::eyre!("the selected account does not support Entra authentication")
            })?;
        Ok(AzureBearerAuthenticationSelection {
            account_id: profile.id,
            auth_source: source.try_into()?,
            tenant_id,
        })
    }

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
            matches!(source, AuthSource::Auto | AuthSource::PersonalAccessToken),
            "Entra authentication requires an explicit tenant"
        );
        let catalogue = self.load()?;
        let profile = catalogue.resolve(
            &CloudTerrastodonAccountNamespace::AzureDevOps {
                organization: organization.clone(),
            },
            account,
        )?;
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
        Ok(self
            .load()?
            .resolve(
                &CloudTerrastodonAccountNamespace::Gitea {
                    instance: instance.parse()?,
                },
                account,
            )?
            .clone())
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
                .ok_or_else(|| eyre::eyre!("Gitea uses its dedicated provider"))?,
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
            .ok_or_else(|| eyre::eyre!("Gitea accounts use the tea provider"))?;
        let mut state =
            ResolvedAuthContext::new(source, None, crate::is_headless_auth_context(), None);
        state.account_id = profile.id;
        state.tenant_id = match profile.namespace {
            CloudTerrastodonAccountNamespace::Entra { tenant_id } => Some(tenant_id),
            _ => None,
        };
        state.credential_store = Some(store.clone());
        state.session_id = Uuid::new_v4();
        match &registration.credential {
            CloudTerrastodonAccountCredential::AzureCli { .. } => {
                state.azure_cli_sessions = AzureCliSessions::at(store.directory.join("azure-cli"));
            }
            CloudTerrastodonAccountCredential::Browser {
                tenant_id,
                client_id,
            } => {
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
        }
        Ok(AuthContext::Resolved(Arc::new(state)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure_types::EntraApplicationClientId;
    use cloud_terrastodon_command::CancellationToken;

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
    async fn source_logout_preserves_other_source_and_principal() -> Result<()> {
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
                // The synthetic CLI slot has no nested sessions, so cleanup requires
                // only temporary files and never launches a process.
                store
                    .logout_source(
                        account.id,
                        Some(CloudTerrastodonCredentialSource::AzureCli),
                        &scope,
                    )
                    .await?;
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
                assert!(
                    !store.root.join("credentials").exists(),
                    "this exercises metadata and lease policy without token acquisition"
                );
                eyre::Ok(())
            })
            .await
    }
}
