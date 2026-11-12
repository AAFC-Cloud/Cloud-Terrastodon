use arbitrary::Arbitrary;
use facet::Facet;

/// Existing password audit categories, preserving the 30-day replacement policy.
#[derive(Debug, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureAuditPasswordCredentialStatus {
    ExpiredWithoutReplacement,
    ExpiringWithoutReplacement,
    ExpiredWithReplacement,
}

cloud_terrastodon_registry::register_thing!(AzureAuditPasswordCredentialStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureAuditPasswordCredentialStatus);
