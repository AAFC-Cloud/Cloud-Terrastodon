use crate::CacheKey;
use eyre::Result;

/// Supplies an operation's logical cache scope and result freshness policy.
///
/// The operation owns key construction, including its inputs and service
/// identity. This does not select an invalidation policy: implement
/// [`crate::InvalidatesOwnCacheKey`] for the single-key default, or
/// [`crate::CacheInvalidationKeys`] to select several scopes.
pub trait HasCacheKey {
    fn cache_key(&self) -> Result<CacheKey>;
}
