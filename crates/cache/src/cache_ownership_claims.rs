use crate::CacheKey;
use crate::CacheOperationOwner;
use crate::CacheScope;
use crate::ExecutorKind;
use crate::cache_ownership_claim::CacheOwnershipClaim;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::any::TypeId;
use std::any::type_name;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::panic::Location;
use std::sync::Mutex;

/// Canonical operation and output claims that can survive a disk ownership handoff.
#[derive(Debug, Default)]
pub(crate) struct CacheOwnershipClaims {
    entries: Mutex<HashMap<(ExecutorKind, CacheScope), CacheOwnershipClaim>>,
}

impl CacheOwnershipClaims {
    pub fn is_poisoned(&self) -> bool {
        self.entries.is_poisoned()
    }

    #[track_caller]
    pub fn claim<T: 'static>(
        &self,
        kind: ExecutorKind,
        key: &CacheKey,
        owner: CacheOperationOwner,
    ) -> Result<()> {
        let caller = Location::caller();
        let mut entries = self.entries.lock().map_err(|_| {
            eyre!("cache ownership registry was poisoned by a panic while holding its lock")
        })?;
        match entries.entry((kind, key.scope.clone())) {
            Entry::Occupied(entry) => {
                let (kind, scope) = entry.key();
                let existing = entry.get();
                ensure!(
                    existing.owner == owner,
                    "cache operation owner conflict for {kind:?} scope {scope:?}: {} first claimed at {}; {} attempted at {caller}",
                    existing.owner.name(),
                    existing.first_claim,
                    owner.name(),
                );
                ensure!(
                    existing.output_type_id == TypeId::of::<T>(),
                    "cache output type conflict for {kind:?} scope {scope:?}, owner {}: {} first claimed at {}; {} attempted at {caller}",
                    owner.name(),
                    existing.output_type_name,
                    existing.first_claim,
                    type_name::<T>(),
                );
            }
            Entry::Vacant(entry) => {
                entry.insert(CacheOwnershipClaim {
                    owner,
                    output_type_id: TypeId::of::<T>(),
                    output_type_name: type_name::<T>(),
                    first_claim: caller,
                });
            }
        }
        Ok(())
    }
}
