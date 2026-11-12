use crate::AuthContext;
use crate::CloudTerrastodonAccountStore;
use crate::cloud_terrastodon_account_admission::CloudTerrastodonAccountAdmission;
use crate::cloud_terrastodon_account_authentication::CloudTerrastodonAccountAuthentication;
use crate::cloud_terrastodon_account_binding::CloudTerrastodonAccountBinding;
use crate::cloud_terrastodon_account_lease::CloudTerrastodonAccountLease;
use crate::cloud_terrastodon_credential_identity::CloudTerrastodonCredentialIdentity;
use crate::fetch_azure_bearer_access_token;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountService;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::GiteaCommandSession;
use eyre::Result;
use eyre::ensure;
use reqwest::Url;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

/// Invocation-owned account mechanisms, initialized once and retained by leases.
/// There is no current account. Each request selects an exact account and source.
/// Construction is inert; selector resolution reads metadata only, and execution
/// loads a selected credential session on first admission. No ambient fallback.
#[derive(Clone, Default)]
pub struct CredentialAuthenticationProvider {
    store: Option<CloudTerrastodonAccountStore>,
    accounts: Arc<
        Mutex<HashMap<CloudTerrastodonCredentialIdentity, Arc<CloudTerrastodonAccountBinding>>>,
    >,
    // Lock before accounts while initializing; retirement closes every mechanism,
    // including a source that has not yet been initialized in this invocation.
    retired: Arc<Mutex<HashSet<CloudTerrastodonAccountId>>>,
    retired_sources: Arc<Mutex<HashSet<CloudTerrastodonCredentialIdentity>>>,
    leases: HashMap<CloudTerrastodonCredentialIdentity, Arc<CloudTerrastodonAccountLease>>,
}
impl CredentialAuthenticationProvider {
    pub fn configured() -> Self {
        Self::from_store(CloudTerrastodonAccountStore::configured())
    }

    /// Inject the account catalogue location without reading it or its credentials.
    pub fn from_store(store: CloudTerrastodonAccountStore) -> Self {
        Self {
            store: Some(store),
            ..Self::default()
        }
    }

    /// Combine explicit captured sessions for embedding and isolated tests.
    pub fn combine(providers: impl IntoIterator<Item = Self>) -> Result<Self> {
        let mut accounts = HashMap::new();
        for provider in providers {
            ensure!(
                provider.store.is_none(),
                "combine captured providers before installing a catalogue owner"
            );
            ensure!(
                provider.leases.is_empty(),
                "cannot combine already admitted execution leases"
            );
            for (key, binding) in provider
                .accounts
                .lock()
                .map_err(|_| eyre::eyre!("account session map poisoned"))?
                .iter()
            {
                ensure!(
                    accounts.insert(*key, binding.clone()).is_none(),
                    "the invocation supplied the same account mechanism more than once"
                );
            }
        }
        Ok(Self {
            accounts: Arc::new(Mutex::new(accounts)),
            ..Self::default()
        })
    }

    /// Inject one explicit session. Unresolved preferences are not credentials.
    pub fn new(context: AuthContext) -> Result<Self> {
        match context {
            AuthContext::Resolved(_) => Self::from_accounts([context]),
            AuthContext::None => Ok(Self::default()),
            _ => {
                eyre::bail!("an unresolved authentication context cannot supply credentials");
            }
        }
    }

    pub fn from_accounts(contexts: impl IntoIterator<Item = AuthContext>) -> Result<Self> {
        let mut accounts = HashMap::new();
        for context in contexts {
            let AuthContext::Resolved(state) = &context else {
                eyre::bail!("unresolved account cannot be admitted");
            };
            let key =
                CloudTerrastodonCredentialIdentity::for_context(state.account_id, state.source)?;
            let entry = Arc::new(CloudTerrastodonAccountBinding {
                admission: state.admission.clone(),
                authentication: CloudTerrastodonAccountAuthentication::Azure(context),
                profile: None,
            });
            ensure!(
                accounts.insert(key, entry).is_none(),
                "the invocation supplied the same account mechanism more than once"
            );
        }
        Ok(Self {
            accounts: Arc::new(Mutex::new(accounts)),
            ..Self::default()
        })
    }

