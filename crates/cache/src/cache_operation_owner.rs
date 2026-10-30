use std::any::TypeId;
use std::any::type_name;

/// The in-process identity of a canonical operation and its decoding contract.
///
/// Use the request type or a dedicated operation marker as `Operation`. Call
/// sites and caller-specific projections are not distinct canonical owners.
/// This identity is process-local and must not be serialized as disk identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CacheOperationOwner {
    // These remain private so the diagnostic name always describes the TypeId.
    type_id: TypeId,
    type_name: &'static str,
}

impl CacheOperationOwner {
    /// Identify the operation that owns canonical execution and decoding.
    pub fn of<Operation: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<Operation>(),
            type_name: type_name::<Operation>(),
        }
    }

    /// A diagnostic name, not a persistent or collision-proof type identifier.
    pub fn name(&self) -> &'static str {
        self.type_name
    }
}
