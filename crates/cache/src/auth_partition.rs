use eyre::Report;
use eyre::Result;
use eyre::ensure;
use facet::Facet;

/// An admitted authentication identity partition supplied by an execution owner.
///
/// This is a diagnostic, non-secret identifier. It must never contain a token,
/// PAT, assertion, or a value derived from credential bytes. Constructing a
/// partition validates its representation; it does not authenticate its owner.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Facet)]
#[facet(proxy = String)]
pub struct AuthPartition {
    // Private storage ensures decoding and construction share nonempty validation.
    value: String,
}

impl AuthPartition {
    /// Validate a nonempty identifier without normalizing its identity.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        Self::try_from(value.into())
    }

    /// Inspect the exact admitted partition identifier.
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl TryFrom<String> for AuthPartition {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        ensure!(
            !value.is_empty(),
            "authentication partition must not be empty"
        );
        Ok(Self { value })
    }
}

impl From<&AuthPartition> for String {
    fn from(partition: &AuthPartition) -> Self {
        partition.value.clone()
    }
}

cloud_terrastodon_registry_values::register_thing!(AuthPartition);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_and_decoding_enforce_nonempty_identity() {
        assert!(AuthPartition::new("").is_err());
        let encoded = facet_json::to_string(&String::new()).unwrap();
        assert!(facet_json::from_str::<AuthPartition>(&encoded).is_err());
    }

    #[test]
    fn preserves_exact_partition_identity() {
        let partition = AuthPartition::new(" Session/é ").unwrap();
        assert_eq!(partition.as_str(), " Session/é ");
        assert_eq!(String::from(&partition), " Session/é ");
        assert_ne!(partition, AuthPartition::new("session/é").unwrap());
    }
}
