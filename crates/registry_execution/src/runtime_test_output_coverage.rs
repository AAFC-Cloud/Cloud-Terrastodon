use facet::Facet;
use facet_reflect::Peek;

/// Counts returned collection entries without retaining or serializing payloads.
///
/// This is evidence about the returned value, not proof of a network exchange or
/// decoding on this run. A memory hit may return already decoded entries, and a
/// positive count does not establish coverage of every optional or nested shape.
#[derive(Debug, Default, PartialEq, Eq, Facet)]
#[repr(u8)]
pub enum RuntimeTestOutputCoverage {
    /// The returned collection contains this many entries. Zero proves no item
    /// representation was observed in this returned collection.
    Collection { item_count: usize },
    /// This output has no directly measurable collection, or reflection does
    /// not expose one. No item-count or nested-schema coverage is inferred.
    #[default]
    Unmeasured,
}

impl RuntimeTestOutputCoverage {
    pub fn observe<T: Facet<'static>>(output: &T) -> Self {
        let mut value = Peek::new(output);
        // Only peel ownership wrappers; do not inspect records or traverse
        // service values. The bound also handles unusual recursive pointers.
        for _ in 0..16 {
            if let Ok(pointer) = value.into_pointer()
                && let Some(inner) = pointer.borrow_inner()
            {
                value = inner;
                continue;
            }
            if let Ok(list) = value.into_list_like() {
                return Self::Collection {
                    item_count: list.len(),
                };
            }
            if let Ok(set) = value.into_set() {
                return Self::Collection {
                    item_count: set.len(),
                };
            }
            if let Ok(map) = value.into_map() {
                return Self::Collection {
                    item_count: map.len(),
                };
            }
            break;
        }
        Self::Unmeasured
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Facet)]
    struct NonCloneItem {
        value: usize,
    }

    #[test]
    fn distinguishes_empty_and_populated_shared_collections_without_cloning_items() {
        let empty: Arc<Vec<Arc<NonCloneItem>>> = Arc::new(Vec::new());
        let populated = Arc::new(vec![Arc::new(NonCloneItem { value: 1 })]);
        assert_eq!(
            RuntimeTestOutputCoverage::observe(&empty),
            RuntimeTestOutputCoverage::Collection { item_count: 0 }
        );
        assert_eq!(
            RuntimeTestOutputCoverage::observe(&populated),
            RuntimeTestOutputCoverage::Collection { item_count: 1 }
        );
        assert_eq!(Arc::strong_count(&populated), 1);
        assert_eq!(Arc::strong_count(&populated[0]), 1);
        assert_eq!(
            RuntimeTestOutputCoverage::observe(&NonCloneItem { value: 1 }),
            RuntimeTestOutputCoverage::Unmeasured
        );
    }
}
