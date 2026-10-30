use arbitrary::Arbitrary;
use facet::Facet;

/// Separate cache namespaces for transport envelopes and composed SDK outputs.
///
/// This identifies an execution family, not its effects or cancellation policy.
/// `Other` is an open vocabulary for additional execution families. Its values
/// remain distinct from built-in variants even when their text matches a name.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Facet, Arbitrary)]
#[repr(u8)]
pub enum ExecutorKind {
    Command,
    Rest,
    InProcess,
    Other(String),
}

cloud_terrastodon_registry_values::register_thing!(ExecutorKind);
cloud_terrastodon_registry_values::register_arbitrary!(ExecutorKind);
