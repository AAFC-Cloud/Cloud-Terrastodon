use cloud_terrastodon_azure_types::uuid::Uuid;
use facet::Facet;

/// Durable local credential lifecycle, separate from the approved account ID.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum CloudTerrastodonAccountSession {
    Unprovisioned,
    /// Provisioning has started but no request may use this generation. A retry
    /// cleans interrupted credentials before another attempt.
    Provisioning {
        generation: Uuid,
    },
    Active {
        generation: Uuid,
    },
    /// Keep the generation so failed credential cleanup can be retried.
    Retired {
        generation: Uuid,
    },
}
