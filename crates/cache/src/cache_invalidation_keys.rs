use crate::CacheKey;
use crate::InvalidatesOwnCacheKey;
use eyre::Result;

/// Selects the logical cache scopes invalidated when an operation is refreshed.
///
/// A composed operation can select its children's keys, its own key, or both.
/// Obtain child keys from their owning requests rather than rebuilding their
/// cache layout. Freshness policies do not limit invalidation.
///
/// Implement [`InvalidatesOwnCacheKey`] for the single-own-key default, or
/// implement this trait directly for a custom selection. These are alternative
/// policies: implementing both would conflict with the blanket implementation.
pub trait CacheInvalidationKeys {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>>;
}

impl<T: InvalidatesOwnCacheKey> CacheInvalidationKeys for T {
    fn cache_keys_to_invalidate(&self) -> Result<Vec<CacheKey>> {
        Ok(vec![self.cache_key()?])
    }
}
