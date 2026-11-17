use crate::StructuredLogRecord;
use crate::TerminalLogBuffer;
use crate::terminal_log_buffer_state::TerminalLogBufferState;
use std::sync::Mutex;
use std::sync::Weak;

/// A nonowning capture writer that cannot keep its subscriber alive through a
/// retained report's span trace. The terminal session owns the retained state.
#[derive(Clone)]
pub(crate) struct WeakTerminalLogBuffer {
    inner: Weak<Mutex<TerminalLogBufferState>>,
}

impl WeakTerminalLogBuffer {
    pub(crate) fn new(inner: Weak<Mutex<TerminalLogBufferState>>) -> Self {
        Self { inner }
    }

    pub(crate) fn push(&self, record: StructuredLogRecord) {
        if let Some(inner) = self.inner.upgrade() {
            TerminalLogBuffer::from_state(inner).push(record);
        }
    }
}
