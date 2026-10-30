use facet::Facet;

/// Controls raw artifact persistence and reuse for an operation.
///
/// This policy is independent of cache validity and execution success. Select
/// it before beginning an attempt, including before writing adjacent inputs.
/// The caller chooses the policy; the store does not detect or redact secrets.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum ArtifactRetention {
    /// Persist inputs when staged, and payloads, diagnostics, and metadata when
    /// an attempt is finished. Permit reuse after successful decoding when fresh.
    /// Zero cache validity still retains these artifacts but prevents reuse.
    Full,
    /// Bypass artifact reads, input staging, outcome retention, and publication.
    ///
    /// Use this for credential-producing operations whose raw inputs or outputs
    /// must not enter the artifact store. It does not skip the operation, redact
    /// its results, or delete previously retained artifacts. Credential caching
    /// and deliberately redacted diagnostics belong to the authentication provider.
    Disabled,
}

cloud_terrastodon_registry_values::register_thing!(ArtifactRetention);
