use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use cloud_terrastodon_registry::RuntimeValue;
use facet::{Shape, Type, UserType};
use facet_reflect::Partial;

use super::arena::{Arena, ArenaError};
use super::arena_address_source::ArenaAddressSource;
use super::arena_slot_state::ArenaSlotState;
use super::borrow_graph::{BorrowError, BorrowGraph};
use super::borrow_lease::{BorrowHolder, BorrowLease};
use super::borrow_materializer::{materialize_borrow, validate_borrow};
use super::field_binding::FieldBinding;
use super::slot_id::SlotId;

#[derive(Debug)]
struct BuilderField {
    name: String,
    shape: &'static Shape,
    has_default: bool,
    binding: FieldBinding,
}

impl BuilderField {
    fn from_facet(field: &'static facet::Field) -> Self {
        let has_default = field.has_default();
        Self {
            name: field.effective_name().to_owned(),
            shape: field.shape(),
            has_default,
            binding: if has_default {
                FieldBinding::Default
            } else {
                FieldBinding::Unset
            },
        }
    }
}

#[derive(Debug)]
enum BuilderKind {
    Scalar(Option<RuntimeValue>),
    Struct(Vec<BuilderField>),
    Enum {
        variant: Option<usize>,
        fields: Vec<BuilderField>,
    },
}

/// Construction state for one Building arena root.
#[derive(Debug)]
pub(crate) struct ValueBuilder {
    shape: &'static Shape,
    kind: BuilderKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuilderTransition {
    Building,
    Ready,
}

#[derive(Debug)]
pub(crate) enum ValueBuilderError {
    DuplicateBuilder(SlotId),
    UnknownBuilder(SlotId),
    BuilderShapeUnset(SlotId),
    BuilderShapeAlreadySet(SlotId),
    SlotIsNotBuilding(SlotId),
    NotStructured,
    NotScalar,
    NotEnum,
    UnknownField(usize),
    FieldNotPendingProducer {
        slot: SlotId,
        field: usize,
    },
    ProducerCompletionUnresolved {
        slot: SlotId,
        field: usize,
        binding: &'static str,
    },
    UnknownVariant(usize),
    InvalidDefault(String),
    ShapeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
    Incomplete,
    DuplicateMoveSource(SlotId),
    MoveSourceBorrowed(SlotId),
    MoveConflictsWithBorrow(SlotId),
    MissingBorrowLease {
        slot: SlotId,
        field: usize,
    },
    UnexpectedBorrowLease {
        slot: SlotId,
        field: usize,
    },
    InvalidBorrowLeaseTransfer {
        slot: SlotId,
        expected: &'static str,
        actual: &'static str,
    },
    Reflection(String),
    Arena(ArenaError),
    Borrow(BorrowError),
}

impl fmt::Display for ValueBuilderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateBuilder(slot) => write!(formatter, "slot {slot} already has a builder"),
            Self::UnknownBuilder(slot) => write!(formatter, "slot {slot} has no builder"),
            Self::BuilderShapeUnset(slot) => {
                write!(formatter, "slot {slot} has no selected shape")
            }
            Self::BuilderShapeAlreadySet(slot) => {
                write!(formatter, "slot {slot} already has a selected shape")
            }
            Self::SlotIsNotBuilding(slot) => write!(formatter, "slot {slot} is not Building"),
            Self::NotStructured => write!(formatter, "the builder does not have reflected fields"),
            Self::NotScalar => write!(formatter, "the builder is not a scalar/general value"),
            Self::NotEnum => write!(formatter, "the builder is not an enum"),
            Self::UnknownField(index) => write!(formatter, "field {index} does not exist"),
            Self::FieldNotPendingProducer { slot, field } => write!(
                formatter,
                "field {field} of slot {slot} is not waiting for a producer"
            ),
            Self::ProducerCompletionUnresolved {
                slot,
                field,
                binding,
            } => write!(
                formatter,
                "producer completion for field {field} of slot {slot} cannot remain {binding}"
            ),
            Self::UnknownVariant(index) => write!(formatter, "variant {index} does not exist"),
            Self::InvalidDefault(field) => write!(formatter, "field {field} has no default"),
            Self::ShapeMismatch {
                field,
                expected,
                actual,
            } => write!(formatter, "field {field} expects {expected}, not {actual}"),
            Self::Incomplete => write!(formatter, "the builder is incomplete"),
            Self::DuplicateMoveSource(slot) => {
                write!(formatter, "slot {slot} cannot be moved into two fields")
            }
            Self::MoveSourceBorrowed(slot) => {
                write!(formatter, "slot {slot} cannot move while it is borrowed")
            }
            Self::MoveConflictsWithBorrow(slot) => write!(
                formatter,
                "slot {slot} cannot be moved while this builder also borrows from it"
            ),
            Self::MissingBorrowLease { slot, field } => write!(
                formatter,
                "field {field} of slot {slot} has a borrowed pointer without an active lease"
            ),
            Self::UnexpectedBorrowLease { slot, field } => write!(
                formatter,
                "field {field} of slot {slot} has a lease but no borrowed binding"
            ),
            Self::InvalidBorrowLeaseTransfer {
                slot,
                expected,
                actual,
            } => write!(
                formatter,
                "slot {slot} must be {expected} to transfer borrow leases, but it is {actual}"
            ),
            Self::Reflection(message) => formatter.write_str(message),
            Self::Arena(error) => error.fmt(formatter),
            Self::Borrow(error) => error.fmt(formatter),
        }
    }
}

impl Error for ValueBuilderError {}

impl From<ArenaError> for ValueBuilderError {
    fn from(value: ArenaError) -> Self {
        Self::Arena(value)
    }
}

impl From<BorrowError> for ValueBuilderError {
    fn from(value: BorrowError) -> Self {
        Self::Borrow(value)
    }
}

impl ValueBuilder {
    pub(crate) fn new(shape: &'static Shape) -> Self {
        let kind = match shape.ty {
            Type::User(UserType::Struct(object)) => {
                BuilderKind::Struct(object.fields.iter().map(BuilderField::from_facet).collect())
            }
            Type::User(UserType::Enum(_)) => BuilderKind::Enum {
                variant: None,
                fields: Vec::new(),
            },
            _ => BuilderKind::Scalar(None),
        };
        Self { shape, kind }
    }

