use crate::HasCacheKey;

/// Opt into invalidating the scope selected by this operation's own key.
///
/// This supplies the default [`crate::CacheInvalidationKeys`] implementation.
/// Implement that trait directly instead when invalidation selects other keys;
/// having a cache key alone does not select an invalidation policy.
pub trait InvalidatesOwnCacheKey: HasCacheKey {}
