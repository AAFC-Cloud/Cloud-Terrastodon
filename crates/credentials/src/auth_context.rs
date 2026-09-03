use crate::AuthSource;
use crate::AzureBearerToken;
use crate::AzureRestResource;
use crate::AzureTenantAuthContext;
use crate::browser_access_token::BrowserSession;
use crate::browser_access_token::BrowserTokenCache;
use crate::browser_access_token::load_browser_session;
use crate::is_headless_auth_context;
use crate::load_workload_identity_config;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use eyre::bail;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type WorkloadIdentityTokenCache =
    Arc<Mutex<HashMap<(AzureTenantId, AzureRestResource), AzureAccessToken<AzureBearerToken>>>>;

/// The authentication policy selected for one CLI invocation.
///
/// This is deliberately owned by the invocation and passed by reference to
/// request/credential helpers. It must not be stored in process-global state:
/// callers may run multiple independent request trees in the same process.
#[derive(Debug, Clone, Default, facet::Facet)]
#[repr(u8)]
pub enum AuthContext {
    /// Synthetic request state used by registry/arbitrary construction.
    /// Production context resolution never returns this variant.
    #[default]
    None,
    Resolved {
        requested_source: AuthSource,
        source: AuthSource,
        #[facet(sensitive)]
        workload_identity: Option<crate::WorkloadIdentityConfig>,
        headless: bool,
        #[facet(opaque, sensitive)]
        browser_token_cache: BrowserTokenCache,
        #[facet(opaque, sensitive)]
        workload_identity_token_cache: WorkloadIdentityTokenCache,
        #[facet(sensitive)]
        browser_session: Option<BrowserSession>,
    },
}

impl AuthContext {
    /// Resolve a CLI preference once, at the process entrypoint.
    pub fn resolve(requested: AuthSource) -> eyre::Result<Self> {
        let headless = is_headless_auth_context();
        let workload_identity =
            if matches!(requested, AuthSource::Auto | AuthSource::WorkloadIdentity) {
                load_workload_identity_config()?
            } else {
                None
            };
        let browser_session = if matches!(requested, AuthSource::Browser) {
            load_browser_session()?
        } else {
            None
        };
        let source = select_auth_source(requested, workload_identity.is_some());
        Ok(Self::Resolved {
            requested_source: requested,
            source,
            workload_identity,
            headless,
            browser_token_cache: BrowserTokenCache::from_session(browser_session.as_ref()),
            workload_identity_token_cache: Arc::new(Mutex::new(HashMap::new())),
            browser_session,
        })
    }

    /// Construct an explicit context for library callers and tests.
    pub fn explicit(source: AuthSource) -> Self {
        Self::Resolved {
            requested_source: source,
            source,
            workload_identity: None,
            headless: false,
            browser_token_cache: BrowserTokenCache::default(),
            workload_identity_token_cache: Arc::new(Mutex::new(HashMap::new())),
            browser_session: None,
        }
    }

    /// Construct an explicit Azure CLI compatibility context.
    ///
    /// This is intentionally distinct from [`Default`], which represents a
    /// non-executable registry/arbitrary placeholder.
    pub fn explicit_azure_cli() -> Self {
        Self::explicit(AuthSource::AzureCli)
    }

    #[cfg(test)]
    pub(crate) fn explicit_headless(source: AuthSource) -> Self {
        let mut context = Self::explicit(source);
        if let Self::Resolved { headless, .. } = &mut context {
            *headless = true;
        }
        context
    }

    pub fn requested_source(&self) -> Option<AuthSource> {
        match self {
            Self::None => None,
            Self::Resolved {
                requested_source, ..
            } => Some(*requested_source),
        }
    }

    pub fn source(&self) -> Option<AuthSource> {
        match self {
            Self::None => None,
            Self::Resolved { source, .. } => Some(*source),
        }
    }

    pub fn workload_identity(&self) -> Option<&crate::WorkloadIdentityConfig> {
        match self {
            Self::None => None,
            Self::Resolved {
                workload_identity, ..
            } => workload_identity.as_ref(),
        }
    }

    /// Attach explicit workload identity configuration for library callers and tests.
    pub fn with_workload_identity(
        mut self,
        config: crate::WorkloadIdentityConfig,
    ) -> eyre::Result<Self> {
        let Self::Resolved {
            workload_identity, ..
        } = &mut self
        else {
            bail!("cannot configure the placeholder authentication context");
        };
        *workload_identity = Some(config);
        Ok(self)
    }

    /// Return the tenant supplied by the selected non-CLI credential source,
    /// when one is available. This lets headless commands avoid resolving a
    /// default tenant through `az account`.
    pub fn tenant_id(&self) -> Option<AzureTenantId> {
        self.workload_identity()
            .map(|config| config.tenant_id)
            .or_else(|| self.browser_session().map(|session| session.tenant_id))
    }

    /// Bind this invocation's authentication policy to one resolved Azure tenant.
    pub fn bind_to_azure_tenant(
        &self,
        tenant_id: AzureTenantId,
    ) -> eyre::Result<AzureTenantAuthContext> {
        AzureTenantAuthContext::new(self, tenant_id)
    }

    pub(crate) fn browser_token_cache(&self) -> Option<&BrowserTokenCache> {
        match self {
            Self::None => None,
            Self::Resolved {
                browser_token_cache,
                ..
            } => Some(browser_token_cache),
        }
    }

    pub(crate) fn workload_identity_token_cache(&self) -> Option<&WorkloadIdentityTokenCache> {
        match self {
            Self::None => None,
            Self::Resolved {
                workload_identity_token_cache,
                ..
            } => Some(workload_identity_token_cache),
        }
    }

    pub(crate) fn browser_session(&self) -> Option<&BrowserSession> {
        match self {
            Self::None => None,
            Self::Resolved {
                browser_session, ..
            } => browser_session.as_ref(),
        }
    }

    pub fn is_headless(&self) -> bool {
        match self {
            Self::None => true,
            Self::Resolved { headless, .. } => *headless,
        }
    }

    pub fn allows_interactive_reauthentication(&self) -> bool {
        matches!(self.source(), Some(AuthSource::AzureCli)) && !self.is_headless()
    }

    /// Automatic source selection must not surprise the caller by opening a
    /// browser. Interactive browser authentication is an explicit `browser`
    /// choice or the tenant-login command.
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
        assert!(matches!(context, AuthContext::None));
        assert_eq!(context.source(), None);
        assert!(context.is_headless());
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
}