    pub(crate) const fn shape(&self) -> &'static Shape {
        self.shape
    }

    pub(crate) fn field_count(&self) -> usize {
        self.fields().map_or(0, <[_]>::len)
    }

    pub(crate) fn field_binding(&self, index: usize) -> Option<&FieldBinding> {
        self.fields()?.get(index).map(|field| &field.binding)
    }

    pub(crate) fn field_name(&self, index: usize) -> Option<&str> {
        self.fields()?.get(index).map(|field| field.name.as_str())
    }

    pub(crate) fn field_shape(&self, index: usize) -> Option<&'static Shape> {
        self.fields()?.get(index).map(|field| field.shape)
    }

    pub(crate) fn field_has_default(&self, index: usize) -> Option<bool> {
        self.fields()?.get(index).map(|field| field.has_default)
    }

    pub(crate) fn selected_variant(&self) -> Option<usize> {
        match &self.kind {
            BuilderKind::Enum { variant, .. } => *variant,
            BuilderKind::Scalar(_) | BuilderKind::Struct(_) => None,
        }
    }

    pub(crate) fn scalar_is_set(&self) -> bool {
        matches!(&self.kind, BuilderKind::Scalar(Some(_)))
    }

    pub(crate) fn set_field(
        &mut self,
        index: usize,
        binding: FieldBinding,
    ) -> Result<(), ValueBuilderError> {
        let field = self
            .fields_mut()
            .ok_or(ValueBuilderError::NotStructured)?
            .get_mut(index)
            .ok_or(ValueBuilderError::UnknownField(index))?;
        if matches!(binding, FieldBinding::Default) && !field.has_default {
            return Err(ValueBuilderError::InvalidDefault(field.name.clone()));
        }
        if let FieldBinding::InlineOwned(value) = &binding
            && !owned_source_is_compatible(field.shape, value.shape())
        {
            return Err(ValueBuilderError::ShapeMismatch {
                field: field.name.clone(),
                expected: cloud_terrastodon_registry::describe_shape(field.shape).to_owned(),
                actual: cloud_terrastodon_registry::describe_shape(value.shape()).to_owned(),
            });
        }
        field.binding = binding;
        Ok(())
    }

    pub(crate) fn set_scalar(&mut self, value: RuntimeValue) -> Result<(), ValueBuilderError> {
        let BuilderKind::Scalar(current) = &mut self.kind else {
            return Err(ValueBuilderError::NotScalar);
        };
        if !self.shape.is_shape(value.shape()) {
            return Err(ValueBuilderError::ShapeMismatch {
                field: "value".to_owned(),
                expected: cloud_terrastodon_registry::describe_shape(self.shape).to_owned(),
                actual: cloud_terrastodon_registry::describe_shape(value.shape()).to_owned(),
            });
        }
        *current = Some(value);
        Ok(())
    }

    pub(crate) fn select_variant(&mut self, index: usize) -> Result<(), ValueBuilderError> {
        let Type::User(UserType::Enum(object)) = self.shape.ty else {
            return Err(ValueBuilderError::NotEnum);
        };
        let selected = object
            .variants
            .get(index)
            .ok_or(ValueBuilderError::UnknownVariant(index))?;
        let BuilderKind::Enum { variant, fields } = &mut self.kind else {
            return Err(ValueBuilderError::NotEnum);
        };
        *variant = Some(index);
        *fields = selected
            .data
            .fields
            .iter()
            .map(BuilderField::from_facet)
            .collect();
        Ok(())
    }

    pub(crate) fn is_complete(&self) -> bool {
        match &self.kind {
            BuilderKind::Scalar(value) => value.is_some(),
            BuilderKind::Struct(fields) => fields.iter().all(|field| field.binding.is_resolved()),
            BuilderKind::Enum { variant, fields } => {
                variant.is_some() && fields.iter().all(|field| field.binding.is_resolved())
            }
        }
    }

    fn fields(&self) -> Option<&[BuilderField]> {
        match &self.kind {
            BuilderKind::Struct(fields) | BuilderKind::Enum { fields, .. } => Some(fields),
            BuilderKind::Scalar(_) => None,
        }
    }

    fn fields_mut(&mut self) -> Option<&mut Vec<BuilderField>> {
        match &mut self.kind {
            BuilderKind::Struct(fields) | BuilderKind::Enum { fields, .. } => Some(fields),
            BuilderKind::Scalar(_) => None,
        }
    }

    fn materialize(
        self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        borrower: SlotId,
        leases_by_field: BTreeMap<usize, BorrowLease>,
        inherited_leases: Vec<BorrowLease>,
    ) -> Result<BuilderMaterialization, ValueBuilderError> {
        if !self.is_complete() {
            return Err(ValueBuilderError::Incomplete);
        }
        match self.kind {
            BuilderKind::Scalar(Some(value))
                if leases_by_field.is_empty() && inherited_leases.is_empty() =>
            {
                Ok(BuilderMaterialization {
                    value,
                    leases: Vec::new(),
                    moved_sources: Vec::new(),
                })
            }
            BuilderKind::Scalar(Some(_)) => {
                let field = leases_by_field.keys().next().copied().unwrap_or(0);
                release_leases(
                    borrow_graph,
                    leases_by_field
                        .into_values()
                        .chain(inherited_leases)
                        .collect(),
                );
                Err(ValueBuilderError::UnexpectedBorrowLease {
                    slot: borrower,
                    field,
                })
            }
            BuilderKind::Scalar(None) => Err(ValueBuilderError::Incomplete),
            BuilderKind::Struct(fields) => materialize_fields(
                self.shape,
                None,
                fields,
                arena,
                borrow_graph,
                borrower,
                leases_by_field,
                inherited_leases,
            ),
            BuilderKind::Enum {
                variant: Some(variant),
                fields,
            } => materialize_fields(
                self.shape,
                Some(variant),
                fields,
                arena,
                borrow_graph,
                borrower,
                leases_by_field,
                inherited_leases,
            ),
            BuilderKind::Enum { variant: None, .. } => Err(ValueBuilderError::Incomplete),
        }
    }
}

struct BuilderMaterialization {
    value: RuntimeValue,
    leases: Vec<BorrowLease>,
    moved_sources: Vec<SlotId>,
}

enum BuilderEntry {
    ShapeUnset,
    Defined(ValueBuilder),
}

/// Builders and leases are engine metadata keyed by ownership-bearing SlotId.
#[derive(Default)]
pub(crate) struct BuilderStore {
    builders: BTreeMap<SlotId, BuilderEntry>,
    builder_leases: BTreeMap<SlotId, BTreeMap<usize, BorrowLease>>,
    builder_inherited_leases: BTreeMap<SlotId, BTreeMap<usize, Vec<BorrowLease>>>,
    ready_leases: BTreeMap<SlotId, Vec<BorrowLease>>,
    pending_leases: BTreeMap<SlotId, Vec<BorrowLease>>,
}

impl BuilderStore {
    pub(crate) fn reserve(&mut self, arena: &mut Arena) -> Result<SlotId, ValueBuilderError> {
        let slot = arena.reserve_builder()?;
        let replaced = self.builders.insert(slot, BuilderEntry::ShapeUnset);
        debug_assert!(replaced.is_none(), "Arena returned a fresh SlotId");
        Ok(slot)
    }