    pub(crate) fn for_gitea(
        account_id: CloudTerrastodonAccountId,
        instance: CloudTerrastodonAccountService,
        session: Arc<GiteaCommandSession>,
        generation: Uuid,
    ) -> Result<Self> {
        let key = CloudTerrastodonCredentialIdentity {
            account_id,
            source: CloudTerrastodonCredentialSource::Gitea,
        };
        let entry = Arc::new(CloudTerrastodonAccountBinding {
            admission: Arc::new(CloudTerrastodonAccountAdmission::new()),
            authentication: CloudTerrastodonAccountAuthentication::Gitea {
                instance,
                session,
                generation,
            },
            profile: None,
        });
        Ok(Self {
            accounts: Arc::new(Mutex::new(HashMap::from([(key, entry)]))),
            ..Self::default()
        })
    }

    fn entry(
        &self,
        selection: &AuthenticationSelection,
    ) -> Result<Arc<CloudTerrastodonAccountBinding>> {
        let key = CloudTerrastodonCredentialIdentity::for_selection(selection)
            .ok_or_else(|| eyre::eyre!("anonymous execution has no credential session"))?;
        let retired = self
            .retired
            .lock()
            .map_err(|_| eyre::eyre!("account retirement map poisoned"))?;
        ensure!(
            !retired.contains(&key.account_id),
            "account {} is retired; new execution is forbidden",
            key.account_id
        );
        ensure!(
            !self
                .retired_sources
                .lock()
                .map_err(|_| eyre::eyre!("credential retirement map poisoned"))?
                .contains(&key),
            "the selected account mechanism is retired; new execution is forbidden"
        );
        let mut accounts = self
            .accounts
            .lock()
            .map_err(|_| eyre::eyre!("account session map poisoned"))?;
        if let Some(entry) = accounts.get(&key) {
            if let (Some(store), Some(captured)) = (&self.store, &entry.profile) {
                let catalogue = store.load()?;
                let current = catalogue.by_id(key.account_id)?;
                ensure!(
                    current.namespace == captured.namespace
                        && current.principal == captured.principal
                        && current.registration(key.source)?
                            == captured.registration(key.source)?,
                    "the selected account credential changed or retired; start a new invocation for its current generation"
                );
            }
            return Ok(entry.clone());
        }
        let store = self.store.as_ref().ok_or_else(|| {
            eyre::eyre!(
                "account {} was not admitted by this executor",
                key.account_id
            )
        })?;
        let catalogue = store.load()?;
        let profile = catalogue.by_id(key.account_id)?;
        if let AuthenticationSelection::AzureBearer(value) = selection {
            ensure!(
                profile.namespace
                    == cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace::Entra {
                        tenant_id: value.tenant_id
                    },
                "selected account belongs to another tenant"
            );
        }
        // Only local credential loading occurs under this lock, never network
        // authorization. The single publication preserves token caches and the
        // admission owner across concurrent callers and independently cloned handles.
        let provider = store.provider_for(profile, key.source)?;
        let entry = provider
            .accounts
            .lock()
            .map_err(|_| eyre::eyre!("account session map poisoned"))?
            .get(&key)
            .cloned()
            .ok_or_else(|| eyre::eyre!("account provider did not supply the selected mechanism"))?;
        entry.authentication.admit(selection)?;
        let entry = Arc::new(CloudTerrastodonAccountBinding {
            authentication: entry.authentication.clone(),
            admission: entry.admission.clone(),
            profile: Some(profile.clone()),
        });
        accounts.insert(key, entry.clone());
        Ok(entry)
    }

    fn binding(
        &self,
        selection: &AuthenticationSelection,
    ) -> Result<CloudTerrastodonAccountAuthentication> {
        let key = CloudTerrastodonCredentialIdentity::for_selection(selection)
            .ok_or_else(|| eyre::eyre!("anonymous requests have no credential binding"))?;
        if let Some(lease) = self.leases.get(&key) {
            return Ok(lease.binding.clone());
        }
        let entry = self.entry(selection)?;
        ensure!(
            entry
                .admission
                .state
                .lock()
                .map_err(|_| eyre::eyre!("account admission lock poisoned"))?
                .open,
            "account is retired; new execution is forbidden"
        );
        Ok(entry.authentication.clone())
    }

