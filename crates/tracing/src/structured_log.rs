use crate::TerminalLogBuffer;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use color_eyre::owo_colors::Style;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::Subscriber;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::span::Attributes;
use tracing::span::Id;
use tracing::span::Record;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuredLogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl StructuredLogLevel {
    pub fn is_toast_level(self) -> bool {
        matches!(self, Self::Info | Self::Warn | Self::Error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuredSpan {
    pub name: Arc<str>,
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct StructuredLogRecord {
    pub timestamp: SystemTime,
    pub level: StructuredLogLevel,
    pub target: Arc<str>,
    pub message: Arc<str>,
    pub fields: Vec<(String, String)>,
    pub spans: Vec<StructuredSpan>,
    pub file: Option<Arc<str>>,
    pub line: Option<u32>,
}

pub type TerminalActivityProbe = Arc<dyn Fn() -> bool + Send + Sync>;

#[derive(Clone, Default)]
struct CapturedFields {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl CapturedFields {
    fn merge(&mut self, other: &Self) {
        if other.message.is_some() {
            self.message = other.message.clone();
        }
        for (name, value) in &other.fields {
            if let Some(existing) = self.fields.iter_mut().find(|(key, _)| key == name) {
                existing.1.clone_from(value);
            } else {
                self.fields.push((name.clone(), value.clone()));
            }
        }
    }

    fn record_value(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
            self.message = Some(value);
        } else {
            self.fields.push((field.name().to_string(), value));
        }
    }
}

impl Visit for CapturedFields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_value(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_value(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_value(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_value(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_value(field, value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record_value(field, value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.record_value(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_value(field, value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.record_value(field, value.to_string());
    }
}

#[derive(Clone)]
pub struct TerminalEventLayer {
    buffer: TerminalLogBuffer,
    active: TerminalActivityProbe,
    inactive_writer: InactiveLogWriter,
}

type InactiveLogWriter = Arc<dyn Fn(&StructuredLogRecord) + Send + Sync>;

impl TerminalEventLayer {
    pub fn new(buffer: TerminalLogBuffer, active: TerminalActivityProbe) -> Self {
        Self {
            buffer,
            active,
            inactive_writer: Arc::new(|record| {
                eprintln!("{}", format_record(record));
            }),
        }
    }

    /// Override the human-output sink used while no terminal owner is active.
    ///
    /// The application uses the default stderr sink. A caller that owns another
    /// presentation surface can inject a sink without changing the structured
    /// record path or the guarded buffering behavior.
    pub fn with_inactive_writer(
        mut self,
        writer: impl Fn(&StructuredLogRecord) + Send + Sync + 'static,
    ) -> Self {
        self.inactive_writer = Arc::new(writer);
        self
    }
}

impl<S> Layer<S> for TerminalEventLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut fields = CapturedFields::default();
        attrs.record(&mut fields);
        // The independent canvas capture and human-output layers can share a
        // registry. Both observe the same span fields; do not insert twice.
        let mut extensions = span.extensions_mut();
        if extensions.get_mut::<CapturedFields>().is_none() {
            extensions.insert(fields);
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        let mut extensions = span.extensions_mut();
        let Some(fields) = extensions.get_mut::<CapturedFields>() else {
            return;
        };
        let mut update = CapturedFields::default();
        values.record(&mut update);
        fields.merge(&update);
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let mut captured = CapturedFields::default();
        event.record(&mut captured);
        let spans = ctx
            .event_scope(event)
            .into_iter()
            .flat_map(|scope| scope.from_root())
            .map(|span| {
                let captured_fields = span
                    .extensions()
                    .get::<CapturedFields>()
                    .cloned()
                    .unwrap_or_default();
                let mut fields = captured_fields.fields;
                if let Some(message) = captured_fields.message {
                    fields.push(("message".to_string(), message));
                }
                StructuredSpan {
                    name: Arc::from(span.name()),
                    fields,
                }
            })
            .collect();
        let metadata = event.metadata();
        let record = StructuredLogRecord {
            timestamp: SystemTime::now(),
            level: level(metadata.level()),
            target: Arc::from(metadata.target()),
            message: Arc::from(
                captured
                    .message
                    .unwrap_or_else(|| metadata.name().to_string()),
            ),
            fields: captured.fields,
            spans,
            file: metadata.file().map(Arc::from),
            line: metadata.line(),
        };
        if (self.active)() {
            self.buffer.push(record);
        } else {
            (self.inactive_writer)(&record);
        }
    }
}

fn level(level: &tracing::Level) -> StructuredLogLevel {
    match *level {
        tracing::Level::ERROR => StructuredLogLevel::Error,
        tracing::Level::WARN => StructuredLogLevel::Warn,
        tracing::Level::INFO => StructuredLogLevel::Info,
        tracing::Level::DEBUG => StructuredLogLevel::Debug,
        tracing::Level::TRACE => StructuredLogLevel::Trace,
    }
}

pub fn format_record(record: &StructuredLogRecord) -> String {
    format_record_with_style(record, |text, style| {
        text.if_supports_color(Stream::Stderr, |text| text.style(style))
            .to_string()
    })
}

fn format_record_with_style(
    record: &StructuredLogRecord,
    styled: impl Fn(&str, Style) -> String,
) -> String {
    let mut event = format!(
        "  {} {}",
        format_level(record.level),
        escape_terminal_controls(&record.message)
    );
    for (name, value) in &record.fields {
        let _ = write!(
            event,
            ", {}: {}",
            escape_terminal_controls(name),
            escape_terminal_controls(value)
        );
    }

    let level_style = match record.level {
        StructuredLogLevel::Trace => Style::new().purple(),
        StructuredLogLevel::Debug => Style::new().blue(),
        StructuredLogLevel::Info => Style::new().green(),
        StructuredLogLevel::Warn => Style::new().yellow(),
        StructuredLogLevel::Error => Style::new().red(),
    };
    let mut output = styled(&event, level_style);
    if let Some(file) = &record.file {
        let _ = write!(
            output,
            "\n    {} {}",
            styled("at", Style::new().dimmed().italic()),
            escape_terminal_controls(file)
        );
        if let Some(line) = record.line {
            let _ = write!(output, ":{line}");
        }
    }

    for span in &record.spans {
        let _ = write!(
            output,
            "\n    {} {}",
            styled("in", Style::new().dimmed().italic()),
            styled(&escape_terminal_controls(&span.name), Style::new().bold())
        );
        if !span.fields.is_empty() {
            output.push(' ');
            output.push_str(&styled("with", Style::new().dimmed().italic()));
            output.push(' ');
            for (index, (name, value)) in span.fields.iter().enumerate() {
                if index != 0 {
                    output.push_str(", ");
                }
                let _ = write!(
                    output,
                    "{}: {}",
                    escape_terminal_controls(name),
                    escape_terminal_controls(value)
                );
            }
        }
    }
    output
}

fn format_level(level: StructuredLogLevel) -> &'static str {
    match level {
        StructuredLogLevel::Trace => "TRACE",
        StructuredLogLevel::Debug => "DEBUG",
        StructuredLogLevel::Info => " INFO",
        StructuredLogLevel::Warn => " WARN",
        StructuredLogLevel::Error => "ERROR",
    }
}

// The formatter is the trust boundary for terminal output. Preserve multiline
// messages, but escape content-provided ANSI/control sequences before applying
// our own styles. This applies equally to live stderr, inline output and replay.
fn escape_terminal_controls(value: &str) -> Cow<'_, str> {
    if !value
        .chars()
        .any(|character| character.is_control() && character != '\n')
    {
        return Cow::Borrowed(value);
    }
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_control() && character != '\n' {
            escaped.extend(character.escape_default());
        } else {
            escaped.push(character);
        }
    }
    Cow::Owned(escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::thread;
    use tracing_subscriber::layer::SubscriberExt;

    fn active_probe(active: &Arc<AtomicBool>) -> TerminalActivityProbe {
        let active = Arc::clone(active);
        Arc::new(move || active.load(Ordering::Acquire))
    }

    #[test]
    fn inactive_events_are_written_but_not_buffered() {
        let active = Arc::new(AtomicBool::new(false));
        let buffer = TerminalLogBuffer::new();
        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let subscriber = tracing_subscriber::registry().with(
            TerminalEventLayer::new(buffer.clone(), active_probe(&active)).with_inactive_writer({
                let output = Arc::clone(&output);
                move |record| {
                    output
                        .lock()
                        .expect("output lock")
                        .push(format_record(record))
                }
            }),
        );

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(source = "inactive", "not captured");
        });

        let mut cursor = 0;
        assert!(buffer.records_since(&mut cursor).is_empty());
        let output = output.lock().expect("output lock");
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("not captured"));
    }

    #[test]
    fn active_events_are_buffered_without_writing_to_the_inactive_sink() {
        let active = Arc::new(AtomicBool::new(true));
        let buffer = TerminalLogBuffer::new();
        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let subscriber = tracing_subscriber::registry().with(
            TerminalEventLayer::new(buffer.clone(), active_probe(&active)).with_inactive_writer({
                let output = Arc::clone(&output);
                move |record| {
                    output
                        .lock()
                        .expect("output lock")
                        .push(format_record(record))
                }
            }),
        );

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(source = "active", "captured");
        });

        let mut cursor = 0;
        let records = buffer.records_since(&mut cursor);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message.as_ref(), "captured");
        assert!(output.lock().expect("output lock").is_empty());
    }

    #[test]
    fn replay_preserves_later_presentation_of_structured_records() {
        let active = Arc::new(AtomicBool::new(true));
        let buffer = TerminalLogBuffer::new();
        let subscriber = tracing_subscriber::registry().with(TerminalEventLayer::new(
            buffer.clone(),
            active_probe(&active),
        ));

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("picker_handler", stage = "initial_load");
            let _entered = span.enter();
            tracing::warn!(query = "smith", "search returned no users");
        });

        active.store(false, Ordering::Release);
        let mut replay = Vec::new();
        buffer.replay_to(&mut replay).expect("replay should write");
        let replay = String::from_utf8(replay).expect("replay should be UTF-8");
        assert!(replay.contains("search returned no users"));
        assert!(replay.contains("query: smith"));
        assert!(replay.contains("picker_handler"));
        assert!(replay.contains("stage: initial_load"));
        assert!(replay.contains(" at "));
    }

    #[test]
    fn concurrent_event_producers_are_drained_without_losing_records() {
        let active = Arc::new(AtomicBool::new(true));
        let buffer = TerminalLogBuffer::new();
        let mut threads = Vec::new();
        for worker in 0..8 {
            let buffer = buffer.clone();
            let active = Arc::clone(&active);
            threads.push(thread::spawn(move || {
                let subscriber = tracing_subscriber::registry()
                    .with(TerminalEventLayer::new(buffer, active_probe(&active)));
                tracing::subscriber::with_default(subscriber, || {
                    for event in 0..32 {
                        tracing::info!(worker, event, "concurrent event");
                    }
                });
            }));
        }
        for thread in threads {
            thread.join().expect("event producer should finish");
        }

        let mut cursor = 0;
        let records = buffer.records_since(&mut cursor);
        assert_eq!(records.len(), 8 * 32);
        assert!(
            records
                .iter()
                .all(|record| record.message.as_ref() == "concurrent event")
        );
    }

    #[test]
    fn pretty_format_includes_fields_and_source_location() {
        let record = StructuredLogRecord {
            timestamp: SystemTime::now(),
            level: StructuredLogLevel::Info,
            target: Arc::from("cloud_terrastodon_entrypoint"),
            message: Arc::from("hello world"),
            fields: vec![("answer".to_string(), "42".to_string())],
            spans: Vec::new(),
            file: Some(Arc::from("crates/entrypoint/src/echo.rs")),
            line: Some(12),
        };

        assert_eq!(
            format_record_with_style(&record, |text, _| text.to_owned()),
            "   INFO hello world, answer: 42\n    at crates/entrypoint/src/echo.rs:12"
        );
    }

    #[test]
    fn pretty_format_includes_empty_span_context() {
        let record = StructuredLogRecord {
            timestamp: SystemTime::now(),
            level: StructuredLogLevel::Info,
            target: Arc::from("cloud_terrastodon_entrypoint"),
            message: Arc::from("hello world"),
            fields: Vec::new(),
            spans: vec![StructuredSpan {
                name: Arc::from("cli_invocation"),
                fields: Vec::new(),
            }],
            file: None,
            line: None,
        };

        assert_eq!(
            format_record_with_style(&record, |text, _| text.to_owned()),
            "   INFO hello world\n    in cli_invocation"
        );
    }

    #[test]
    fn terminal_format_colours_the_level_without_hyperlinking() {
        let record = StructuredLogRecord {
            timestamp: SystemTime::now(),
            level: StructuredLogLevel::Info,
            target: Arc::from("cloud_terrastodon_entrypoint"),
            message: Arc::from("hello world"),
            fields: Vec::new(),
            spans: Vec::new(),
            file: Some(Arc::from("crates/entrypoint/src/echo.rs")),
            line: Some(12),
        };

        let output = format_record_with_style(&record, |text, style| text.style(style).to_string());

        assert!(output.starts_with("\x1b[32m   INFO hello world\x1b[0m"));
        assert!(output.contains("\n    \x1b[2;3mat\x1b[0m crates/entrypoint/src/echo.rs:12"));
        assert!(!output.contains("\x1b]8;;"));
    }

    #[test]
    fn buffered_terminal_output_uses_the_same_formatter_as_immediate_output() {
        let active = Arc::new(AtomicBool::new(false));
        let buffer = TerminalLogBuffer::new();
        let immediate = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::registry().with(
            TerminalEventLayer::new(buffer.clone(), active_probe(&active)).with_inactive_writer({
                let immediate = Arc::clone(&immediate);
                move |record| immediate.lock().unwrap().push(format_record(record))
            }),
        );
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("drawing_update", path = "demo.excalidraw");
            let _entered = span.enter();
            for guarded in [false, true] {
                active.store(guarded, Ordering::Release);
                tracing::warn!(attempt = 3, "could not save drawing");
            }
        });
        let buffered = buffer.formatted_records_since(&mut 0);
        assert_eq!(buffered, *immediate.lock().unwrap());
        assert_eq!(buffered.len(), 1);
        assert!(buffered[0].contains("attempt: 3"));
        assert!(buffered[0].contains("drawing_update"));
        assert!(buffered[0].contains("demo.excalidraw"));
        assert!(buffered[0].contains("structured_log.rs:"));
    }

    #[test]
    fn terminal_controls_in_record_content_are_escaped_before_styling() {
        let record = StructuredLogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: StructuredLogLevel::Warn,
            target: Arc::from("test"),
            message: Arc::from("unsafe\x1b[2J\nsecond line"),
            fields: vec![("key\r".to_owned(), "value\x07".to_owned())],
            spans: vec![StructuredSpan {
                name: Arc::from("span\x1b[2J"),
                fields: vec![("scope".to_owned(), "value\x08".to_owned())],
            }],
            file: Some(Arc::from("file\x1b[2J.rs")),
            line: Some(42),
        };
        let plain = format_record_with_style(&record, |text, _| text.to_owned());
        assert!(
            !plain
                .chars()
                .any(|character| character.is_control() && character != '\n')
        );
        assert!(plain.contains("unsafe\\u{1b}[2J\nsecond line"));
        assert!(plain.contains("file\\u{1b}[2J.rs:42"));
        let styled = format_record_with_style(&record, |text, style| text.style(style).to_string());
        assert!(styled.contains("\x1b[33m"));
        assert!(!styled.contains("\x1b[2J"));
    }
}