    pub(crate) fn set_shape_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        shape: &'static Shape,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.assert_building(arena, slot)?;
        let entry = self
            .builders
            .get_mut(&slot)
            .ok_or(ValueBuilderError::UnknownBuilder(slot))?;
        if matches!(entry, BuilderEntry::Defined(_)) {
            return Err(ValueBuilderError::BuilderShapeAlreadySet(slot));
        }
        *entry = BuilderEntry::Defined(ValueBuilder::new(shape));
        self.finalize_if_complete(arena, borrow_graph, slot)
    }

    pub(crate) fn create_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        shape: &'static Shape,
    ) -> Result<(SlotId, BuilderTransition), ValueBuilderError> {
        let slot = self.reserve(arena)?;
        let transition = self.set_shape_and_finalize(arena, borrow_graph, slot, shape)?;
        Ok((slot, transition))
    }

    pub(crate) fn insert_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        builder: ValueBuilder,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.assert_building(arena, slot)?;
        if self.builders.contains_key(&slot) {
            return Err(ValueBuilderError::DuplicateBuilder(slot));
        }
        self.builders.insert(slot, BuilderEntry::Defined(builder));
        self.finalize_if_complete(arena, borrow_graph, slot)
    }

    pub(crate) fn set_field_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        index: usize,
        binding: FieldBinding,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.replace_field_and_finalize(arena, borrow_graph, slot, index, binding)
    }

    pub(crate) fn unset_field_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        index: usize,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.set_field_and_finalize(arena, borrow_graph, slot, index, FieldBinding::Unset)
    }

    pub(crate) fn complete_pending_field_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        index: usize,
        binding: FieldBinding,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.assert_building(arena, slot)?;
        if !binding.is_resolved() {
            return Err(ValueBuilderError::ProducerCompletionUnresolved {
                slot,
                field: index,
                binding: binding.label(),
            });
        }
        let builder = self.defined_builder(slot)?;
        if !matches!(
            builder.field_binding(index),
            Some(FieldBinding::PendingProducer)
        ) {
            return Err(ValueBuilderError::FieldNotPendingProducer { slot, field: index });
        }
        self.replace_field_and_finalize(arena, borrow_graph, slot, index, binding)
    }

    pub(crate) fn select_variant_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        variant: usize,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.assert_building(arena, slot)?;
        self.defined_builder_mut(slot)?.select_variant(variant)?;
        self.release_builder_leases(borrow_graph, slot);
        self.finalize_if_complete(arena, borrow_graph, slot)
    }

    pub(crate) fn set_scalar_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        value: RuntimeValue,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.assert_building(arena, slot)?;
        self.defined_builder_mut(slot)?.set_scalar(value)?;
        self.finalize_if_complete(arena, borrow_graph, slot)
    }

    pub(crate) fn builder(&self, slot: SlotId) -> Option<&ValueBuilder> {
        match self.builders.get(&slot) {
            Some(BuilderEntry::Defined(builder)) => Some(builder),
            Some(BuilderEntry::ShapeUnset) | None => None,
        }
    }

    pub(crate) fn contains(&self, slot: SlotId) -> bool {
        self.builders.contains_key(&slot)
    }

    pub(crate) fn abandon(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
    ) -> Result<(), ValueBuilderError> {
        self.assert_building(arena, slot)?;
        if !self.builders.contains_key(&slot) {
            return Err(ValueBuilderError::UnknownBuilder(slot));
        }
        arena.abandon_builder(slot)?;
        self.builders.remove(&slot);
        self.release_builder_leases(borrow_graph, slot);
        Ok(())
    }

    pub(crate) fn leases(&self, slot: SlotId) -> &[BorrowLease] {
        self.ready_leases.get(&slot).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn builder_lease_count(&self, slot: SlotId) -> usize {
        self.builder_leases.get(&slot).map_or(0, BTreeMap::len)
            + self
                .builder_inherited_leases
                .get(&slot)
                .map_or(0, |fields| fields.values().map(Vec::len).sum())
    }

    pub(crate) fn take_leases(&mut self, slot: SlotId) -> Vec<BorrowLease> {
        self.ready_leases.remove(&slot).unwrap_or_default()
    }

    pub(crate) fn delete(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
    ) -> Result<(), ValueBuilderError> {
        if self.contains(slot) {
            return self.abandon(arena, borrow_graph, slot);
        }
        borrow_graph.ensure_root_unprotected(slot)?;
        arena.delete(slot)?;
        release_leases(borrow_graph, self.take_leases(slot));
        Ok(())
    }

    pub(crate) fn transfer_ready_leases_to_pending(
        &mut self,
        arena: &Arena,
        borrow_graph: &mut BorrowGraph,
        ready_slot: SlotId,
        pending_slot: SlotId,
    ) -> Result<usize, ValueBuilderError> {
        let ready_state = arena
            .slot(ready_slot)
            .ok_or(ArenaError::UnknownSlot(ready_slot))?
            .state()
            .label();
        if ready_state != "Consumed" {
            return Err(ValueBuilderError::InvalidBorrowLeaseTransfer {
                slot: ready_slot,
                expected: "Consumed",
                actual: ready_state,
            });
        }
        let pending_state = arena
            .slot(pending_slot)
            .ok_or(ArenaError::UnknownSlot(pending_slot))?
            .state()
            .label();
        if pending_state != "Pending" {
            return Err(ValueBuilderError::InvalidBorrowLeaseTransfer {
                slot: pending_slot,
                expected: "Pending",
                actual: pending_state,
            });
        }
        let mut leases = self.ready_leases.remove(&ready_slot).unwrap_or_default();
        for lease in &mut leases {
            if let Err(error) = borrow_graph.transfer_to_pending(lease, pending_slot) {
                for transferred in &mut leases {
                    if transferred.holder() == BorrowHolder::PendingInvocation(pending_slot) {
                        let _ = borrow_graph.transfer_to_ready(transferred, ready_slot);
                    }
                }
                self.ready_leases.insert(ready_slot, leases);
                return Err(error.into());
            }
        }
        let count = leases.len();
        if !leases.is_empty() {
            self.pending_leases
                .entry(pending_slot)
                .or_default()
                .append(&mut leases);
        }
        Ok(count)
    }

    pub(crate) fn clone_ready_leases_to_pending(
        &mut self,
        arena: &Arena,
        borrow_graph: &mut BorrowGraph,
        ready_slot: SlotId,
        pending_slot: SlotId,
    ) -> Result<usize, ValueBuilderError> {
        let ready_state = arena
            .slot(ready_slot)
            .ok_or(ArenaError::UnknownSlot(ready_slot))?
            .state()
            .label();
        if ready_state != "Ready" {
            return Err(ValueBuilderError::InvalidBorrowLeaseTransfer {
                slot: ready_slot,
                expected: "Ready",
                actual: ready_state,
            });
        }
        let pending_state = arena
            .slot(pending_slot)
            .ok_or(ArenaError::UnknownSlot(pending_slot))?
            .state()
            .label();
        if pending_state != "Pending" {
            return Err(ValueBuilderError::InvalidBorrowLeaseTransfer {
                slot: pending_slot,
                expected: "Pending",
                actual: pending_state,
            });
        }

        let mut duplicates = Vec::new();
        if let Some(source_leases) = self.ready_leases.get(&ready_slot) {
            for source_lease in source_leases {
                match borrow_graph.duplicate_for_pending(
                    arena,
                    source_lease,
                    pending_slot,
                    format!("cloned invocation of {ready_slot}"),
                ) {
                    Ok(lease) => duplicates.push(lease),
                    Err(error) => {
                        release_leases(borrow_graph, duplicates);
                        return Err(error.into());
                    }
                }
            }
        }
        let count = duplicates.len();
        if !duplicates.is_empty() {
            self.pending_leases
                .entry(pending_slot)
                .or_default()
                .append(&mut duplicates);
        }
        Ok(count)
    }

    fn finish_pending_leases(&mut self, borrow_graph: &mut BorrowGraph, pending_slot: SlotId) {
        let leases = self
            .pending_leases
            .remove(&pending_slot)
            .unwrap_or_default();
        release_leases(borrow_graph, leases);
    }

    pub(crate) fn pending_lease_count(&self, pending_slot: SlotId) -> usize {
        self.pending_leases.get(&pending_slot).map_or(0, Vec::len)
    }

    pub(crate) fn complete_pending(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        pending_slot: SlotId,
        value: RuntimeValue,
    ) -> Result<(), ValueBuilderError> {
        arena.set_ready(pending_slot, value)?;
        self.finish_pending_leases(borrow_graph, pending_slot);
        Ok(())
    }

    pub(crate) fn fail_pending(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        pending_slot: SlotId,
        message: impl Into<String>,
    ) -> Result<(), ValueBuilderError> {
        arena.set_failed(pending_slot, message)?;
        self.finish_pending_leases(borrow_graph, pending_slot);
        Ok(())
    }

    pub(crate) fn cancel_pending(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        pending_slot: SlotId,
    ) -> Result<(), ValueBuilderError> {
        arena.cancel_pending(pending_slot)?;
        self.finish_pending_leases(borrow_graph, pending_slot);
        Ok(())
    }

    pub(crate) fn release_all_leases(&mut self, borrow_graph: &mut BorrowGraph) {
        let builder_leases = std::mem::take(&mut self.builder_leases)
            .into_values()
            .flat_map(BTreeMap::into_values);
        let inherited_leases = std::mem::take(&mut self.builder_inherited_leases)
            .into_values()
            .flat_map(BTreeMap::into_values)
            .flatten();
        let ready_leases = std::mem::take(&mut self.ready_leases)
            .into_values()
            .flatten();
        let pending_leases = std::mem::take(&mut self.pending_leases)
            .into_values()
            .flatten();
        release_leases(
            borrow_graph,
            builder_leases
                .chain(inherited_leases)
                .chain(ready_leases)
                .chain(pending_leases)
                .collect(),
        );
    }

    fn replace_field_and_finalize(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
        index: usize,
        binding: FieldBinding,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        self.assert_building(arena, slot)?;
        self.validate_field_binding(arena, borrow_graph, slot, index, &binding)?;

        let field_name = self
            .defined_builder(slot)?
            .fields()
            .and_then(|fields| fields.get(index))
            .ok_or(ValueBuilderError::UnknownField(index))?
            .name
            .clone();
        let new_lease = match &binding {
            FieldBinding::BorrowFrom(address) => {
                Some(borrow_graph.borrow(arena, address.clone(), slot, field_name.clone())?)
            }
            _ => None,
        };
        let mut new_inherited_leases = Vec::new();
        if let FieldBinding::CloneFrom(address) = &binding
            && let Some(source_leases) = self.ready_leases.get(&address.root_id())
        {
            for source_lease in source_leases {
                match borrow_graph.duplicate_for_builder(
                    arena,
                    source_lease,
                    slot,
                    format!("{field_name} cloned from {address}"),
                ) {
                    Ok(lease) => new_inherited_leases.push(lease),
                    Err(error) => {
                        if let Some(lease) = new_lease {
                            let _ = borrow_graph.release(lease);
                        }
                        release_leases(borrow_graph, new_inherited_leases);
                        return Err(error.into());
                    }
                }
            }
        }

        if let Err(error) = self.defined_builder_mut(slot)?.set_field(index, binding) {
            if let Some(lease) = new_lease {
                let _ = borrow_graph.release(lease);
            }
            release_leases(borrow_graph, new_inherited_leases);
            return Err(error);
        }

        let old_lease = self
            .builder_leases
            .get_mut(&slot)
            .and_then(|leases| leases.remove(&index));
        let old_inherited_leases = self
            .builder_inherited_leases
            .get_mut(&slot)
            .and_then(|leases| leases.remove(&index))
            .unwrap_or_default();
        if let Some(lease) = new_lease {
            self.builder_leases
                .entry(slot)
                .or_default()
                .insert(index, lease);
        }
        if !new_inherited_leases.is_empty() {
            self.builder_inherited_leases
                .entry(slot)
                .or_default()
                .insert(index, new_inherited_leases);
        }
        if self
            .builder_leases
            .get(&slot)
            .is_some_and(BTreeMap::is_empty)
        {
            self.builder_leases.remove(&slot);
        }
        if self
            .builder_inherited_leases
            .get(&slot)
            .is_some_and(BTreeMap::is_empty)
        {
            self.builder_inherited_leases.remove(&slot);
        }
        if let Some(lease) = old_lease {
            borrow_graph.release(lease)?;
        }
        release_leases(borrow_graph, old_inherited_leases);

        self.finalize_if_complete(arena, borrow_graph, slot)
    }

    fn release_builder_leases(&mut self, borrow_graph: &mut BorrowGraph, slot: SlotId) {
        let direct = self
            .builder_leases
            .remove(&slot)
            .into_iter()
            .flat_map(BTreeMap::into_values);
        let inherited = self
            .builder_inherited_leases
            .remove(&slot)
            .into_iter()
            .flat_map(BTreeMap::into_values)
            .flatten();
        let leases = direct.chain(inherited).collect();
        release_leases(borrow_graph, leases);
    }

    fn finalize_if_complete(
        &mut self,
        arena: &mut Arena,
        borrow_graph: &mut BorrowGraph,
        slot: SlotId,
    ) -> Result<BuilderTransition, ValueBuilderError> {
        let is_complete = match self.builders.get(&slot) {
            Some(BuilderEntry::Defined(builder)) => builder.is_complete(),
            Some(BuilderEntry::ShapeUnset) => {
                return Ok(BuilderTransition::Building);
            }
            None => return Err(ValueBuilderError::UnknownBuilder(slot)),
        };
        if !is_complete {
            return Ok(BuilderTransition::Building);
        }

        let entry = self
            .builders
            .remove(&slot)
            .expect("builder existence checked above");
        let BuilderEntry::Defined(builder) = entry else {
            unreachable!("only a complete defined builder reaches materialization")
        };
        let leases_by_field = self.builder_leases.remove(&slot).unwrap_or_default();
        let inherited_leases = self
            .builder_inherited_leases
            .remove(&slot)
            .into_iter()
            .flat_map(BTreeMap::into_values)
            .flatten()
            .collect();
        match builder.materialize(arena, borrow_graph, slot, leases_by_field, inherited_leases) {
            Ok(BuilderMaterialization {
                value,
                mut leases,
                moved_sources,
            }) => {
                for source in moved_sources {
                    leases.extend(self.ready_leases.remove(&source).unwrap_or_default());
                }
                for lease in &mut leases {
                    if let Err(error) = borrow_graph.transfer_to_ready(lease, slot) {
                        drop(value);
                        release_leases(borrow_graph, leases);
                        let _ = arena.set_failed(slot, error.to_string());
                        return Err(error.into());
                    }
                }
                if let Err(error) = arena.set_ready(slot, value) {
                    release_leases(borrow_graph, leases);
                    return Err(error.into());
                }
                if !leases.is_empty() {
                    self.ready_leases.insert(slot, leases);
                }
                Ok(BuilderTransition::Ready)
            }
            Err(error) => {
                let _ = arena.set_failed(slot, error.to_string());
                Err(error)
            }
        }
    }

    fn assert_building(&self, arena: &Arena, slot: SlotId) -> Result<(), ValueBuilderError> {
        match arena.slot(slot).map(|slot| slot.state()) {
            Some(ArenaSlotState::Building) => Ok(()),
            _ => Err(ValueBuilderError::SlotIsNotBuilding(slot)),
        }
    }

    fn defined_builder_mut(
        &mut self,
        slot: SlotId,
    ) -> Result<&mut ValueBuilder, ValueBuilderError> {
        match self.builders.get_mut(&slot) {
            Some(BuilderEntry::Defined(builder)) => Ok(builder),
            Some(BuilderEntry::ShapeUnset) => Err(ValueBuilderError::BuilderShapeUnset(slot)),
            None => Err(ValueBuilderError::UnknownBuilder(slot)),
        }
    }

    fn defined_builder(&self, slot: SlotId) -> Result<&ValueBuilder, ValueBuilderError> {
        match self.builders.get(&slot) {
            Some(BuilderEntry::Defined(builder)) => Ok(builder),
            Some(BuilderEntry::ShapeUnset) => Err(ValueBuilderError::BuilderShapeUnset(slot)),
            None => Err(ValueBuilderError::UnknownBuilder(slot)),
        }
    }

    pub(crate) fn validate_field_binding(
        &self,
        arena: &Arena,
        borrow_graph: &BorrowGraph,
        slot: SlotId,
        index: usize,
        binding: &FieldBinding,
    ) -> Result<(), ValueBuilderError> {
        let builder = self.defined_builder(slot)?;
        let fields = builder.fields().ok_or(ValueBuilderError::NotStructured)?;
        let field = fields
            .get(index)
            .ok_or(ValueBuilderError::UnknownField(index))?;
        match binding {
            FieldBinding::CloneFrom(address) => {
                let value = ArenaAddressSource::new(arena)
                    .resolve(address)
                    .map_err(|error| {
                        ValueBuilderError::Reflection(format!(
                            "clone source {address} does not resolve: {error}"
                        ))
                    })?;
                ensure_owned_source_shape(field, value.shape())?;
            }
            FieldBinding::BorrowFrom(address) => {
                validate_borrow(arena, address, field.shape).map_err(|error| {
                    ValueBuilderError::Reflection(format!(
                        "could not borrow {address} into {}: {error}",
                        field.name
                    ))
                })?;
                if fields.iter().enumerate().any(|(other_index, other)| {
                    other_index != index
                        && matches!(
                            &other.binding,
                            FieldBinding::MoveFrom(source)
                                if *source == address.root_id()
                        )
                }) {
                    return Err(ValueBuilderError::MoveConflictsWithBorrow(
                        address.root_id(),
                    ));
                }
            }
            FieldBinding::MoveFrom(source) => {
                if borrow_graph.protects_root(*source) {
                    return Err(ValueBuilderError::MoveSourceBorrowed(*source));
                }
                ensure_owned_source_shape(field, arena.resolve_root(*source)?.shape())?;
                for (other_index, other) in fields.iter().enumerate() {
                    if other_index == index {
                        continue;
                    }
                    match &other.binding {
                        FieldBinding::MoveFrom(other_source) if other_source == source => {
                            return Err(ValueBuilderError::DuplicateMoveSource(*source));
                        }
                        FieldBinding::BorrowFrom(address) if address.root_id() == *source => {
                            return Err(ValueBuilderError::MoveConflictsWithBorrow(*source));
                        }
                        _ => {}
                    }
                }
            }
            FieldBinding::Default
            | FieldBinding::InlineOwned(_)
            | FieldBinding::Unset
            | FieldBinding::PendingProducer => {}
        }
        Ok(())
    }
}

