use crate::CacheEntryIdentity;
use crate::CacheScope;

/// The exact coverage of one invalidation, including a failed durable attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CacheInvalidationTarget {
    Scope(CacheScope),
    Entry(CacheEntryIdentity),
}

impl CacheInvalidationTarget {
    pub fn covers(&self, identity: &CacheEntryIdentity) -> bool {
        match self {
            Self::Scope(scope) => identity.scope.starts_with(scope),
            Self::Entry(entry) => entry == identity,
        }
    }

    /// Whether durable success for this target also resolves another failure.
    pub fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Scope(scope), Self::Scope(other)) => other.starts_with(scope),
            (Self::Scope(_), Self::Entry(other)) => self.covers(other),
            (Self::Entry(entry), Self::Entry(other)) => entry == other,
            (Self::Entry(_), Self::Scope(_)) => false,
        }
    }
}
