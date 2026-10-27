use crate::StructuredLogRecord;
use crate::format_record;
use crate::terminal_log_buffer_state::TerminalLogBufferState;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

/// Structured terminal output retained while a terminal owner suppresses stderr.
#[derive(Clone)]
pub struct TerminalLogBuffer {
    inner: Arc<Mutex<TerminalLogBufferState>>,
    debug_capture: Option<Arc<TerminalLogBuffer>>,
}

impl TerminalLogBuffer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TerminalLogBufferState {
                records: Default::default(),
                first_index: 0,
                capacity: None,
            })),
            debug_capture: None,
        }
    }

    /// Attach a bounded, independently filtered Debug-through-Error capture.
    ///
    /// Configure this before installing tracing. The capture is for a separate
    /// presentation surface; it never changes stderr's filter or exit replay.
    pub fn with_debug_capture(mut self, capacity: usize) -> Self {
        let capture = Self::new();
        capture
            .inner
            .lock()
            .expect("terminal log buffer poisoned")
            .capacity = Some(capacity.max(1));
        self.debug_capture = Some(Arc::new(capture));
        self
    }

    pub fn records_since(&self, cursor: &mut usize) -> Vec<StructuredLogRecord> {
        let state = self.inner.lock().expect("terminal log buffer poisoned");
        let start = cursor
            .saturating_sub(state.first_index)
            .min(state.records.len());
        *cursor = state.first_index + state.records.len();
        state.records.iter().skip(start).cloned().collect()
    }

    /// Format buffered events exactly as the tracing layer prints them while
    /// stderr has no terminal owner. Uses the same cursor and acknowledgement
    /// contract as `records_since`, so inline output is never replayed on exit.
    pub fn formatted_records_since(&self, cursor: &mut usize) -> Vec<String> {
        self.records_since(cursor)
            .iter()
            .map(format_record)
            .collect()
    }

    /// Read the independently captured records, if enabled at application startup.
    pub fn captured_records_since(&self, cursor: &mut usize) -> Vec<StructuredLogRecord> {
        self.debug_capture
            .as_ref()
            .map(|capture| capture.records_since(cursor))
            .unwrap_or_default()
    }

    /// Release records already printed by an inline terminal, preventing exit replay.
    ///
    /// Call only after successful output using the cursor returned by
    /// `records_since`. Events received after that read remain available.
    pub fn acknowledge_through(&self, cursor: usize) {
        let mut state = self.inner.lock().expect("terminal log buffer poisoned");
        let count = cursor
            .saturating_sub(state.first_index)
            .min(state.records.len());
        state.records.drain(..count);
        state.first_index += count;
    }

    pub fn replay_to<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        let state = self.inner.lock().expect("terminal log buffer poisoned");
        for record in &state.records {
            writeln!(writer, "{}", format_record(record))?;
        }
        Ok(())
    }

    pub fn replay_to_stderr(&self) {
        let _ = self.replay_to(std::io::stderr());
    }

    pub(crate) fn debug_capture(&self) -> Option<Self> {
        self.debug_capture.as_deref().cloned()
    }

    pub(crate) fn push(&self, record: StructuredLogRecord) {
        let mut state = self.inner.lock().expect("terminal log buffer poisoned");
        state.records.push_back(record);
        if let Some(capacity) = state.capacity {
            while state.records.len() > capacity {
                state.records.pop_front();
                state.first_index += 1;
            }
        }
    }
}

impl Default for TerminalLogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TerminalEventLayer;
    use tracing_subscriber::layer::SubscriberExt;

    #[test]
    fn acknowledging_printed_logs_keeps_events_arriving_during_output() {
        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(buffer.clone(), Arc::new(|| true)));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("printed");
            let mut cursor = 0;
            let formatted = buffer.formatted_records_since(&mut cursor);
            assert_eq!(formatted.len(), 1);
            assert!(formatted[0].contains("printed"));
            assert!(formatted[0].contains("terminal_log_buffer.rs:"));
            tracing::info!("arrived while printing");
            buffer.acknowledge_through(cursor);
            // A stale acknowledgement must not delete more recent events.
            buffer.acknowledge_through(cursor);
            let remaining = buffer.records_since(&mut cursor);
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining[0].message.as_ref(), "arrived while printing");
            let mut replay = Vec::new();
            buffer.replay_to(&mut replay).unwrap();
            let replay = String::from_utf8(replay).unwrap();
            assert!(!replay.contains("printed"));
            assert!(replay.contains("arrived while printing"));
            buffer.acknowledge_through(cursor);
            assert!(buffer.records_since(&mut 0).is_empty());
        });
    }
}
