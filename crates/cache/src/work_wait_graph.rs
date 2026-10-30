use crate::CacheEntryIdentity;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::collections::HashMap;
use std::collections::HashSet;

/// Active initializer waits, including children executing after acquiring a gate.
///
/// One edge may belong to several concurrent child invocations. Each admission
/// must be paired with removal when that complete invocation ends or is dropped.
#[derive(Debug, Default)]
pub(crate) struct WorkWaitGraph {
    // Counts preserve another invocation's wait when one parallel child ends.
    edges: HashMap<CacheEntryIdentity, HashMap<CacheEntryIdentity, usize>>,
}

impl WorkWaitGraph {
    /// Admit one wait without introducing a direct or transitive cycle.
    pub fn add(&mut self, source: &CacheEntryIdentity, target: &CacheEntryIdentity) -> Result<()> {
        ensure!(
            source != target && !self.reaches(target, source),
            "cache initialization wait cycle from {source:?} to {target:?}"
        );
        let count = self
            .edges
            .entry(source.clone())
            .or_default()
            .entry(target.clone())
            .or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| eyre!("cache initialization wait count exhausted"))?;
        Ok(())
    }

    /// Release one invocation's wait, reclaiming its edge only after the last.
    pub fn remove(&mut self, source: &CacheEntryIdentity, target: &CacheEntryIdentity) {
        if let Some(children) = self.edges.get_mut(source) {
            if let Some(count) = children.get_mut(target) {
                if *count > 1 {
                    *count -= 1;
                } else {
                    children.remove(target);
                }
            }
            if children.is_empty() {
                self.edges.remove(source);
            }
        }
    }

    fn reaches(&self, start: &CacheEntryIdentity, destination: &CacheEntryIdentity) -> bool {
        let mut pending = vec![start];
        let mut visited = HashSet::new();
        while let Some(current) = pending.pop() {
            if current == destination {
                return true;
            }
            if visited.insert(current)
                && let Some(children) = self.edges.get(current)
            {
                pending.extend(children.keys());
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthPartition;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;

    #[test]
    fn parallel_waits_preserve_cycle_rejection_until_the_last_invocation_ends() {
        let [first, second, third] =
            ["first", "second", "third"].map(|operation| CacheEntryIdentity {
                kind: ExecutorKind::InProcess,
                scope: CacheScope::try_from_components(["wait-graph", operation]).unwrap(),
                auth_partition: AuthPartition::new("synthetic-anonymous").unwrap(),
                fingerprint: NonEmptyString::new("synthetic-input").unwrap(),
            });
        let mut graph = WorkWaitGraph::default();
        assert!(graph.add(&first, &first).is_err());
        graph.add(&first, &second).unwrap();
        graph.add(&first, &second).unwrap();
        graph.add(&second, &third).unwrap();
        assert!(graph.add(&third, &first).is_err());

        graph.remove(&first, &second);
        assert!(graph.add(&third, &first).is_err());
        graph.remove(&first, &second);
        assert!(!graph.edges.contains_key(&first));
        graph.add(&third, &first).unwrap();

        graph.remove(&second, &third);
        graph.remove(&third, &first);
        assert!(graph.edges.is_empty());
        graph.add(&second, &first).unwrap();
    }
}
