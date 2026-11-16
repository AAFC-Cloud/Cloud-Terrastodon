use crate::PickerLogRecord;
use std::time::Instant;

pub(super) struct PickerToast {
    pub(super) record: PickerLogRecord,
    pub(super) expires_at: Instant,
}
