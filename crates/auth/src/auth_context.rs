use crate::AzureCliSessions;
use crate::AzureDevOpsPersonalAccessToken;
use crate::AzureTenantAuthContext;
use crate::BrowserSession;
use crate::BrowserTokenCache;
use crate::CancellationScope;
use crate::CredentialAuthenticationProvider;
use crate::ResolvedAuthContext;
use crate::WorkloadIdentityConfig;
use crate::browser_access_token::load_browser_session;
use crate::is_headless_auth_context;
use crate::load_workload_identity_config;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::bail;
use eyre::ensure;
use std::sync::Arc;

/// Invocation-owned authentication policy, credentials and identity partition.
///
/// A resolved context shares its immutable credential session and sensitive
/// caches with clones. Independent contexts never share a session partition.
#[derive(Debug, Clone, Default, facet::Facet)]
#[repr(u8)]
pub enum AuthContext {
    #[default]
    Unresolved,
    None,
    Resolved(#[facet(opaque, sensitive)] Arc<ResolvedAuthContext>),
}

impl AuthContext {
    /// Resolve a startup preference once. Disabled authentication performs no discovery.
    pub fn resolve(requested: AuthSource) -> eyre::Result<Self> {
        if requested == AuthSource::None {
            return Ok(Self::None);
        }
        let workload_identity =
            if matches!(requested, AuthSource::Auto | AuthSource::WorkloadIdentity) {
                load_workload_identity_config()?
            } else {
                None
            };
        let browser_session = if requested == AuthSource::Browser {
            load_browser_session()?
        } else {
            None
        };
        Ok(Self::Resolved(Arc::new(ResolvedAuthContext::new(
            requested,
            select_auth_source(requested, workload_identity.is_some()),
            workload_identity,
            is_headless_auth_context(),
            browser_session,
        ))))
    }

    /// Construct explicit state without reading configuration or acquiring credentials.
    pub fn explicit(source: AuthSource) -> Self {
        Self::explicit_with_headless(source, false)
    }

    pub fn explicit_azure_cli() -> Self {
        Self::explicit(AuthSource::AzureCli)
    }

    /// Construct a library-owned context which forbids interactive authentication.
    pub fn explicit_headless(source: AuthSource) -> Self {
        Self::explicit_with_headless(source, true)
    }

    fn explicit_with_headless(source: AuthSource, headless: bool) -> Self {
        if source == AuthSource::None {
            return Self::None;
        }
        Self::Resolved(Arc::new(ResolvedAuthContext::new(
            source, source, None, headless, None,
        )))
    }

    pub fn requested_source(&self) -> Option<AuthSource> {
        match self {
            Self::Unresolved => None,
            Self::None => Some(AuthSource::None),
            Self::Resolved(state) => Some(state.requested_source),
        }
    }

    pub fn source(&self) -> Option<AuthSource> {
        match self {
            Self::Unresolved => None,
            Self::None => Some(AuthSource::None),
            Self::Resolved(state) => Some(state.source),
        }
    }

    /// Return the resolved bearer selection without acquiring credentials.
    pub fn azure_bearer_source(&self) -> eyre::Result<AzureBearerAuthSource> {
        AzureBearerAuthSource::try_from(self.source().ok_or_else(|| {
            eyre::eyre!("authentication source must be resolved before selecting Azure credentials")
        })?)
    }

    pub fn workload_identity(&self) -> Option<&WorkloadIdentityConfig> {
        match self {
            Self::Resolved(state) => state.workload_identity.as_ref(),
            Self::Unresolved | Self::None => None,
        }
    }

    /// Create a new workload session, leaving existing clones and token caches intact.
    pub fn with_workload_identity(self, config: WorkloadIdentityConfig) -> eyre::Result<Self> {
        if self.source() == Some(AuthSource::None) {
            bail!(
                "authentication is disabled by auth source none; cannot configure workload identity credentials"
            );
        }
        let Self::Resolved(state) = self else {
            bail!("cannot configure the placeholder authentication context");
        };
        ensure!(
            state.source == AuthSource::WorkloadIdentity,
            "workload identity credentials require the workload identity authentication source"
        );
        Ok(Self::Resolved(Arc::new(ResolvedAuthContext::new(
            state.requested_source,
            state.source,
            Some(config),
            state.headless,
            None,
        ))))
    }

    /// A configured source's tenant; this never consults Azure CLI defaults.
    pub fn tenant_id(&self) -> Option<AzureTenantId> {
        self.workload_identity()
            .map(|config| config.tenant_id)
            .or_else(|| self.browser_session().map(|session| session.tenant_id))
    }

    pub fn bind_to_azure_tenant(
        &self,
        tenant_id: AzureTenantId,
    ) -> eyre::Result<AzureTenantAuthContext> {
        AzureTenantAuthContext::new(self, tenant_id)
    }

