use crate::AuthSource;
use crate::is_headless_auth_context;
use crate::load_workload_identity_config;

/// The authentication policy selected for one CLI invocation.
///
/// This is deliberately owned by the invocation and passed by reference to
/// request/credential helpers. It must not be stored in process-global state:
/// callers may run multiple independent request trees in the same process.
#[derive(Debug, Clone)]
pub struct AuthContext {
    source: AuthSource,
    workload_identity: Option<crate::WorkloadIdentityConfig>,
    headless: bool,
}

impl AuthContext {
    /// Resolve a CLI preference once, at the process entrypoint.
    pub fn resolve(requested: AuthSource) -> eyre::Result<Self> {
        let workload_identity = load_workload_identity_config()?;
        let source = match requested {
            AuthSource::Auto if workload_identity.is_some() => AuthSource::WorkloadIdentity,
            AuthSource::Auto => AuthSource::AzureCli,
            explicit => explicit,
        };
        Ok(Self {
            source,
            workload_identity,
            headless: is_headless_auth_context(),
        })
    }

    /// Construct an explicit context for library callers and tests.
    pub fn explicit(source: AuthSource) -> Self {
        Self {
            source,
            workload_identity: None,
            headless: false,
        }
    }

    pub fn source(&self) -> AuthSource {
        self.source
    }

    pub fn workload_identity(&self) -> Option<&crate::WorkloadIdentityConfig> {
        self.workload_identity.as_ref()
    }

    pub fn allows_interactive_reauthentication(&self) -> bool {
        matches!(self.source, AuthSource::AzureCli) && !self.headless
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_context_does_not_read_or_write_process_state() {
        let context = AuthContext::explicit(AuthSource::WorkloadIdentity);
        assert_eq!(context.source(), AuthSource::WorkloadIdentity);
        assert!(context.workload_identity().is_none());
        assert!(!context.allows_interactive_reauthentication());
    }
}