    fn leased(&self, selection: &AuthenticationSelection) -> Result<Self> {
        let Some(key) = CloudTerrastodonCredentialIdentity::for_selection(selection) else {
            return Ok(self.clone());
        };
        if self.leases.contains_key(&key) {
            self.admit(selection)?;
            return Ok(self.clone());
        }
        let entry = self.entry(selection)?;
        let binding = {
            let mut state = entry
                .admission
                .state
                .lock()
                .map_err(|_| eyre::eyre!("account admission lock poisoned"))?;
            ensure!(state.open, "account is retired; new execution is forbidden");
            let binding = entry.authentication.clone();
            state.partitions.insert(binding.admit(selection)?);
            state.active += 1;
            binding
        };
        let mut leased = self.clone();
        leased.leases.insert(
            key,
            Arc::new(CloudTerrastodonAccountLease {
                admission: entry.admission.clone(),
                binding,
            }),
        );
        Ok(leased)
    }

    /// Close all sources before draining required completion. Uninitialized sources
    /// also remain closed; retirement never reloads a replacement generation.
    pub async fn retire(
        &self,
        account_id: CloudTerrastodonAccountId,
        cache: &WorkCache,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.retire_mechanisms(account_id, None, cache, cancellation)
            .await
    }

    /// Retire one mechanism while preserving other sources of the same account.
    pub async fn retire_source(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
        cache: &WorkCache,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.retire_mechanisms(account_id, Some(source), cache, cancellation)
            .await
    }