    /// Explicitly bind this selected credential session to an existing executor.
    ///
    /// Use at the caller boundary after resolving an authentication preference.
    /// This performs no configuration lookup or credential acquisition and keeps
    /// this context's shared sensitive caches and identity partition. The derived
    /// executor retains the supplied cache, scope, and semantic ancestry. SDK and
    /// transport requests must keep their injected provider instead of rebinding it.
    pub fn bind_executor(
        &self,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication(Arc::new(CredentialAuthenticationProvider::new(
            self.clone(),
        )))
    }

    /// Check selection policy and derive identity before any reusable result lookup.
    /// Admission never acquires credentials or reads configuration.
    pub fn admit(&self, selection: &AuthenticationSelection) -> eyre::Result<AuthPartition> {
        if matches!(selection, AuthenticationSelection::Anonymous) {
            return AuthPartition::new("anonymous");
        }
        let state = match self {
            Self::Resolved(state) => state,
            Self::None => {
                bail!("authentication is disabled by auth source none");
            }
            Self::Unresolved => {
                bail!("the placeholder authentication context cannot admit credentials");
            }
        };
        match *selection {
            AuthenticationSelection::Anonymous => AuthPartition::new("anonymous"),
            AuthenticationSelection::AzureDevOpsPersonalAccessToken => {
                ensure!(
                    state.source == AuthSource::PersonalAccessToken,
                    "requested authentication source does not match the invocation source"
                );
                AuthPartition::new(format!("session/{}/source/pat", state.session_id))
            }
            AuthenticationSelection::AzureBearer {
                auth_source,
                tenant_id,
            } => {
                ensure!(
                    state.source == AuthSource::from(auth_source),
                    "requested authentication source does not match the invocation source"
                );
                match auth_source {
                    AzureBearerAuthSource::WorkloadIdentity => {
                        let configured = state.workload_identity.as_ref().ok_or_else(|| {
                            eyre::eyre!("workload identity credentials are not configured")
                        })?;
                        ensure!(
                            configured.tenant_id == tenant_id,
                            "requested tenant does not match the workload identity tenant"
                        );
                        AuthPartition::new(format!(
                            "workload-identity/authority/login.microsoftonline.com/tenant/{}/client/{}",
                            configured.tenant_id, configured.client_id,
                        ))
                    }
                    AzureBearerAuthSource::Browser => {
                        state.browser_token_cache.admit_tenant(tenant_id)?;
                        AuthPartition::new(format!(
                            "session/{}/source/browser/tenant/{tenant_id}",
                            state.session_id,
                        ))
                    }
                    AzureBearerAuthSource::AzureCli => {
                        ensure!(
                            !state.headless,
                            "Azure CLI authentication is unavailable in a headless authentication context"
                        );
                        AuthPartition::new(format!(
                            "session/{}/source/azure-cli/tenant/{tenant_id}",
                            state.session_id,
                        ))
                    }
                }
            }
        }
    }

    /// Obtain the invocation's captured PAT once; later external changes cannot replace it.
    pub async fn personal_access_token(
        &self,
        _cancellation: &CancellationScope,
    ) -> eyre::Result<Arc<AzureDevOpsPersonalAccessToken>> {
        self.admit(&AuthenticationSelection::AzureDevOpsPersonalAccessToken)?;
        let Self::Resolved(state) = self else {
            bail!("authentication state is not resolved");
        };
        state
            .personal_access_token
            .get_or_try_init(|| async {
                crate::get_azure_devops_personal_access_token_from_credential_manager()
                    .await
                    .map(Arc::new)
            })
            .await
            .cloned()
    }

