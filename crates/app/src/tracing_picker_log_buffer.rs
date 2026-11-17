use cloud_terrastodon_user_input::PickerLogBuffer;
use cloud_terrastodon_user_input::PickerLogLevel;
use cloud_terrastodon_user_input::PickerLogRecord;
use cloud_terrastodon_user_input::PickerLogSpan;
use teamy_tracing::capture::LogBuffer;
use teamy_tracing::capture::LogLevel;
use teamy_tracing::capture::LogRecord;

/// Adapt shared capture records to the terminal picker's presentation contract.
pub(crate) struct TracingPickerLogBuffer {
    /// Terminal output queued while an interactive view owns the terminal.
    /// The picker acknowledges records after printing them, removing them from this backlog.
    pub buffer: LogBuffer,
    /// Optional bounded diagnostic history collected regardless of terminal ownership.
    /// Reading the history or acknowledging terminal output leaves these records intact.
    pub capture: Option<LogBuffer>,
}

impl PickerLogBuffer for TracingPickerLogBuffer {
    fn records_since(&self, cursor: &mut usize) -> Vec<PickerLogRecord> {
        self.buffer
            .records_since(cursor)
            .into_iter()
            .map(picker_log_record)
            .collect()
    }

    fn formatted_records_since(&self, cursor: &mut usize) -> Vec<String> {
        self.buffer.formatted_records_since(cursor)
    }

    fn acknowledge_through(&self, cursor: usize) {
        self.buffer.acknowledge_through(cursor);
    }

    fn captured_records_since(&self, cursor: &mut usize) -> Vec<PickerLogRecord> {
        self.capture
            .as_ref()
            .map(|capture| {
                capture
                    .records_since(cursor)
                    .into_iter()
                    .map(picker_log_record)
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn picker_log_record(record: LogRecord) -> PickerLogRecord {
    PickerLogRecord {
        level: match record.level {
            LogLevel::Debug | LogLevel::Trace => PickerLogLevel::Debug,
            LogLevel::Info => PickerLogLevel::Info,
            LogLevel::Warn => PickerLogLevel::Warn,
            LogLevel::Error => PickerLogLevel::Error,
        },
        message: record.message,
        target: record.target,
        timestamp: record.timestamp,
        fields: record
            .fields
            .into_iter()
            .map(|(name, value)| (name.into(), value.to_string().into()))
            .collect(),
        spans: record
            .spans
            .into_iter()
            .map(|span| PickerLogSpan {
                name: span.name,
                fields: span
                    .fields
                    .into_iter()
                    .map(|(name, value)| (name.into(), value.to_string().into()))
                    .collect(),
            })
            .collect(),
        file: record.file,
        line: record.line,
    }
}