    async fn retire_mechanisms(
        &self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        cache: &WorkCache,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        ensure!(
            !self.leases.keys().any(|key| key.account_id == account_id
                && source.is_none_or(|source| key.source == source)),
            "an operation cannot retire the account lease it currently owns"
        );
        cancellation
            .deferred("retiring admitted account work", async |protected| {
                let entries = {
                    let mut retired = self
                        .retired
                        .lock()
                        .map_err(|_| eyre::eyre!("account retirement map poisoned"))?;
                    if let Some(source) = source {
                        self.retired_sources
                            .lock()
                            .map_err(|_| eyre::eyre!("credential retirement map poisoned"))?
                            .insert(CloudTerrastodonCredentialIdentity { account_id, source });
                    } else {
                        retired.insert(account_id);
                    }
                    let accounts = self
                        .accounts
                        .lock()
                        .map_err(|_| eyre::eyre!("account session map poisoned"))?;
                    let entries = accounts
                        .iter()
                        .filter(|(key, _)| {
                            key.account_id == account_id
                                && source.is_none_or(|source| key.source == source)
                        })
                        .map(|(_, entry)| entry.clone())
                        .collect::<Vec<_>>();
                    for entry in &entries {
                        entry
                            .admission
                            .state
                            .lock()
                            .map_err(|_| eyre::eyre!("account admission lock poisoned"))?
                            .open = false;
                    }
                    entries
                };
                // Cancellation can wake arbitrary owners; release the admission
                // registry locks before notifying them.
                for entry in &entries {
                    entry
                        .admission
                        .cancellation
                        .request_cancel("the selected account is being retired");
                }
                let mut failures = Vec::new();
                for entry in entries {
                    let admission = &entry.admission;
                    let partitions = admission
                        .state
                        .lock()
                        .map_err(|_| eyre::eyre!("account admission lock poisoned"))?
                        .partitions
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>();
                    for partition in partitions {
                        if let Err(error) = cache
                            .invalidate_authentication(&partition, &protected)
                            .await
                        {
                            failures.push(error);
                        }
                    }
                    loop {
                        let notified = admission.drained.notified();
                        tokio::pin!(notified);
                        notified.as_mut().enable();
                        if admission
                            .state
                            .lock()
                            .map_err(|_| eyre::eyre!("account admission lock poisoned"))?
                            .active
                            == 0
                        {
                            break;
                        }
                        notified.await;
                    }
                    let partitions = admission
                        .state
                        .lock()
                        .map_err(|_| eyre::eyre!("account admission lock poisoned"))?
                        .partitions
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>();
                    for partition in partitions {
                        if let Err(error) = cache
                            .invalidate_authentication(&partition, &protected)
                            .await
                        {
                            failures.push(error);
                        }
                    }
                }
                if failures.is_empty() {
                    Ok(())
                } else {
                    Err(crate::auth_logout_error::AuthLogoutError { errors: failures }.into())
                }
            })
            .await
    }
}
impl AuthenticationProvider for CredentialAuthenticationProvider {
    fn retire_account<'a>(
        &'a self,
        account_id: CloudTerrastodonAccountId,
        source: Option<CloudTerrastodonCredentialSource>,
        cache: &'a WorkCache,
        cancellation: &'a CancellationScope,
    ) -> ExecutionFuture<'a, ()> {
        Box::pin(self.retire_mechanisms(account_id, source, cache, cancellation))
    }

    fn resolve_azure_authentication(
        &self,
        tenant_id: AzureTenantId,
        account: Option<&str>,
        source: AuthSource,
    ) -> Result<AzureBearerAuthenticationSelection> {
        ensure!(
            source != AuthSource::None,
            "authentication is disabled by auth source none"
        );
        self.store
            .as_ref()
            .ok_or_else(|| {
                eyre::eyre!("this injected provider does not resolve account selectors")
            })?
            .resolve_azure_selection(tenant_id, account, source)
    }
    fn resolve_azure_devops_authentication(
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
        self.store
            .as_ref()
            .ok_or_else(|| {
                eyre::eyre!("this injected provider does not resolve account selectors")
            })?
            .resolve_devops_selection(organization, tenant_id, account, source)
    }
    fn resolve_gitea_authentication(
        &self,
        instance: &str,
        account: Option<&str>,
    ) -> Result<CloudTerrastodonAccountId> {
        Ok(self
            .store
            .as_ref()
            .ok_or_else(|| {
                eyre::eyre!("this injected provider does not resolve account selectors")
            })?
            .resolve_gitea(instance, account)?
            .id)
    }
    fn validate_target(&self, selection: &AuthenticationSelection, target: &Url) -> Result<()> {
        self.admit(selection)?;
        if !matches!(
            selection,
            AuthenticationSelection::AzureDevOpsPersonalAccessToken { .. }
        ) {
            return Ok(());
        }
        let CloudTerrastodonAccountAuthentication::Azure(AuthContext::Resolved(state)) =
            self.binding(selection)?
        else {
            eyre::bail!("PAT authorization requires a captured account binding");
        };
        let expected = state
            .organization
            .as_ref()
            .ok_or_else(|| {
                eyre::eyre!("PAT authorization requires an explicitly registered organization")
            })?
            .api_url("dev.azure.com", "", &[])?;
        // Microsoft documents API-area hosts separately from organization
        // identity: https://learn.microsoft.com/en-us/azure/devops/extend/develop/work-with-urls?view=azure-devops
        // Server collection paths and legacy visualstudio.com prefixes remain
        // exact. Never accept a suffix-matching host or partial organization name.
        let same_origin = expected.origin() == target.origin();
        let cloud_api_area = expected.host_str() == Some("dev.azure.com")
            && expected.scheme() == "https"
            && target.scheme() == "https"
            && expected.port().is_none()
            && target.port().is_none()
            && matches!(
                target.host_str(),
                Some(
                    "dev.azure.com"
                        | "vssps.dev.azure.com"
                        | "vsaex.dev.azure.com"
                        | "vsrm.dev.azure.com"
                )
            );
        ensure!(
            target.username().is_empty()
                && target.password().is_none()
                && (same_origin || cloud_api_area),
            "PAT request destination does not belong to the selected account organization"
        );
        let prefix = expected.path().trim_end_matches('/');
        ensure!(
            target.path() == prefix
                || target
                    .path()
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('/')),
            "PAT request destination does not belong to the selected account organization"
        );
        Ok(())
    }
    fn retirement_tokens(&self) -> Vec<CancellationToken> {
        self.leases
            .values()
            .map(|lease| lease.admission.cancellation.clone())
            .collect()
    }
    fn lease(
        &self,
        selection: &AuthenticationSelection,
    ) -> Result<Option<Arc<dyn AuthenticationProvider>>> {
        Ok(Some(Arc::new(self.leased(selection)?)))
    }
    fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        if *selection == AuthenticationSelection::Anonymous {
            return AuthPartition::new("anonymous");
        }
        let key = CloudTerrastodonCredentialIdentity::for_selection(selection)
            .ok_or_else(|| eyre::eyre!("authenticated selection has no account"))?;
        // An admitted lease remains valid through required completion after
        // retirement. It must not consult the closed entry map again.
        if let Some(lease) = self.leases.get(&key) {
            let partition = lease.binding.admit(selection)?;
            lease
                .admission
                .state
                .lock()
                .map_err(|_| eyre::eyre!("account admission lock poisoned"))?
                .partitions
                .insert(partition.clone());
            return Ok(partition);
        }
        let entry = self.entry(selection)?;
        let mut state = entry
            .admission
            .state
            .lock()
            .map_err(|_| eyre::eyre!("account admission lock poisoned"))?;
        ensure!(state.open, "account is retired; new execution is forbidden");
        let partition = entry.authentication.admit(selection)?;
        state.partitions.insert(partition.clone());
        Ok(partition)
    }
    fn gitea_session(
        &self,
        account_id: CloudTerrastodonAccountId,
        instance: &str,
    ) -> Result<Arc<GiteaCommandSession>> {
        let binding = self.binding(&AuthenticationSelection::Gitea { account_id })?;
        let CloudTerrastodonAccountAuthentication::Gitea {
            instance: expected,
            session,
            ..
        } = binding
        else {
            eyre::bail!("selected account is not a tea login");
        };
        ensure!(
            expected == instance.parse::<CloudTerrastodonAccountService>()?,
            "selected Gitea account belongs to another instance"
        );
        Ok(session)
    }
    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            if *selection == AuthenticationSelection::Anonymous {
                return Ok(Authorization::Anonymous);
            }
            let retained = self.leased(selection)?;
            let CloudTerrastodonAccountAuthentication::Azure(context) =
                retained.binding(selection)?
            else {
                eyre::bail!("tea accounts cannot authorize Azure HTTP requests");
            };
            match *selection {
                AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                    tenant_id,
                    ..
                }) => {
                    let token = fetch_azure_bearer_access_token(
                        &context,
                        tenant_id,
                        resource,
                        executor.cancellation(),
                    )
                    .await?;
                    Authorization::bearer(token.access_token.as_str())
                }
                AuthenticationSelection::AzureDevOpsPersonalAccessToken { .. } => {
                    ensure!(
                        resource == AzureCredentialResource::AzureDevOps,
                        "Azure DevOps personal access tokens cannot authorize another resource audience"
                    );
                    Authorization::personal_access_token(
                        context
                            .personal_access_token(executor.cancellation())
                            .await?
                            .as_str(),
                    )
                }
                _ => {
                    eyre::bail!("requested credential source cannot authorize this resource");
                }
            }
        })
    }
    fn container_registry_authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        challenge: &'a ContainerRegistryBearerChallenge,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            let retained = self.leased(selection)?;
            let CloudTerrastodonAccountAuthentication::Azure(context) =
                retained.binding(selection)?
            else {
                eyre::bail!("Container Registry requires an Entra account");
            };
            let AuthenticationSelection::AzureBearer(AzureBearerAuthenticationSelection {
                tenant_id,
                ..
            }) = selection
            else {
                eyre::bail!("Container Registry requires a tenant-bound bearer selection");
            };
            crate::container_registry_authorization::authorize_container_registry(
                challenge,
                &context,
                *tenant_id,
                executor.cancellation(),
            )
            .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureDevOpsPersonalAccessToken;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_command::CancellationToken;

    fn synthetic_pat() -> Result<AuthContext> {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let AuthContext::Resolved(state) = &context else {
            panic!("explicit source resolves a session");
        };
        state
            .personal_access_token
            .set(Arc::new(AzureDevOpsPersonalAccessToken::new(
                "synthetic-pat",
            )))?;
        Ok(context)
    }

    #[tokio::test]
    async fn two_accounts_share_an_executor_without_sharing_admission_or_results() -> Result<()> {
        let first = synthetic_pat()?;
        let second = synthetic_pat()?;
        let first_selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken {
            account_id: first.account_id()?,
        };
        let second_selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken {
            account_id: second.account_id()?,
        };
        let missing = AuthenticationSelection::AzureDevOpsPersonalAccessToken {
            account_id: CloudTerrastodonAccountId::new(Uuid::from_u128(99)),
        };
        let provider = Arc::new(CredentialAuthenticationProvider::from_accounts([
            first.clone(),
            second,
        ])?);
        assert_ne!(
            provider.admit(&first_selection)?,
            provider.admit(&second_selection)?
        );
        assert!(provider.admit(&missing).is_err());
        assert!(
            provider
                .admit(&AuthenticationSelection::AzureBearer(
                    AzureBearerAuthenticationSelection {
                        account_id: first.account_id()?,
                        auth_source: AzureBearerAuthSource::AzureCli,
                        tenant_id: cloud_terrastodon_azure_types::AzureTenantId::new(
                            Uuid::from_u128(1)
                        )
                    }
                ))
                .is_err()
        );
        assert!(CredentialAuthenticationProvider::from_accounts([first.clone(), first]).is_err());
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let executor =
                    CloudTerrastodonExecutor::new(WorkCache::temporary()?, provider.clone(), scope);
                for selection in [first_selection, second_selection] {
                    let authorization = executor
                        .authorization(&selection, AzureCredentialResource::AzureDevOps)
                        .await?;
                    assert!(
                        authorization
                            .header()
                            .is_some_and(|header| header.is_sensitive())
                    );
                    assert!(
                        executor
                            .authorization(&selection, AzureCredentialResource::MicrosoftGraph)
                            .await
                            .is_err()
                    );
                }
                eyre::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn retirement_closes_new_admission_but_drains_existing_sdk_leases() -> Result<()> {
        let context = synthetic_pat()?;
        let account_id = context.account_id()?;
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id };
        let separately_bound = CredentialAuthenticationProvider::new(context.clone())?;
        let provider = CredentialAuthenticationProvider::new(context)?;
        let retained = provider.leased(&selection)?;
        CancellationToken::new().with_async_scope(async |scope| {
            let cache = WorkCache::temporary()?;
            let retirement = provider.retire(account_id, &cache, &scope);
            tokio::pin!(retirement);
            tokio::select! {
                result = &mut retirement => panic!("retirement completed before the admitted request drained: {result:?}"),
                _ = tokio::task::yield_now() => {}
            }
            assert!(provider.admit(&selection).is_err());
            assert!(separately_bound.admit(&selection).is_err());
            assert!(provider.leased(&selection).is_err());
            // Required response/decode/invalidation work retains the old binding.
            assert!(retained.admit(&selection).is_ok());
            let child = retained.leased(&selection)?;
            assert!(child.admit(&selection).is_ok());
            drop(child);
            drop(retained);
            retirement.await?;
            assert!(provider.admit(&selection).is_err());
            eyre::Ok(())
        }).await
    }

    #[test]
    fn tea_binding_rejects_another_instance_and_retains_a_private_snapshot() -> Result<()> {
        let account_id = CloudTerrastodonAccountId::new(Uuid::from_u128(1));
        let session = Arc::new(GiteaCommandSession::new(
            "synthetic".into(),
            "synthetic private fixture",
        )?);
        let directory = session.configuration_home().to_path_buf();
        let provider = CredentialAuthenticationProvider::for_gitea(
            account_id,
            "https://gitea.example.invalid/instance".parse()?,
            session.clone(),
            Uuid::from_u128(2),
        )?;
        drop(session);
        assert!(
            provider
                .gitea_session(account_id, "https://other.example.invalid/instance")
                .is_err()
        );
        let retained =
            provider.gitea_session(account_id, "https://gitea.example.invalid/instance/")?;
        drop(provider);
        assert!(directory.exists());
        assert_eq!(
            std::fs::read_to_string(directory.join("tea/config.yml"))?,
            "synthetic private fixture"
        );
        drop(retained);
        assert!(!directory.exists());
        Ok(())
    }

    #[tokio::test]
    async fn nested_account_admission_retains_both_bindings_until_parent_completion() -> Result<()>
    {
        let first = synthetic_pat()?;
        let second = synthetic_pat()?;
        let selections = [first.account_id()?, second.account_id()?].map(|account_id| {
            AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id }
        });
        let provider = CredentialAuthenticationProvider::from_accounts([first, second])?;
        let first_lease = provider.leased(&selections[0])?;
        let both_leases = first_lease.leased(&selections[1])?;
        drop(first_lease);
        CancellationToken::new().with_async_scope(async |scope| {
            let cache = WorkCache::temporary()?;
            let retire_first = provider.retire(selections[0].account_id().unwrap(), &cache, &scope);
            let retire_second = provider.retire(selections[1].account_id().unwrap(), &cache, &scope);
            tokio::pin!(retire_first, retire_second);
            tokio::select! {
                result = &mut retire_first => panic!("first account retired while a parent retained it: {result:?}"),
                result = &mut retire_second => panic!("second account retired while a parent retained it: {result:?}"),
                _ = tokio::task::yield_now() => {}
            }
            for selection in selections {
                assert!(provider.admit(&selection).is_err());
                both_leases.admit(&selection)?;
            }
            drop(both_leases);
            let (first, second) = tokio::join!(retire_first, retire_second);
            first?;
            second?;
            eyre::Ok(())
        }).await
    }

    #[test]
    fn pat_targets_retain_exact_organization_and_server_collection_boundaries() -> Result<()> {
        for (organization, accepted, rejected) in [
            (
                "https://dev.azure.com/synthetic",
                "https://vssps.dev.azure.com/synthetic/_apis/graph/groups",
                "https://dev.azure.com/synthetic-other/_apis/projects",
            ),
            (
                "https://synthetic.visualstudio.com/DefaultCollection",
                "https://synthetic.visualstudio.com/DefaultCollection/_apis/projects",
                "https://synthetic.visualstudio.com/OtherCollection/_apis/projects",
            ),
            (
                "https://server.example.invalid/deployment/Collection",
                "https://server.example.invalid/deployment/Collection/project/_apis/git",
                "https://server.example.invalid/another/Collection/_apis/projects",
            ),
        ] {
            let mut context = synthetic_pat()?;
            let AuthContext::Resolved(state) = &mut context else {
                unreachable!();
            };
            Arc::get_mut(state)
                .expect("fixture owns its context")
                .organization = Some(organization.parse()?);
            let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken {
                account_id: context.account_id()?,
            };
            let provider = CredentialAuthenticationProvider::new(context)?;
            provider.validate_target(&selection, &accepted.parse()?)?;
            assert!(
                provider
                    .validate_target(&selection, &rejected.parse()?)
                    .is_err()
            );
            assert!(
                provider
                    .validate_target(
                        &selection,
                        &"https://evil.example.invalid/synthetic/_apis".parse()?
                    )
                    .is_err()
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn retirement_cancels_optional_work_but_drains_deferred_completion() -> Result<()> {
        let context = synthetic_pat()?;
        let account_id = context.account_id()?;
        let selection = AuthenticationSelection::AzureDevOpsPersonalAccessToken { account_id };
        let provider = Arc::new(CredentialAuthenticationProvider::new(context)?);
        CancellationToken::new().with_async_scope(async |scope| {
            let cache = WorkCache::temporary()?;
            let executor = CloudTerrastodonExecutor::new(cache.clone(), provider.clone(), scope.clone());
            let (read_started, read_admitted) = tokio::sync::oneshot::channel();
            let (write_started, write_admitted) = tokio::sync::oneshot::channel();
            let finish = tokio::sync::Notify::new();
            let completed = std::sync::atomic::AtomicBool::new(false);
            let read = async {
                let leased = executor.with_authentication_lease(&selection)?;
                leased.with_account_cancellation(async |_child| {
                    read_started.send(()).map_err(|_| eyre::eyre!("read coordinator disappeared"))?;
                    std::future::pending::<Result<()>>().await
                }).await
            };
            let write = async {
                let leased = executor.with_authentication_lease(&selection)?;
                leased.with_account_cancellation(async |child| {
                    child.deferred("synthetic required response and invalidation", async |protected| {
                        write_started.send(()).map_err(|_| eyre::eyre!("write coordinator disappeared"))?;
                        finish.notified().await;
                        // The leased identity remains usable after retirement so
                        // admitted mutation completion can invalidate affected reads.
                        protected.admit(&selection)?;
                        completed.store(true, std::sync::atomic::Ordering::SeqCst);
                        eyre::Ok(())
                    }).await
                }).await
            };
            let retire = async {
                read_admitted.await?;
                write_admitted.await?;
                let retirement = provider.retire(account_id, &cache, &scope);
                tokio::pin!(retirement);
                tokio::select! {
                    result = &mut retirement => panic!("retired before required work completed: {result:?}"),
                    _ = tokio::task::yield_now() => {}
                }
                assert!(provider.admit(&selection).is_err());
                assert!(!completed.load(std::sync::atomic::Ordering::SeqCst));
                finish.notify_one();
                retirement.await
            };
            let (read, write, retired) = tokio::join!(read, write, retire);
            assert!(read.is_err());
            write?;
            retired?;
            assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
            assert!(!scope.is_cancelled(), "account retirement must not cancel unrelated caller work");
            eyre::Ok(())
        }).await
    }
}
