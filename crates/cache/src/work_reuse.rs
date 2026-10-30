use facet::Facet;

/// Whether an operation's full semantic behavior may be replaced by a cache hit.
///
/// The owning operation decides this before lookup, including its transitive
/// obligations. Mutations, solicitation, and deliberately uncached reads must
/// execute every time. Credential acquisition belongs to the credential provider,
/// outside this semantic work cache; it cannot opt out of dependencies here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Facet)]
#[repr(u8)]
pub enum WorkReuse {
    /// A repeatable read whose semantic children are also reusable reads.
    ReusableRead,
    /// Execute this invocation even when a matching reusable value exists.
    ExecuteEveryTime,
}

cloud_terrastodon_registry_values::register_thing!(WorkReuse);