fn ensure_shape(field: &BuilderField, actual: &'static Shape) -> Result<(), ValueBuilderError> {
    if field.shape.is_shape(actual) {
        return Ok(());
    }
    Err(ValueBuilderError::ShapeMismatch {
        field: field.name.clone(),
        expected: cloud_terrastodon_registry::describe_shape(field.shape).to_owned(),
        actual: cloud_terrastodon_registry::describe_shape(actual).to_owned(),
    })
}

fn ensure_owned_source_shape(
    field: &BuilderField,
    actual: &'static Shape,
) -> Result<(), ValueBuilderError> {
    if owned_source_is_compatible(field.shape, actual) {
        return Ok(());
    }
    Err(ValueBuilderError::ShapeMismatch {
        field: field.name.clone(),
        expected: cloud_terrastodon_registry::describe_shape(field.shape).to_owned(),
        actual: cloud_terrastodon_registry::describe_shape(actual).to_owned(),
    })
}

fn owned_source_is_compatible(expected: &'static Shape, actual: &'static Shape) -> bool {
    expected.is_shape(actual) || RuntimeValue::can_own_pointee(expected, actual)
}

fn adapt_owned_source(
    expected: &'static Shape,
    field: &str,
    value: RuntimeValue,
) -> Result<RuntimeValue, ValueBuilderError> {
    if expected.is_shape(value.shape()) {
        return Ok(value);
    }
    let actual = value.shape();
    if RuntimeValue::can_own_pointee(expected, actual) {
        return RuntimeValue::from_owned_pointee(expected, value).map_err(|error| {
            ValueBuilderError::Reflection(format!(
                "could not wrap owned {} for {field}: {error}",
                cloud_terrastodon_registry::describe_shape(actual)
            ))
        });
    }
    Err(ValueBuilderError::ShapeMismatch {
        field: field.to_owned(),
        expected: cloud_terrastodon_registry::describe_shape(expected).to_owned(),
        actual: cloud_terrastodon_registry::describe_shape(actual).to_owned(),
    })
}

