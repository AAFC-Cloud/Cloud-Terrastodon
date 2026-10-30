use crate::CacheOperationOwner;
use std::any::TypeId;
use std::panic::Location;

/// A retained binding between one logical operation and its canonical output.
/// Claims outlive individual results and cannot be replaced within a handle.
#[derive(Debug)]
pub(crate) struct CacheOwnershipClaim {
    pub owner: CacheOperationOwner,
    pub output_type_id: TypeId,
    pub output_type_name: &'static str,
    pub first_claim: &'static Location<'static>,
}
