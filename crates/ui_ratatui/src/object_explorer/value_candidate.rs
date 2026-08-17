use cloud_terrastodon_registry::RuntimeValue;
use facet::Shape;

use super::arena_address_source::ArenaAddressSource;
use super::preorder_cursor::PreorderCursor;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use super::value_path::ValuePathSegment;
use super::work_budget::WorkBudget;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ValueOwner {
    ArenaRoot {
        slot: SlotId,
    },
    ReflectedField {
        owner: ValueAddress,
        owner_shape: String,
        field: String,
    },
    SequenceElement {
        owner: ValueAddress,
        owner_shape: String,
        index: usize,
    },
    MapValue {
        owner: ValueAddress,
        owner_shape: String,
        key: String,
    },
}

/// A picker row describing a value without cloning or owning that value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValueCandidate {
    address: ValueAddress,
    shape: String,
    owner: ValueOwner,
}

impl ValueCandidate {
    pub(crate) fn resolve(source: &ArenaAddressSource<'_>, address: ValueAddress) -> Option<Self> {
        let shape =
            cloud_terrastodon_registry::describe_shape(source.resolve(&address).ok()?.shape())
                .to_owned();
        let owner = match address.path().segments().last() {
            None => ValueOwner::ArenaRoot {
                slot: address.root_id(),
            },
            Some(segment) => {
                let parent = address.parent()?;
                let owner_shape = cloud_terrastodon_registry::describe_shape(
                    source.resolve(&parent).ok()?.shape(),
                )
                .to_owned();
                match segment {
                    ValuePathSegment::Field(field) => ValueOwner::ReflectedField {
                        owner: parent,
                        owner_shape,
                        field: field.clone(),
                    },
                    ValuePathSegment::Index(index) => ValueOwner::SequenceElement {
                        owner: parent,
                        owner_shape,
                        index: *index,
                    },
                    ValuePathSegment::Key(key) => ValueOwner::MapValue {
                        owner: parent,
                        owner_shape,
                        key: key.clone(),
                    },
                }
            }
        };
        Some(Self {
            address,
            shape,
            owner,
        })
    }

    pub(crate) const fn address(&self) -> &ValueAddress {
        &self.address
    }

    pub(crate) fn shape(&self) -> &str {
        &self.shape
    }

    pub(crate) const fn owner(&self) -> &ValueOwner {
        &self.owner
    }

    pub(crate) fn ownership_label(&self) -> String {
        match &self.owner {
            ValueOwner::ArenaRoot { slot } => format!("owned by arena slot {slot}"),
            ValueOwner::ReflectedField {
                owner,
                owner_shape,
                field,
            } => format!("field {field} of {owner} ({owner_shape})"),
            ValueOwner::SequenceElement {
                owner,
                owner_shape,
                index,
            } => format!("element {index} of {owner} ({owner_shape})"),
            ValueOwner::MapValue {
                owner,
                owner_shape,
                key,
            } => format!("key {key:?} of {owner} ({owner_shape})"),
        }
    }

    pub(crate) fn display_label(&self) -> String {
        format!("{} — {}", self.address, self.ownership_label())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValueCandidateBatch {
    pub(crate) candidates: Vec<ValueCandidate>,
    pub(crate) inspected: usize,
    pub(crate) complete: bool,
}

/// Advance a picker candidate scan by at most `budget` reflected addresses.
///
/// The caller retains the cursor between UI ticks. Candidate values are never
/// cloned; the bounded batch contains only addresses and small display
/// metadata, including the generic owner relationship requested by pickers.
pub(crate) fn scan_value_candidates(
    cursor: &mut PreorderCursor<'_, ArenaAddressSource<'_>>,
    source: &ArenaAddressSource<'_>,
    target_shape: &'static Shape,
    mut budget: WorkBudget,
) -> ValueCandidateBatch {
    let mut candidates = Vec::new();
    let mut inspected = 0;
    let mut complete = false;
    while budget.try_consume() {
        let Some(address) = cursor.next() else {
            complete = true;
            break;
        };
        inspected += 1;
        if source.resolve(&address).is_ok_and(|value| {
            value.shape().is_shape(target_shape)
                || RuntimeValue::can_own_pointee(target_shape, value.shape())
                || RuntimeValue::can_borrow_pointee(target_shape, value.shape())
        }) && let Some(candidate) = ValueCandidate::resolve(source, address)
        {
            candidates.push(candidate);
        }
    }
    ValueCandidateBatch {
        candidates,
        inspected,
        complete,
    }
}

#[cfg(test)]
mod tests {
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    use super::*;
    use crate::object_explorer::arena::Arena;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::tab::Tab;

    #[derive(Facet)]
    #[repr(C)]
    struct QueryOwner {
        query: Breadcrumbs,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn breadcrumb_picker_candidates_explain_their_tab_field_owners_generically() {
        let mut arena = Arena::default();
        let first = arena
            .insert_ready(runtime(Tab::new("first", Breadcrumbs::default())))
            .unwrap();
        let second = arena
            .insert_ready(runtime(Tab::new("second", Breadcrumbs::default())))
            .unwrap();
        let allocated_roots = arena.allocated_slot_count();
        let source = ArenaAddressSource::new(&arena);
        let mut cursor = PreorderCursor::new(&source);
        let mut matches = Vec::new();

        loop {
            let batch =
                scan_value_candidates(&mut cursor, &source, Breadcrumbs::SHAPE, WorkBudget::new(3));
            assert!(batch.inspected <= 3);
            matches.extend(batch.candidates);
            if batch.complete {
                break;
            }
        }

        let expected = [first, second]
            .map(|slot| {
                ValueAddress::root(slot).child(ValuePathSegment::Field("breadcrumbs".to_owned()))
            })
            .to_vec();
        assert_eq!(
            matches
                .iter()
                .map(ValueCandidate::address)
                .cloned()
                .collect::<Vec<_>>(),
            expected
        );
        for candidate in &matches {
            assert_eq!(candidate.shape(), "Breadcrumbs");
            assert!(matches!(
                candidate.owner(),
                ValueOwner::ReflectedField {
                    owner_shape,
                    field,
                    ..
                } if owner_shape == "Tab" && field == "breadcrumbs"
            ));
            assert!(
                candidate
                    .ownership_label()
                    .contains("field breadcrumbs of slot")
            );
            assert!(candidate.ownership_label().contains("(Tab)"));
            assert!(
                candidate
                    .display_label()
                    .contains(" — field breadcrumbs of slot")
            );
            assert!(candidate.display_label().contains("(Tab)"));
        }
        assert_eq!(
            arena.allocated_slot_count(),
            allocated_roots,
            "describing projected candidates must not allocate view slots"
        );
    }

    #[test]
    fn candidate_owner_provenance_is_not_tab_specific() {
        let mut arena = Arena::default();
        let owner = arena
            .insert_ready(runtime(QueryOwner {
                query: Breadcrumbs::default(),
            }))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let address = ValueAddress::root(owner).child(ValuePathSegment::Field("query".to_owned()));

        let candidate = ValueCandidate::resolve(&source, address.clone()).unwrap();

        assert_eq!(candidate.address(), &address);
        assert_eq!(candidate.shape(), "Breadcrumbs");
        assert_eq!(
            candidate.owner(),
            &ValueOwner::ReflectedField {
                owner: ValueAddress::root(owner),
                owner_shape: "QueryOwner".to_owned(),
                field: "query".to_owned(),
            }
        );
        assert_eq!(
            candidate.display_label(),
            format!("{address} — field query of slot {owner} (QueryOwner)")
        );
    }
}