enum PreparedField {
    Owned {
        index: usize,
        name: String,
        value: RuntimeValue,
    },
    Move {
        index: usize,
        name: String,
        source: SlotId,
        field_shape: &'static Shape,
    },
}

fn materialize_fields(
    shape: &'static Shape,
    variant: Option<usize>,
    fields: Vec<BuilderField>,
    arena: &mut Arena,
    borrow_graph: &mut BorrowGraph,
    borrower: SlotId,
    mut leases_by_field: BTreeMap<usize, BorrowLease>,
    inherited_leases: Vec<BorrowLease>,
) -> Result<BuilderMaterialization, ValueBuilderError> {
    let mut move_sources = BTreeSet::new();
    let borrow_roots = fields
        .iter()
        .filter_map(|field| match &field.binding {
            FieldBinding::BorrowFrom(address) => Some(address.root_id()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for field in &fields {
        let FieldBinding::MoveFrom(source) = &field.binding else {
            continue;
        };
        let source = *source;
        if !move_sources.insert(source) {
            return Err(ValueBuilderError::DuplicateMoveSource(source));
        }
        if borrow_graph.protects_root(source) {
            return Err(ValueBuilderError::MoveSourceBorrowed(source));
        }
        if borrow_roots.contains(&source) {
            return Err(ValueBuilderError::MoveConflictsWithBorrow(source));
        }
        let value = arena.resolve_root(source)?;
        ensure_owned_source_shape(field, value.shape())?;
    }

    let moved_sources = move_sources.into_iter().collect::<Vec<_>>();
    let mut leases = inherited_leases;
    let source = ArenaAddressSource::new(arena);
    let prepared = (|| {
        let mut prepared = Vec::new();
        for (index, field) in fields.into_iter().enumerate() {
            let value = match field.binding {
                FieldBinding::Default => None,
                FieldBinding::InlineOwned(value) => {
                    Some(adapt_owned_source(field.shape, &field.name, value)?)
                }
                FieldBinding::CloneFrom(address) => {
                    let value = source.resolve(&address).map_err(|error| {
                        ValueBuilderError::Reflection(format!(
                            "clone source {address} does not resolve: {error}"
                        ))
                    })?;
                    let cloned = RuntimeValue::clone_from_peek(value.peek()).map_err(|error| {
                        ValueBuilderError::Reflection(format!(
                            "could not clone {address} into {}: {error}",
                            field.name
                        ))
                    })?;
                    Some(adapt_owned_source(field.shape, &field.name, cloned)?)
                }
                FieldBinding::BorrowFrom(address) => {
                    let lease = leases_by_field.remove(&index).ok_or(
                        ValueBuilderError::MissingBorrowLease {
                            slot: borrower,
                            field: index,
                        },
                    )?;
                    if lease.source() != &address {
                        leases.push(lease);
                        return Err(ValueBuilderError::MissingBorrowLease {
                            slot: borrower,
                            field: index,
                        });
                    }
                    let pointer = materialize_borrow(arena, borrow_graph, &lease, field.shape)
                        .map_err(|error| {
                            ValueBuilderError::Reflection(format!(
                                "could not borrow {address} into {}: {error}",
                                field.name
                            ))
                        });
                    leases.push(lease);
                    let pointer = pointer?;
                    Some(pointer)
                }
                FieldBinding::MoveFrom(source) => {
                    prepared.push(PreparedField::Move {
                        index,
                        name: field.name,
                        source,
                        field_shape: field.shape,
                    });
                    continue;
                }
                FieldBinding::Unset | FieldBinding::PendingProducer => {
                    return Err(ValueBuilderError::Incomplete);
                }
            };
            if let Some(value) = value {
                if !field.shape.is_shape(value.shape()) {
                    return Err(ValueBuilderError::ShapeMismatch {
                        field: field.name,
                        expected: cloud_terrastodon_registry::describe_shape(field.shape)
                            .to_owned(),
                        actual: cloud_terrastodon_registry::describe_shape(value.shape())
                            .to_owned(),
                    });
                }
                prepared.push(PreparedField::Owned {
                    index,
                    name: field.name,
                    value,
                });
            }
        }
        if let Some(field) = leases_by_field.keys().next().copied() {
            leases.extend(std::mem::take(&mut leases_by_field).into_values());
            return Err(ValueBuilderError::UnexpectedBorrowLease {
                slot: borrower,
                field,
            });
        }
        Ok(prepared)
    })();
    let prepared = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            leases.extend(leases_by_field.into_values());
            release_leases(borrow_graph, leases);
            return Err(error);
        }
    };
    drop(source);

    let built = RuntimeValue::build_with(shape, |mut partial| {
        if let Some(variant) = variant {
            partial = partial.select_nth_variant(variant).map_err(|error| {
                eyre::eyre!("could not select variant {variant} while building: {error}")
            })?;
        }
        for field in prepared {
            match field {
                PreparedField::Owned { index, name, value } => {
                    partial = set_runtime_field(partial, index, &name, value)?;
                }
                PreparedField::Move {
                    index,
                    name,
                    source,
                    field_shape,
                } => {
                    let value = arena.consume(source).map_err(|error| {
                        eyre::eyre!("{name}: could not consume move source {source}: {error}")
                    })?;
                    let value = adapt_owned_source(field_shape, &name, value)
                        .map_err(|error| eyre::eyre!(error.to_string()))?;
                    partial = set_runtime_field(partial, index, &name, value)?;
                }
            }
        }
        Ok(partial)
    })
    .map_err(|error| ValueBuilderError::Reflection(error.to_string()));

    match built {
        Ok(value) => Ok(BuilderMaterialization {
            value,
            leases,
            moved_sources,
        }),
        Err(error) => {
            release_leases(borrow_graph, leases);
            Err(error)
        }
    }
}

fn set_runtime_field(
    partial: Partial<'static, false>,
    index: usize,
    field: &str,
    value: RuntimeValue,
) -> eyre::Result<Partial<'static, false>> {
    let partial = partial
        .begin_nth_field(index)
        .map_err(|error| eyre::eyre!("{field}: could not begin field: {error}"))?;
    let peek = value.peek();
    // SAFETY: ValueBuilder validated the destination field shape against this
    // RuntimeValue before materialization. Facet transfers the initialized
    // value into the active field; on success the source allocation is freed
    // without dropping moved bytes, while on failure value still drops
    // normally.
    match unsafe { partial.set_from_peek(&peek) } {
        Ok(partial) => {
            value.deallocate_after_move();
            partial
                .end()
                .map_err(|error| eyre::eyre!("{field}: could not finish field: {error}"))
        }
        Err(error) => Err(eyre::eyre!("{field}: could not set field: {error}")),
    }
}