    /// Explicitly establish an application-owned CLI login for the selected tenant.
    /// Existing credential bindings cannot be replaced within this invocation.
    pub async fn login_azure_cli(
        &self,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> eyre::Result<()> {
        self.admit(&AuthenticationSelection::AzureBearer {
            auth_source: AzureBearerAuthSource::AzureCli,
            tenant_id: tenant,
        })?;
        self.azure_cli_sessions()
            .ok_or_else(|| eyre::eyre!("Azure CLI authentication is not selected"))?
            .login(tenant, cancellation)
            .await
    }

    pub(crate) fn azure_cli_sessions(&self) -> Option<&AzureCliSessions> {
        match self {
            Self::Resolved(state) if state.source == AuthSource::AzureCli => {
                Some(&state.azure_cli_sessions)
            }
            _ => None,
        }
    }

    pub(crate) fn browser_token_cache(&self) -> Option<&BrowserTokenCache> {
        match self {
            Self::Resolved(state) => Some(&state.browser_token_cache),
            Self::Unresolved | Self::None => None,
        }
    }

    pub(crate) fn workload_identity_token_cache(
        &self,
    ) -> Option<&crate::resolved_auth_context::WorkloadIdentityTokenCache> {
        match self {
            Self::Resolved(state) => Some(&state.workload_identity_token_cache),
            Self::Unresolved | Self::None => None,
        }
    }

    pub(crate) fn browser_session(&self) -> Option<&BrowserSession> {
        match self {
            Self::Resolved(state) => state.browser_session.as_ref(),
            Self::Unresolved | Self::None => None,
        }
    }

    pub fn is_headless(&self) -> bool {
        match self {
            Self::Resolved(state) => state.headless,
            Self::Unresolved | Self::None => true,
        }
    }

    pub fn allows_interactive_reauthentication(&self) -> bool {
        matches!(self.source(), Some(AuthSource::AzureCli)) && !self.is_headless()
    }

    pub fn allows_interactive_browser_login(&self) -> bool {
        matches!(self.requested_source(), Some(AuthSource::Browser)) && !self.is_headless()
    }
}

fn select_auth_source(requested: AuthSource, has_workload_identity: bool) -> AuthSource {
    match requested {
        AuthSource::Auto if has_workload_identity => AuthSource::WorkloadIdentity,
        AuthSource::Auto => AuthSource::AzureCli,
        explicit => explicit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn explicit_context_does_not_read_or_write_process_state() {
        let context = AuthContext::explicit(AuthSource::WorkloadIdentity);
        assert_eq!(context.source(), Some(AuthSource::WorkloadIdentity));
        assert!(context.workload_identity().is_none());
        assert!(!context.allows_interactive_reauthentication());
    }

    #[test]
    fn default_context_is_an_explicit_non_executable_placeholder() {
        let context = AuthContext::default();
        assert!(matches!(context, AuthContext::Unresolved));
        assert_eq!(context.source(), None);
        assert_eq!(context.requested_source(), None);
        assert!(context.browser_token_cache().is_none());
        assert!(context.workload_identity_token_cache().is_none());
        assert!(context.is_headless());
    }

    #[test]
    fn disabled_authentication_has_no_credentials_caches_or_fallback() -> eyre::Result<()> {
        for context in [
            AuthContext::resolve(AuthSource::None)?,
            AuthContext::explicit(AuthSource::None),
            AuthContext::None,
        ] {
            assert!(matches!(context, AuthContext::None));
            assert_eq!(context.source(), Some(AuthSource::None));
            assert_eq!(context.requested_source(), Some(AuthSource::None));
            assert!(context.tenant_id().is_none());
            assert!(context.workload_identity().is_none());
            assert!(context.browser_session().is_none());
            assert!(context.browser_token_cache().is_none());
            assert!(context.workload_identity_token_cache().is_none());
            assert!(!context.allows_interactive_browser_login());
            assert!(!context.allows_interactive_reauthentication());
        }
        Ok(())
    }

    #[test]
    fn credentials_cannot_be_attached_to_disabled_or_unresolved_contexts() -> eyre::Result<()> {
        for (context, diagnostic) in [
            (AuthContext::None, "auth source none"),
            (AuthContext::Unresolved, "placeholder"),
        ] {
            let error = context
                .with_workload_identity(WorkloadIdentityConfig {
                    client_id: "11111111-1111-1111-1111-111111111111".parse()?,
                    tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
                    federated_token: "synthetic-unused-token".to_owned(),
                })
                .expect_err("a non-executable context must reject credentials");
            assert!(error.to_string().contains(diagnostic));
        }
        Ok(())
    }

    #[test]
    fn auto_prefers_workload_identity_and_defaults_to_azure_cli() {
        assert_eq!(
            select_auth_source(AuthSource::Auto, true),
            AuthSource::WorkloadIdentity
        );
        assert_eq!(
            select_auth_source(AuthSource::Auto, false),
            AuthSource::AzureCli
        );
        assert_eq!(
            select_auth_source(AuthSource::Browser, false),
            AuthSource::Browser
        );
        assert!(!AuthContext::explicit(AuthSource::Auto).allows_interactive_browser_login());
        assert!(AuthContext::explicit(AuthSource::Browser).allows_interactive_browser_login());
        assert!(
            !AuthContext::explicit_headless(AuthSource::Browser).allows_interactive_browser_login()
        );
    }

    #[test]
    fn clones_share_session_partitions_but_independent_contexts_do_not() -> eyre::Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        for (source, selection) in [
            (
                AuthSource::AzureCli,
                AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::AzureCli,
                    tenant_id,
                },
            ),
            (
                AuthSource::Browser,
                AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::Browser,
                    tenant_id,
                },
            ),
            (
                AuthSource::PersonalAccessToken,
                AuthenticationSelection::AzureDevOpsPersonalAccessToken,
            ),
        ] {
            let context = AuthContext::explicit(source);
            assert_eq!(
                context.admit(&selection)?,
                context.clone().admit(&selection)?
            );
            assert_ne!(
                context.admit(&selection)?,
                AuthContext::explicit(source).admit(&selection)?
            );
        }
        Ok(())
    }

