use crate::StructuredLogRecord;
use crate::format_record;
use crate::terminal_log_buffer_state::TerminalLogBufferState;
use crate::weak_terminal_log_buffer::WeakTerminalLogBuffer;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

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

    pub(crate) fn from_state(inner: Arc<Mutex<TerminalLogBufferState>>) -> Self {
        Self {
            inner,
            debug_capture: None,
        }
    }

    pub(crate) fn downgrade(&self) -> WeakTerminalLogBuffer {
        WeakTerminalLogBuffer::new(Arc::downgrade(&self.inner))
    }

    /// Attach a bounded, independently filtered Debug-through-Error capture.
    ///
    /// Configure this before installing tracing. The capture is for a separate
    /// presentation surface; it never changes stderr's filter or exit replay.
    pub fn with_debug_capture(mut self, capacity: usize) -> Self {
        let capture = Self {
            inner: Arc::new(Mutex::new(TerminalLogBufferState {
                records: Default::default(),
                first_index: 0,
                capacity: Some(capacity.max(1)),
            })),
            debug_capture: None,
        };
        self.debug_capture = Some(Arc::new(capture));
        self
    }

    pub fn records_since(&self, cursor: &mut usize) -> Vec<StructuredLogRecord> {
        let state = self.lock_state();
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
        let removed = {
            let mut state = self.lock_state();
            let count = cursor
                .saturating_sub(state.first_index)
                .min(state.records.len());
            let removed: Vec<_> = state.records.drain(..count).collect();
            state.first_index += count;
            removed
        };
        // Releasing the final report can invoke an error's destructor, which
        // may emit another log. Do not run it under the buffer's lock.
        drop(removed);
    }

    pub fn replay_to<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        // Formatting a retained error can run user-provided Display code and
        // emit more logs. Snapshot first so neither that code nor the writer
        // executes under the buffer's lock.
        for record in self.records_since(&mut 0) {
            writeln!(writer, "{}", format_record(&record))?;
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
        let removed = {
            let mut state = self.lock_state();
            state.records.push_back(record);
            // Capacity is fixed at construction, so one insertion can exceed
            // the bound by at most one record.
            if state
                .capacity
                .is_some_and(|capacity| state.records.len() > capacity)
            {
                state.first_index += 1;
                state.records.pop_front()
            } else {
                None
            }
        };
        drop(removed);
    }
}

impl TerminalLogBuffer {
    fn lock_state(&self) -> MutexGuard<'_, TerminalLogBufferState> {
        // The buffer owns all mutations and invokes no user code while changing
        // its records or cursor. Recover retained records after poisoning.
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
    use crate::DiagnosticReport;
    use crate::TerminalEventLayer;
    use std::fmt;
    use tracing_subscriber::layer::SubscriberExt;

    #[derive(Debug)]
    struct LoggingDropError;

