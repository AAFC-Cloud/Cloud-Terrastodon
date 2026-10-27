use super::arena_slot_state::ArenaSlotState;
use super::slot_id::SlotId;
use facet::Shape;

pub(crate) struct ArenaSlot {
    id: SlotId,
    state: ArenaSlotState,
    // Retain the declared or last materialized shape through lifecycle changes,
    // without treating an unfinished or consumed root as an available value.
    shape: Option<&'static Shape>,
}

impl ArenaSlot {
    pub(crate) fn new(id: SlotId, state: ArenaSlotState) -> Self {
        let shape = state.ready_value().map(|value| value.shape());
        Self { id, state, shape }
    }

    pub(crate) const fn id(&self) -> SlotId {
        self.id
    }

    pub(crate) const fn state(&self) -> &ArenaSlotState {
        &self.state
    }

    pub(crate) const fn shape(&self) -> Option<&'static Shape> {
        self.shape
    }

    /// Arena validates the lifecycle and owns revision changes for this update.
    pub(super) fn set_unmaterialized_shape(&mut self, shape: &'static Shape) {
        debug_assert!(matches!(
            self.state,
            ArenaSlotState::Building | ArenaSlotState::Pending
        ));
        self.shape = Some(shape);
    }

    pub(crate) fn replace_state(&mut self, state: ArenaSlotState) -> ArenaSlotState {
        if let Some(value) = state.ready_value() {
            self.shape = Some(value.shape());
        }
        std::mem::replace(&mut self.state, state)
    }
}