    #[test]
    fn workload_partition_tracks_configured_principal_and_rejects_other_selections()
    -> eyre::Result<()> {
        let config = WorkloadIdentityConfig {
            client_id: "11111111-1111-1111-1111-111111111111".parse()?,
            tenant_id: "22222222-2222-2222-2222-222222222222".parse()?,
            federated_token: "synthetic-assertion-one".to_owned(),
        };
        let selection = AuthenticationSelection::AzureBearer {
            auth_source: AzureBearerAuthSource::WorkloadIdentity,
            tenant_id: config.tenant_id,
        };
        let context = AuthContext::explicit(AuthSource::WorkloadIdentity)
            .with_workload_identity(config.clone())?;
        let rotated_assertion = AuthContext::explicit(AuthSource::WorkloadIdentity)
            .with_workload_identity(WorkloadIdentityConfig {
                federated_token: "synthetic-assertion-two".to_owned(),
                ..config.clone()
            })?;
        assert_eq!(
            context.admit(&selection)?,
            rotated_assertion.admit(&selection)?
        );
        assert!(
            !context
                .admit(&selection)?
                .as_str()
                .contains("synthetic-assertion")
        );

        let other_principal = context
            .clone()
            .with_workload_identity(WorkloadIdentityConfig {
                client_id: "33333333-3333-3333-3333-333333333333".parse()?,
                ..config.clone()
            })?;
        assert_ne!(
            context.admit(&selection)?,
            other_principal.admit(&selection)?
        );
        assert_eq!(
            context.workload_identity().unwrap().client_id,
            config.client_id
        );
        assert!(!std::ptr::eq(
            context.workload_identity_token_cache().unwrap(),
            other_principal.workload_identity_token_cache().unwrap()
        ));

        for rejected in [
            AuthenticationSelection::AzureBearer {
                auth_source: AzureBearerAuthSource::AzureCli,
                tenant_id: config.tenant_id,
            },
            AuthenticationSelection::AzureBearer {
                auth_source: AzureBearerAuthSource::WorkloadIdentity,
                tenant_id: "44444444-4444-4444-4444-444444444444".parse()?,
            },
            AuthenticationSelection::AzureDevOpsPersonalAccessToken,
        ] {
            assert!(context.admit(&rejected).is_err());
        }
        assert!(
            AuthContext::explicit(AuthSource::WorkloadIdentity)
                .admit(&selection)
                .is_err()
        );
        assert!(AuthContext::None.admit(&selection).is_err());
        assert!(AuthContext::Unresolved.admit(&selection).is_err());
        assert!(
            AuthContext::None
                .admit(&AuthenticationSelection::Anonymous)
                .is_ok()
        );
        Ok(())
    }

    #[tokio::test]
    async fn executor_binding_uses_the_selected_context_and_preserves_disabled_policy()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let original = AuthContext::explicit(AuthSource::AzureCli);
                let selected = AuthContext::explicit(AuthSource::Browser);
                let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
                let browser = AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::Browser,
                    tenant_id,
                };
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(CredentialAuthenticationProvider::new(original)),
                    scope,
                );
                assert!(executor.admit(&browser).is_err());

                let rebound = selected.bind_executor(&executor)?;
                assert_eq!(rebound.admit(&browser)?, selected.admit(&browser)?);
                assert_eq!(
                    selected.clone().bind_executor(&executor)?.admit(&browser)?,
                    rebound.admit(&browser)?
                );
                // Binding is explicit and cannot mutate the caller's original
                // provider or fall back when the selected context is disabled.
                assert!(executor.admit(&browser).is_err());
                for context in [AuthContext::None, AuthContext::Unresolved] {
                    let disabled = context.bind_executor(&rebound)?;
                    assert!(disabled.admit(&browser).is_err());
                    assert!(disabled.admit(&AuthenticationSelection::Anonymous).is_ok());
                }
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn captured_pat_is_shared_without_resolving_external_credentials() -> eyre::Result<()> {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        let AuthContext::Resolved(state) = &context else {
            panic!("explicit PAT context must be resolved");
        };
        let captured = Arc::new(AzureDevOpsPersonalAccessToken::new(
            "synthetic-captured-pat",
        ));
        state.personal_access_token.set(captured.clone())?;
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                let first = context.personal_access_token(&scope).await?;
                let second = context.clone().personal_access_token(&scope).await?;
                assert!(Arc::ptr_eq(&first, &captured));
                assert!(Arc::ptr_eq(&first, &second));
                assert!(!format!("{context:?}").contains("synthetic-captured-pat"));
                Ok(())
            })
            .await
    }
}