    impl fmt::Display for LoggingDropError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("Synthetic retained error")
        }
    }

    impl std::error::Error for LoggingDropError {}

    impl Drop for LoggingDropError {
        fn drop(&mut self) {
            tracing::warn!("Emitted while dropping a retained error");
        }
    }

    fn report_record() -> StructuredLogRecord {
        StructuredLogRecord {
            timestamp: std::time::SystemTime::UNIX_EPOCH,
            level: crate::StructuredLogLevel::Error,
            target: Arc::from("test"),
            message: Arc::from("Retained error"),
            fields: vec![(
                "error".to_owned(),
                crate::StructuredLogValue::Report(DiagnosticReport::new(eyre::Report::new(
                    LoggingDropError,
                ))),
            )],
            spans: Vec::new(),
            file: None,
            line: None,
        }
    }

    #[test]
    fn dropping_the_buffer_releases_reports_that_retain_their_subscriber() {
        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)))
            .with(tracing_error::ErrorLayer::default());
        let dispatch = tracing::Dispatch::new(subscriber);
        let retained_subscriber = dispatch.downgrade();
        let retained_report = tracing::dispatcher::with_default(&dispatch, || {
            let span = tracing::info_span!("retained_report_context");
            let _entered = span.enter();
            let report =
                DiagnosticReport::new(eyre::Report::new(tracing_error::TracedError::from(
                    std::io::Error::other("Synthetic captured failure"),
                )));
            tracing::error!(error = report.as_error(), "Buffered diagnostic");
            Arc::downgrade(&report.report)
        });
        drop(dispatch);
        assert!(retained_subscriber.upgrade().is_some());
        let records = buffer.records_since(&mut 0);
        assert_eq!(records.len(), 1);
        drop(buffer);
        assert!(
            retained_report.upgrade().is_some(),
            "a cloned record must still own its shared diagnostic"
        );
        drop(records);
        assert!(retained_report.upgrade().is_none());
        assert!(retained_subscriber.upgrade().is_none());
    }

    #[test]
    fn acknowledging_a_report_allows_its_destructor_to_emit_logs() {
        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)));
        tracing::subscriber::with_default(subscriber, || {
            buffer.push(report_record());
            let mut cursor = 0;
            assert_eq!(buffer.records_since(&mut cursor).len(), 1);
            buffer.acknowledge_through(cursor);
            let records = buffer.records_since(&mut cursor);
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0].message.as_ref(),
                "Emitted while dropping a retained error"
            );
        });
    }

    #[test]
    fn evicting_a_report_allows_its_destructor_to_emit_logs() {
        let buffer = TerminalLogBuffer::new()
            .with_debug_capture(1)
            .debug_capture()
            .expect("debug capture must be enabled");
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)));
        tracing::subscriber::with_default(subscriber, || {
            buffer.push(report_record());
            // Push directly to exercise destruction outside the dispatcher,
            // whose scoped reentrancy guard can suppress nested log events.
            buffer.push(StructuredLogRecord {
                timestamp: std::time::SystemTime::UNIX_EPOCH,
                level: crate::StructuredLogLevel::Info,
                target: Arc::from("test"),
                message: Arc::from("Eviction trigger"),
                fields: Vec::new(),
                spans: Vec::new(),
                file: None,
                line: None,
            });
            let records = buffer.records_since(&mut 0);
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0].message.as_ref(),
                "Emitted while dropping a retained error"
            );
        });
    }

    #[test]
    fn report_rendering_can_emit_logs_during_replay() {
        #[derive(Debug)]
        struct LoggingError;

        impl fmt::Display for LoggingError {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                tracing::warn!("Emitted while rendering a retained error");
                formatter.write_str("Synthetic error detail")
            }
        }

        impl std::error::Error for LoggingError {}

        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)));
        tracing::subscriber::with_default(subscriber, || {
            let report = DiagnosticReport::new(eyre::Report::new(LoggingError));
            tracing::error!(error = report.as_error(), "Retained error");
            let mut output = Vec::new();
            buffer.replay_to(&mut output).unwrap();
            assert!(
                String::from_utf8(output)
                    .unwrap()
                    .contains("Synthetic error detail")
            );
            assert!(buffer.records_since(&mut 0).iter().any(|record| {
                record.message.as_ref() == "Emitted while rendering a retained error"
            }));
        });
    }

    #[test]
    fn a_panicking_replay_writer_does_not_discard_or_break_buffered_logs() {
        struct PanickingWriter;

        impl Write for PanickingWriter {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                panic!("synthetic replay writer panic");
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)));
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("retained before replay");
            assert!(std::panic::catch_unwind(|| buffer.replay_to(PanickingWriter)).is_err());
            tracing::info!("retained after replay");

            let mut cursor = 0;
            let records = buffer.records_since(&mut cursor);
            let messages: Vec<_> = records
                .iter()
                .map(|record| record.message.as_ref())
                .collect();
            assert_eq!(
                messages,
                ["retained before replay", "retained after replay"]
            );
            buffer.acknowledge_through(cursor);
            assert!(buffer.records_since(&mut cursor).is_empty());
        });
    }

    #[test]
    fn acknowledging_printed_logs_keeps_events_arriving_during_output() {
        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry()
            .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)));
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
