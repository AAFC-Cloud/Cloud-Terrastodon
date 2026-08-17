use super::arena::Arena;
use super::arena_address_source::ArenaAddressSource;
use super::borrow_lease::BorrowHolder;
use super::borrow_lease::BorrowId;
use super::borrow_lease::BorrowLease;
use super::slot_id::SlotId;
use super::value_address::ValueAddress;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BorrowError {
    SourceNotReady(ValueAddress),
    SourceProtected { root: SlotId, leases: usize },
    UnknownLease(BorrowId),
}

impl fmt::Display for BorrowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotReady(source) => {
                write!(
                    formatter,
                    "borrow source {:?} is not a resolved value",
                    source
                )
            }
            Self::SourceProtected { root, leases } => write!(
                formatter,
                "slot {root} cannot be mutated while protected by {leases} borrow lease(s)"
            ),
            Self::UnknownLease(id) => write!(formatter, "borrow lease {id:?} does not exist"),
        }
    }
}

impl Error for BorrowError {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BorrowEdge {
    source: ValueAddress,
    holder: BorrowHolder,
    field: String,
}

#[derive(Default)]
pub(crate) struct BorrowGraph {
    next_id: u64,
    edges: BTreeMap<BorrowId, BorrowEdge>,
}

impl BorrowGraph {
    pub(crate) fn borrow(
        &mut self,
        arena: &Arena,
        source: ValueAddress,
        borrower: SlotId,
        field: impl Into<String>,
    ) -> Result<BorrowLease, BorrowError> {
        if ArenaAddressSource::new(arena).resolve(&source).is_err() {
            return Err(BorrowError::SourceNotReady(source));
        }
        let id = BorrowId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("borrow identity space exhausted");
        let holder = BorrowHolder::Builder(borrower);
        let field = field.into();
        self.edges.insert(
            id,
            BorrowEdge {
                source: source.clone(),
                holder,
                field: field.clone(),
            },
        );
        Ok(BorrowLease::new(id, source, holder, field))
    }

    pub(crate) fn transfer_to_pending(
        &mut self,
        lease: &mut BorrowLease,
        pending_slot: SlotId,
    ) -> Result<(), BorrowError> {
        self.transfer(lease, BorrowHolder::PendingInvocation(pending_slot))
    }

    pub(crate) fn duplicate_for_builder(
        &mut self,
        arena: &Arena,
        lease: &BorrowLease,
        borrower: SlotId,
        field: impl Into<String>,
    ) -> Result<BorrowLease, BorrowError> {
        if !self.contains(lease) {
            return Err(BorrowError::UnknownLease(lease.id()));
        }
        self.borrow(arena, lease.source().clone(), borrower, field)
    }

    pub(crate) fn duplicate_for_pending(
        &mut self,
        arena: &Arena,
        lease: &BorrowLease,
        pending_slot: SlotId,
        field: impl Into<String>,
    ) -> Result<BorrowLease, BorrowError> {
        let mut duplicate = self.duplicate_for_builder(arena, lease, pending_slot, field)?;
        if let Err(error) = self.transfer_to_pending(&mut duplicate, pending_slot) {
            let _ = self.release(duplicate);
            return Err(error);
        }
        Ok(duplicate)
    }

    pub(crate) fn transfer_to_ready(
        &mut self,
        lease: &mut BorrowLease,
        ready_slot: SlotId,
    ) -> Result<(), BorrowError> {
        self.transfer(lease, BorrowHolder::ReadyValue(ready_slot))
    }

    pub(crate) fn transfer(
        &mut self,
        lease: &mut BorrowLease,
        holder: BorrowHolder,
    ) -> Result<(), BorrowError> {
        let edge = self
            .edges
            .get_mut(&lease.id())
            .ok_or(BorrowError::UnknownLease(lease.id()))?;
        edge.holder = holder;
        lease.transfer(holder);
        Ok(())
    }

    pub(crate) fn release(&mut self, lease: BorrowLease) -> Result<(), BorrowError> {
        self.edges
            .remove(&lease.id())
            .ok_or(BorrowError::UnknownLease(lease.id()))?;
        Ok(())
    }

    pub(crate) fn protects_root(&self, root: SlotId) -> bool {
        self.edges
            .values()
            .any(|edge| edge.source.root_id() == root)
    }

