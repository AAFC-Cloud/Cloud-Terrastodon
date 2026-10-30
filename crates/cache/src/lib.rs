//! Explicit ownership and logical identity for shared work caches.
//!
//! An explicit owner reserves operation/output contracts and coordinates raw
//! artifact storage. Typed result reuse, initializer coordination, and generation
//! invalidation are subsequent layers. No implicit legacy-cache bridge exists.

mod artifact_age;
mod artifact_attempt;
mod artifact_digest;
mod artifact_directory;
mod artifact_entry;
mod artifact_expiry;
mod artifact_file;
mod artifact_file_digest;
mod artifact_file_path;
mod artifact_freshness;
mod artifact_freshness_wire;
mod artifact_io_guard;
mod artifact_manifest;
mod artifact_metadata;
mod artifact_observation;
mod artifact_retention;
mod artifact_selection;
mod artifact_set;
mod artifact_timestamp_proxy;
mod auth_partition;
mod cache_entry_identity;
mod cache_state;
mod loaded_artifact;
mod stored_artifact;

mod cache_key;
mod cache_operation_owner;
mod cache_ownership_claim;
mod cache_path_component;
mod cache_scope;
mod executor_kind;
mod work_cache;
mod work_cache_state;

pub use artifact_attempt::ArtifactAttempt;
pub use artifact_entry::ArtifactEntry;
pub use artifact_file_path::ArtifactFilePath;
pub use artifact_freshness::ArtifactFreshness;
pub use artifact_metadata::ArtifactMetadata;
pub use artifact_observation::ArtifactObservation;
pub use artifact_retention::ArtifactRetention;
pub use artifact_set::ArtifactSet;
pub use auth_partition::AuthPartition;
pub use cache_entry_identity::CacheEntryIdentity;
pub use cache_key::CacheKey;
pub use cache_operation_owner::CacheOperationOwner;
pub use cache_scope::CacheScope;
pub use executor_kind::ExecutorKind;
pub use loaded_artifact::LoadedArtifact;
pub use stored_artifact::StoredArtifact;
pub use work_cache::WorkCache;
