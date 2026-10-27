use crate::StructuredLogRecord;
use std::collections::VecDeque;

/// Retained log records and their absolute sequence position.
pub(crate) struct TerminalLogBufferState {
    pub records: VecDeque<StructuredLogRecord>,
    pub first_index: usize,
    pub capacity: Option<usize>,
}
