//! Explicit ownership and logical identity for shared work caches.
//!
//! An explicit owner reserves operation/output contracts and coordinates raw
//! artifact storage and shared decoded results with per-entry initialization,
//! hierarchical invalidation, and transitive semantic dependencies. No implicit
//! legacy-cache bridge exists; production adapters migrate separately.

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
mod cache_generation_stamp;
mod cache_generations;
mod cache_invalidation_target;
mod cache_state;
mod cached_work_spec;
mod cached_work_value;
mod loaded_artifact;
mod non_empty_string;
mod stored_artifact;

mod cache_key;
mod cache_operation_owner;
mod cache_ownership_claim;
mod cache_path_component;
mod cache_scope;
mod executor_kind;
mod work_attempt;
mod work_attempt_guard;
mod work_attempt_state;
mod work_cache;
mod work_cache_invalidation;
mod work_cache_state;
mod work_certificate;
mod work_child_guard;
mod work_coordination;
mod work_dependency;
mod work_freshness;
mod work_output;
mod work_reuse;
mod work_wait_graph;
mod work_wait_guard;

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
pub use cached_work_spec::CachedWorkSpec;
pub use executor_kind::ExecutorKind;
pub use loaded_artifact::LoadedArtifact;
pub use non_empty_string::NonEmptyString;
pub use stored_artifact::StoredArtifact;
pub use work_attempt::WorkAttempt;
pub use work_cache::WorkCache;
pub use work_output::WorkOutput;
pub use work_reuse::WorkReuse;
