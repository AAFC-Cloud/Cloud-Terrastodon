use super::arena_slot_state::ArenaSlotState;
use super::slot_id::SlotId;

pub(crate) struct ArenaSlot {
    id: SlotId,
    state: ArenaSlotState,
}

impl ArenaSlot {
    pub(crate) const fn new(id: SlotId, state: ArenaSlotState) -> Self {
        Self { id, state }
    }

    pub(crate) const fn id(&self) -> SlotId {
        self.id
    }

    pub(crate) const fn state(&self) -> &ArenaSlotState {
        &self.state
    }

    pub(crate) fn replace_state(&mut self, state: ArenaSlotState) -> ArenaSlotState {
        std::mem::replace(&mut self.state, state)
    }
}