    pub(crate) fn ensure_root_unprotected(&self, root: SlotId) -> Result<(), BorrowError> {
        let leases = self
            .edges
            .values()
            .filter(|edge| edge.source.root_id() == root)
            .count();
        if leases == 0 {
            Ok(())
        } else {
            Err(BorrowError::SourceProtected { root, leases })
        }
    }

    pub(crate) fn contains(&self, lease: &BorrowLease) -> bool {
        self.edges.get(&lease.id()).is_some_and(|edge| {
            edge.source == *lease.source()
                && edge.holder == lease.holder()
                && edge.field == lease.field()
        })
    }

    pub(crate) fn holder_edge_count(&self, holder: BorrowHolder) -> usize {
        self.edges
            .values()
            .filter(|edge| edge.holder == holder)
            .count()
    }

    pub(crate) fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::arena_slot_state::ArenaSlotState;
    use cloud_terrastodon_registry::RuntimeValue;

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct TestBreadcrumbs {
        entries: Vec<String>,
    }

    #[derive(Clone, Debug, facet::Facet)]
    #[repr(C)]
    struct TestTab {
        name: String,
        breadcrumbs: TestBreadcrumbs,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: facet::Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn ordinary_cow_borrow_uses_an_edge_without_allocating_a_view_slot() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(String::from("borrowed value")))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let allocated_before_borrow = arena.allocated_slot_count();
        let mut graph = BorrowGraph::default();

        let lease = graph
            .borrow(&arena, ValueAddress::root(source), request, "value")
            .unwrap();

        assert_eq!(lease.source(), &ValueAddress::root(source));
        assert_eq!(lease.holder(), BorrowHolder::Builder(request));
        assert_eq!(lease.field(), "value");
        assert!(graph.protects_root(source));
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(
            arena.allocated_slot_count(),
            allocated_before_borrow,
            "a borrow edge must not allocate a view slot"
        );
        assert!(matches!(
            arena.slot(source).map(|slot| slot.state()),
            Some(ArenaSlotState::Ready(_))
        ));

        graph.release(lease).unwrap();
        assert!(!graph.protects_root(source));
    }

    #[test]
    fn nested_value_borrow_protects_the_complete_source_root() {
        let mut arena = Arena::default();
        let source_root = arena
            .insert_ready(runtime(TestTab {
                name: "admins".to_owned(),
                breadcrumbs: TestBreadcrumbs { entries: vec![] },
            }))
            .unwrap();
        let borrower = arena.reserve_builder().unwrap();
        let name = ValueAddress::root(source_root).child(
            crate::object_explorer::value_path::ValuePathSegment::Field("name".to_owned()),
        );
        let mut graph = BorrowGraph::default();

        let lease = graph.borrow(&arena, name, borrower, "name").unwrap();

        assert!(graph.protects_root(source_root));
        graph.release(lease).unwrap();
        assert!(!graph.protects_root(source_root));
    }

    #[test]
    fn pending_invocation_receives_the_same_unique_lease() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(String::from("borrow me")))
            .unwrap();
        let borrower = arena.reserve_builder().unwrap();
        let pending = arena.insert_pending().unwrap();
        let mut graph = BorrowGraph::default();
        let mut lease = graph
            .borrow(&arena, ValueAddress::root(source), borrower, "value")
            .unwrap();
        let lease_id = lease.id();

        graph.transfer_to_pending(&mut lease, pending).unwrap();

        assert_eq!(lease.id(), lease_id);
        assert_eq!(lease.holder(), BorrowHolder::PendingInvocation(pending));
        assert!(graph.protects_root(source));
        graph.release(lease).unwrap();
        assert!(!graph.protects_root(source));
    }

    #[test]
    fn mutation_conflict_is_typed_and_holder_transfers_are_visible() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(String::from("borrow me")))
            .unwrap();
        let builder = arena.reserve_builder().unwrap();
        let ready = arena
            .insert_ready(runtime(String::from("borrower")))
            .unwrap();
        let mut graph = BorrowGraph::default();
        let mut lease = graph
            .borrow(&arena, ValueAddress::root(source), builder, "value")
            .unwrap();

        assert_eq!(
            graph.ensure_root_unprotected(source),
            Err(BorrowError::SourceProtected {
                root: source,
                leases: 1
            })
        );
        graph.transfer_to_ready(&mut lease, ready).unwrap();
        assert!(graph.contains(&lease));
        assert_eq!(graph.holder_edge_count(BorrowHolder::ReadyValue(ready)), 1);

        graph.release(lease).unwrap();
        assert_eq!(graph.ensure_root_unprotected(source), Ok(()));
    }
}
