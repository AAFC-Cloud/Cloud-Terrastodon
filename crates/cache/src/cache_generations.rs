use crate::CacheEntryIdentity;
use crate::cache_generation_stamp::CacheGenerationStamp;
use crate::cache_invalidation_target::CacheInvalidationTarget;
use eyre::Result;
use eyre::eyre;
use std::collections::HashMap;

/// Nonwrapping invalidation history retained for one cache owner's lifetime.
#[derive(Debug, Default)]
pub(crate) struct CacheGenerations {
    prefixes: HashMap<Vec<String>, u64>,
    entries: HashMap<CacheEntryIdentity, u64>,
    // Exhaustion cannot leave old certificates valid or wrap back to zero.
    exhausted: Vec<CacheInvalidationTarget>,
}

impl CacheGenerations {
    pub fn capture(&self, identity: &CacheEntryIdentity) -> CacheGenerationStamp {
        let components = identity.scope.components();
        CacheGenerationStamp {
            ancestors: (1..=components.len())
                .map(|length| {
                    self.prefixes
                        .get(&components[..length])
                        .copied()
                        .unwrap_or(0)
                })
                .collect(),
            exact: self.entries.get(identity).copied().unwrap_or(0),
        }
    }

    pub fn matches(&self, identity: &CacheEntryIdentity, stamp: &CacheGenerationStamp) -> bool {
        !self.exhausted.iter().any(|target| target.covers(identity))
            && self.capture(identity) == *stamp
    }

    pub fn advance(&mut self, target: &CacheInvalidationTarget) -> Result<()> {
        let generation = match target {
            CacheInvalidationTarget::Scope(scope) => self
                .prefixes
                .entry(scope.components().to_vec())
                .or_default(),
            CacheInvalidationTarget::Entry(identity) => {
                self.entries.entry(identity.clone()).or_default()
            }
        };
        let Some(next) = generation.checked_add(1) else {
            if !self.exhausted.contains(target) {
                self.exhausted.push(target.clone());
            }
            return Err(eyre!(
                "cache generation exhausted; covered entries cannot be reused"
            ));
        };
        *generation = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthPartition;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;

    fn identity(components: &[&str]) -> CacheEntryIdentity {
        CacheEntryIdentity {
            kind: ExecutorKind::Rest,
            scope: CacheScope::try_from_components(components.iter().copied()).unwrap(),
            auth_partition: AuthPartition::new("synthetic").unwrap(),
            fingerprint: NonEmptyString::new("query").unwrap(),
        }
    }

    #[test]
    fn a_new_prefix_record_rejects_old_descendants_and_preserves_siblings() {
        let mut generations = CacheGenerations::default();
        let child = identity(&["projects", "one", "pages", "1"]);
        let sibling = identity(&["projects", "two", "pages", "1"]);
        let original = generations.capture(&child);
        let unrelated = generations.capture(&sibling);
        let target = CacheInvalidationTarget::Scope(
            CacheScope::try_from_components(["projects", "one"]).unwrap(),
        );
        generations.advance(&target).unwrap();
        assert!(!generations.matches(&child, &original));
        assert!(generations.matches(&sibling, &unrelated));
        assert!(generations.matches(&child, &generations.capture(&child)));
        let new_descendant = identity(&["projects", "one", "pages", "2"]);
        assert_eq!(generations.capture(&new_descendant).ancestors[1], 1);
    }

    #[test]
    fn exhausted_generations_never_make_old_or_new_stamps_reusable() {
        let mut generations = CacheGenerations::default();
        let entry = identity(&["projects", "one"]);
        generations.entries.insert(entry.clone(), u64::MAX);
        let stamp = generations.capture(&entry);
        assert!(
            generations
                .advance(&CacheInvalidationTarget::Entry(entry.clone()))
                .is_err()
        );
        assert!(!generations.matches(&entry, &stamp));
        assert!(!generations.matches(&entry, &generations.capture(&entry)));
    }
}
