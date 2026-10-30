//! Explicit ownership and logical identity for shared work caches.
//!
//! This foundation reserves operation/output contracts within an explicitly
//! owned handle. It does not yet store results, coordinate in-flight work, or
//! read, write, or invalidate artifacts. It has no implicit relationship with
//! the legacy command cache.

mod cache_key;
mod cache_operation_owner;
mod cache_ownership_claim;
mod cache_scope;
mod executor_kind;
mod work_cache;
mod work_cache_state;

pub use cache_key::CacheKey;
pub use cache_operation_owner::CacheOperationOwner;
pub use cache_scope::CacheScope;
pub use executor_kind::ExecutorKind;
pub use work_cache::WorkCache;