fn release_leases(borrow_graph: &mut BorrowGraph, leases: Vec<BorrowLease>) {
    for lease in leases {
        let _ = borrow_graph.release(lease);
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use arbitrary::{Arbitrary, Unstructured};
    use cloud_terrastodon_azure_devops::{
        AzureDevOpsOrganizationUrl, AzureDevOpsProjectMemberListRequest,
    };
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;

    use super::*;
    use crate::object_explorer::breadcrumbs::Breadcrumbs;
    use crate::object_explorer::produce_json_request::ProduceJsonRequest;
    use crate::object_explorer::tab::Tab;
    use crate::object_explorer::value_address::ValueAddress;
    use crate::object_explorer::value_path::ValuePathSegment;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct TestBorrowSource {
        value: String,
    }

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct TestBorrowContainer {
        nested: TestBorrowSource,
    }

    #[derive(Debug, Facet)]
    #[repr(C)]
    struct NonCloneLabel {
        value: String,
    }

    #[derive(Debug, Facet)]
    #[repr(C)]
    struct TestBorrowThenUncloneable<'a> {
        source: Cow<'a, TestBorrowSource>,
        label: NonCloneLabel,
    }

    #[derive(Clone, Debug, Facet)]
    #[facet(traits(Clone))]
    #[repr(C)]
    struct TestBorrowRequest<'a> {
        source: Cow<'a, TestBorrowSource>,
    }

    #[derive(Debug, Facet)]
    #[repr(C)]
    struct TestBorrowEnvelope<'a> {
        request: TestBorrowRequest<'a>,
    }

    #[derive(Debug, Facet)]
    #[repr(C)]
    struct TestBorrowThenLabel<'a> {
        source: Cow<'a, TestBorrowSource>,
        label: String,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct BindingFixture {
        moved: String,
        cloned: String,
        inline: String,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn completed_builder_is_finalized_before_cow_borrow() {
        let mut arena = Arena::default();
        let source_slot = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();

        assert_eq!(
            builders
                .insert_and_finalize(
                    &mut arena,
                    &mut borrows,
                    source_slot,
                    ValueBuilder::new(TestBorrowSource::SHAPE),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    source_slot,
                    0,
                    FieldBinding::InlineOwned(runtime(String::from("ready now"))),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert!(builders.builder(source_slot).is_none());
        assert!(arena.ready_value(source_slot).is_some());

        // The command returned only after Ready replaced Building, so the
        // ordinary borrow path succeeds immediately without a render/cache
        // normalization pass.
        let borrower = arena.reserve_builder().unwrap();
        let mut lease = borrows
            .borrow(&arena, ValueAddress::root(source_slot), borrower, "source")
            .expect("newly completed source is immediately borrowable");
        let borrowed = RuntimeValue::from_borrowed_pointer(
            <Cow<'static, TestBorrowSource>>::SHAPE,
            arena.ready_value(source_slot).unwrap().peek(),
        )
        .expect("Facet can form the actual Cow representation");
        assert!(
            borrowed
                .peek()
                .shape()
                .is_shape(<Cow<'static, TestBorrowSource>>::SHAPE)
        );
        drop(borrowed);

        let pending = arena.insert_pending().unwrap();
        borrows.transfer_to_pending(&mut lease, pending).unwrap();
        borrows.release(lease).unwrap();
    }

    #[test]
    fn cow_field_clones_a_projected_pointee_into_owned_form() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowContainer {
                nested: TestBorrowSource {
                    value: "projected clone".to_owned(),
                },
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        let nested = ValueAddress::root(source).child(ValuePathSegment::Field("nested".to_owned()));

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::CloneFrom(nested),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert!(arena.ready_value(source).is_some());
        assert_eq!(borrows.edge_count(), 0);
        let request = arena
            .ready_value(request)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<TestBorrowRequest<'static>>()
            .unwrap()
            .downcast::<TestBorrowRequest<'static>>()
            .unwrap();
        assert!(matches!(request.source, Cow::Owned(_)));
        assert_eq!(request.source.value, "projected clone");
    }

    #[test]
    fn cow_field_moves_an_owned_pointee_root_into_owned_form() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "root move".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::MoveFrom(source),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert!(matches!(
            arena.slot(source).map(|slot| slot.state()),
            Some(ArenaSlotState::Consumed)
        ));
        assert_eq!(borrows.edge_count(), 0);
        let request = arena
            .ready_value(request)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<TestBorrowRequest<'static>>()
            .unwrap()
            .downcast::<TestBorrowRequest<'static>>()
            .unwrap();
        assert!(matches!(request.source, Cow::Owned(_)));
        assert_eq!(request.source.value, "root move");
    }

    #[test]
    fn projected_breadcrumbs_clone_into_produce_json_without_tab_special_case() {
        let mut arena = Arena::default();
        let tab = arena
            .insert_ready(runtime(Tab::new("source", Breadcrumbs::default())))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(ProduceJsonRequest::SHAPE),
            )
            .unwrap();
        let breadcrumbs =
            ValueAddress::root(tab).child(ValuePathSegment::Field("breadcrumbs".to_owned()));

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::CloneFrom(breadcrumbs),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    1,
                    FieldBinding::InlineOwned(runtime(String::from("admins.json"))),
                )
                .unwrap(),
            BuilderTransition::Ready
        );

        let request_value = arena
            .ready_value(request)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<ProduceJsonRequest>()
            .unwrap()
            .downcast::<ProduceJsonRequest>()
            .unwrap();
        assert!(request_value.breadcrumbs().is_empty());
        assert_eq!(request_value.filename(), "admins.json");
        assert!(arena.ready_value(tab).is_some());
        assert_eq!(borrows.edge_count(), 0);
    }

    #[test]
    fn failed_finalization_releases_borrows_prepared_for_earlier_fields() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "borrowed".to_owned(),
            }))
            .unwrap();
        let uncloneable = arena
            .insert_ready(runtime(NonCloneLabel {
                value: "cannot clone through Facet".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();

        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowThenUncloneable<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();

        let error = builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                1,
                FieldBinding::CloneFrom(ValueAddress::root(uncloneable)),
            )
            .expect_err("the later field has no reflected Clone operation");

        assert!(matches!(
            error,
            ValueBuilderError::Reflection(ref message)
                if message.contains("could not clone")
        ));
        assert_eq!(borrows.edge_count(), 0);
        assert!(!borrows.protects_root(source));
        assert!(matches!(
            arena.slot(request).map(|slot| slot.state()),
            Some(ArenaSlotState::Failed(_))
        ));
    }

    #[test]
    fn move_consumes_one_root_while_clone_and_inline_preserve_ownership() {
        let mut arena = Arena::default();
        let moved = arena
            .insert_ready(runtime(String::from("moved value")))
            .unwrap();
        let cloned = arena
            .insert_ready(runtime(String::from("cloned value")))
            .unwrap();
        let target = arena.reserve_builder().unwrap();
        let allocated_roots = arena.allocated_slot_count();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                target,
                ValueBuilder::new(BindingFixture::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    target,
                    0,
                    FieldBinding::move_from_address(ValueAddress::root(moved)).unwrap(),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    target,
                    1,
                    FieldBinding::CloneFrom(ValueAddress::root(cloned)),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    target,
                    2,
                    FieldBinding::InlineOwned(runtime(String::from("inline value"))),
                )
                .unwrap(),
            BuilderTransition::Ready
        );

        assert!(matches!(
            arena.slot(moved).map(|slot| slot.state()),
            Some(ArenaSlotState::Consumed)
        ));
        assert!(arena.ready_value(cloned).is_some());
        assert_eq!(
            arena.allocated_slot_count(),
            allocated_roots,
            "inline and cloned field values do not allocate view/root slots"
        );
        let value = arena
            .ready_value(target)
            .unwrap()
            .try_clone()
            .unwrap()
            .into_box::<BindingFixture>()
            .unwrap()
            .downcast::<BindingFixture>()
            .unwrap();
        assert_eq!(
            value.as_ref(),
            &BindingFixture {
                moved: "moved value".to_owned(),
                cloned: "cloned value".to_owned(),
                inline: "inline value".to_owned(),
            }
        );
    }

    #[test]
    fn invalid_move_conflicts_leave_source_and_builder_unchanged() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(String::from("still owned")))
            .unwrap();
        let external_borrower = arena.reserve_builder().unwrap();
        let target = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                target,
                ValueBuilder::new(BindingFixture::SHAPE),
            )
            .unwrap();
        let lease = borrows
            .borrow(
                &arena,
                ValueAddress::root(source),
                external_borrower,
                "value",
            )
            .unwrap();

        let error = builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                target,
                0,
                FieldBinding::MoveFrom(source),
            )
            .expect_err("an actively borrowed source cannot move");
        assert!(matches!(
            error,
            ValueBuilderError::MoveSourceBorrowed(slot) if slot == source
        ));
        assert!(arena.ready_value(source).is_some());
        assert!(matches!(
            builders.builder(target).unwrap().field_binding(0),
            Some(FieldBinding::Unset)
        ));

        borrows.release(lease).unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                target,
                0,
                FieldBinding::MoveFrom(source),
            )
            .unwrap();
        let duplicate = builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                target,
                1,
                FieldBinding::MoveFrom(source),
            )
            .expect_err("one root cannot move into two destination fields");
        assert!(matches!(
            duplicate,
            ValueBuilderError::DuplicateMoveSource(slot) if slot == source
        ));
        assert!(arena.ready_value(source).is_some());
        assert!(matches!(
            builders.builder(target).unwrap().field_binding(1),
            Some(FieldBinding::Unset)
        ));
    }

    #[test]
    fn builder_borrow_lease_releases_on_replacement_unset_and_abandon() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "protected while building".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowThenLabel<'static>>::SHAPE),
            )
            .unwrap();

        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        assert_eq!(builders.builder_lease_count(request), 1);
        assert!(borrows.protects_root(source));
        assert!(matches!(
            builders.delete(&mut arena, &mut borrows, source),
            Err(ValueBuilderError::Borrow(BorrowError::SourceProtected {
                root,
                leases: 1
            })) if root == source
        ));

        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::InlineOwned(runtime(Cow::<'static, TestBorrowSource>::Owned(
                    TestBorrowSource {
                        value: "replacement".to_owned(),
                    },
                ))),
            )
            .unwrap();
        assert_eq!(builders.builder_lease_count(request), 0);
        assert!(!borrows.protects_root(source));

        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        builders
            .unset_field_and_finalize(&mut arena, &mut borrows, request, 0)
            .unwrap();
        assert_eq!(builders.builder_lease_count(request), 0);
        assert!(!borrows.protects_root(source));

        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        builders.abandon(&mut arena, &mut borrows, request).unwrap();
        assert_eq!(borrows.edge_count(), 0);
        assert!(!borrows.protects_root(source));
    }

    #[test]
    fn ready_borrower_locks_source_until_borrower_is_deleted() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "ready source".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let allocated = arena.allocated_slot_count();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::BorrowFrom(ValueAddress::root(source)),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert_eq!(arena.allocated_slot_count(), allocated);
        assert_eq!(builders.leases(request).len(), 1);
        assert_eq!(
            borrows.holder_edge_count(BorrowHolder::ReadyValue(request)),
            1
        );
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());

        builders.delete(&mut arena, &mut borrows, request).unwrap();
        assert_eq!(borrows.edge_count(), 0);
        builders.delete(&mut arena, &mut borrows, source).unwrap();
    }

    #[test]
    fn cow_field_borrows_nested_address_and_locks_complete_source_root() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowContainer {
                nested: TestBorrowSource {
                    value: "nested source".to_owned(),
                },
            }))
            .unwrap();
        let nested = ValueAddress::root(source).child(ValuePathSegment::Field("nested".to_owned()));
        let request = arena.reserve_builder().unwrap();
        let allocated = arena.allocated_slot_count();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::BorrowFrom(nested.clone()),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert_eq!(arena.allocated_slot_count(), allocated);
        assert_eq!(builders.leases(request)[0].source(), &nested);
        assert!(borrows.protects_root(source));
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());

        builders.delete(&mut arena, &mut borrows, request).unwrap();
        builders.delete(&mut arena, &mut borrows, source).unwrap();
    }

    #[test]
    fn moving_a_ready_borrower_transfers_its_leases_to_the_destination_root() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "move borrower".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let envelope = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                envelope,
                ValueBuilder::new(<TestBorrowEnvelope<'static>>::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    envelope,
                    0,
                    FieldBinding::MoveFrom(request),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert!(matches!(
            arena.slot(request).map(|slot| slot.state()),
            Some(ArenaSlotState::Consumed)
        ));
        assert!(builders.leases(request).is_empty());
        assert_eq!(builders.leases(envelope).len(), 1);
        assert_eq!(
            borrows.holder_edge_count(BorrowHolder::ReadyValue(envelope)),
            1
        );
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());

        builders.delete(&mut arena, &mut borrows, envelope).unwrap();
        builders.delete(&mut arena, &mut borrows, source).unwrap();
    }

    #[test]
    fn cloning_a_ready_borrower_duplicates_its_source_protection() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "clone borrower".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let envelope = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                envelope,
                ValueBuilder::new(<TestBorrowEnvelope<'static>>::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    envelope,
                    0,
                    FieldBinding::CloneFrom(ValueAddress::root(request)),
                )
                .unwrap(),
            BuilderTransition::Ready
        );
        assert_eq!(borrows.edge_count(), 2);
        assert_eq!(builders.leases(request).len(), 1);
        assert_eq!(builders.leases(envelope).len(), 1);

        builders.delete(&mut arena, &mut borrows, request).unwrap();
        assert_eq!(borrows.edge_count(), 1);
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());
        builders.delete(&mut arena, &mut borrows, envelope).unwrap();
        builders.delete(&mut arena, &mut borrows, source).unwrap();
    }

    #[test]
    fn cow_source_is_locked_until_pending_invocation_finishes() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "pending source".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();

        let pending = arena.insert_pending().unwrap();
        assert!(matches!(
            builders.transfer_ready_leases_to_pending(
                &arena,
                &mut borrows,
                request,
                pending
            ),
            Err(ValueBuilderError::InvalidBorrowLeaseTransfer {
                slot,
                expected: "Consumed",
                actual: "Ready"
            }) if slot == request
        ));
        assert!(borrows.protects_root(source));
        let moved_request = arena.consume(request).unwrap();
        assert_eq!(
            builders
                .transfer_ready_leases_to_pending(&arena, &mut borrows, request, pending)
                .unwrap(),
            1
        );
        assert_eq!(builders.pending_lease_count(pending), 1);
        assert_eq!(
            borrows.holder_edge_count(BorrowHolder::PendingInvocation(pending)),
            1
        );
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());

        // A fake deterministic invocation owns the moved request until it
        // produces its completion packet.
        drop(moved_request);
        builders
            .complete_pending(
                &mut arena,
                &mut borrows,
                pending,
                runtime(String::from("fake result")),
            )
            .unwrap();
        assert_eq!(builders.pending_lease_count(pending), 0);
        assert_eq!(borrows.edge_count(), 0);
        builders.delete(&mut arena, &mut borrows, source).unwrap();
    }

    #[test]
    fn cloned_pending_invocation_gets_an_independent_lease() {
        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "cloned pending source".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();

        let cloned_request = arena.ready_value(request).unwrap().try_clone().unwrap();
        let pending = arena.insert_pending().unwrap();
        assert_eq!(
            builders
                .clone_ready_leases_to_pending(&arena, &mut borrows, request, pending)
                .unwrap(),
            1
        );
        assert_eq!(borrows.edge_count(), 2);
        assert_eq!(builders.leases(request).len(), 1);
        assert_eq!(builders.pending_lease_count(pending), 1);

        drop(cloned_request);
        builders
            .fail_pending(&mut arena, &mut borrows, pending, "fake clone failure")
            .unwrap();
        assert_eq!(borrows.edge_count(), 1);
        assert!(builders.delete(&mut arena, &mut borrows, source).is_err());
        builders.delete(&mut arena, &mut borrows, request).unwrap();
        builders.delete(&mut arena, &mut borrows, source).unwrap();
    }

    #[test]
    fn pending_failure_cancellation_and_session_drop_release_leases() {
        for cancel in [false, true] {
            let mut arena = Arena::default();
            let source = arena
                .insert_ready(runtime(TestBorrowSource {
                    value: "terminal pending path".to_owned(),
                }))
                .unwrap();
            let request = arena.reserve_builder().unwrap();
            let mut builders = BuilderStore::default();
            let mut borrows = BorrowGraph::default();
            builders
                .insert_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
                )
                .unwrap();
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::BorrowFrom(ValueAddress::root(source)),
                )
                .unwrap();
            let pending = arena.insert_pending().unwrap();
            let moved = arena.consume(request).unwrap();
            builders
                .transfer_ready_leases_to_pending(&arena, &mut borrows, request, pending)
                .unwrap();
            drop(moved);

            if cancel {
                builders
                    .cancel_pending(&mut arena, &mut borrows, pending)
                    .unwrap();
            } else {
                builders
                    .fail_pending(&mut arena, &mut borrows, pending, "fake failure")
                    .unwrap();
            }
            assert_eq!(borrows.edge_count(), 0);
            builders.delete(&mut arena, &mut borrows, source).unwrap();
        }

        let mut arena = Arena::default();
        let source = arena
            .insert_ready(runtime(TestBorrowSource {
                value: "session shutdown".to_owned(),
            }))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<TestBorrowRequest<'static>>::SHAPE),
            )
            .unwrap();
        builders
            .set_field_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                0,
                FieldBinding::BorrowFrom(ValueAddress::root(source)),
            )
            .unwrap();
        builders.release_all_leases(&mut borrows);
        assert_eq!(borrows.edge_count(), 0);
    }

    #[test]
    fn azure_devops_request_borrows_org_url_without_a_view_slot() {
        let bytes = (0_u8..=255).cycle().take(4096).collect::<Vec<_>>();
        let mut arbitrary = Unstructured::new(&bytes);
        let fake_request =
            AzureDevOpsProjectMemberListRequest::<'static>::arbitrary(&mut arbitrary)
                .expect("the registered arbitrary implementation supplies offline test data");
        let organization = fake_request.org_url.into_owned();
        let project = fake_request.project.into_owned();

        let mut arena = Arena::default();
        let organization = arena
            .insert_ready(runtime::<AzureDevOpsOrganizationUrl>(organization))
            .unwrap();
        let request = arena.reserve_builder().unwrap();
        let allocated_before_binding = arena.allocated_slot_count();
        let mut builders = BuilderStore::default();
        let mut borrows = BorrowGraph::default();
        builders
            .insert_and_finalize(
                &mut arena,
                &mut borrows,
                request,
                ValueBuilder::new(<AzureDevOpsProjectMemberListRequest<'static>>::SHAPE),
            )
            .unwrap();

        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    0,
                    FieldBinding::BorrowFrom(ValueAddress::root(organization)),
                )
                .unwrap(),
            BuilderTransition::Building
        );
        assert_eq!(
            builders
                .set_field_and_finalize(
                    &mut arena,
                    &mut borrows,
                    request,
                    1,
                    FieldBinding::InlineOwned(runtime(project)),
                )
                .unwrap(),
            BuilderTransition::Ready
        );

        assert_eq!(
            arena.allocated_slot_count(),
            allocated_before_binding,
            "borrowing a Cow field adds an edge, never a synthetic view slot"
        );
        assert_eq!(builders.leases(request).len(), 1);
        assert!(borrows.protects_root(organization));
        let org_field =
            ValueAddress::root(request).child(ValuePathSegment::Field("org_url".to_owned()));
        let reflected_org = ArenaAddressSource::new(&arena).resolve(&org_field).unwrap();
        let pointee = reflected_org
            .peek()
            .into_pointer()
            .unwrap()
            .borrow_inner()
            .unwrap();
        assert!(
            pointee.shape().is_shape(AzureDevOpsOrganizationUrl::SHAPE),
            "the ready request contains the ordinary reflected Cow pointer"
        );

        assert!(
            builders
                .delete(&mut arena, &mut borrows, organization)
                .is_err()
        );
        builders.delete(&mut arena, &mut borrows, request).unwrap();
        builders
            .delete(&mut arena, &mut borrows, organization)
            .unwrap();
    }
}
