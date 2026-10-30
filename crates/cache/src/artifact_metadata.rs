use crate::CacheOperationOwner;
use facet::Facet;
use std::any::type_name;

/// Human-readable execution context retained beside a raw artifact payload.
///
/// Both fields are diagnostic only. Neither operation nor Rust output type
/// names determine cache identity or whether the current decoder accepts a
/// payload. Do not include credentials or other sensitive execution inputs.
#[derive(Clone, Debug, Eq, PartialEq, Facet)]
pub struct ArtifactMetadata {
    /// Diagnostic Rust type name of the canonical decoded output.
    pub output_type_name: String,
    /// Diagnostic type name of the operation owning the logical cache address.
    pub operation_owner_type_name: String,
}

impl ArtifactMetadata {
    /// Describe the canonical operation and its output without persisting TypeId.
    pub fn new<T>(operation_owner: CacheOperationOwner) -> Self {
        Self {
            output_type_name: type_name::<T>().to_owned(),
            operation_owner_type_name: operation_owner.name().to_owned(),
        }
    }
}

cloud_terrastodon_registry_values::register_thing!(ArtifactMetadata);
