use cloud_terrastodon_cache::WorkAttempt;

/// Execution capabilities and the active cache attempt inherited by child work.
#[derive(Clone, Debug)]
pub(crate) enum ExecutionMode {
    /// Ordinary execution, optionally inside an active cache initializer.
    Normal(Option<WorkAttempt>),
    /// Provider execution cannot use the general work/artifact cache or call
    /// `executor.authorization()` again. Nested requests, cancellation scopes,
    /// and the provider's own sensitive token caches remain available.
    NoWorkCacheOrAuthorizationReentry,
}
