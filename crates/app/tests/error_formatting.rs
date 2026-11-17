#![cfg(test)]

//! Keep the process-global report hook isolated from other application tests.

use cloud_terrastodon_app::install_error_hook;
use cloud_terrastodon_tracing::DiagnosticReport;
use cloud_terrastodon_tracing::StructuredLogValue;
use cloud_terrastodon_tracing::TerminalEventLayer;
use cloud_terrastodon_tracing::TerminalLogBuffer;
use color_eyre::Section;
use eyre::Report;
use facet::Facet;
use std::fmt;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tracing_error::ErrorLayer;
use tracing_error::TracedError;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Clone)]
struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("captured log lock")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Facet)]
struct LoggedEvent {
    fields: LoggedFields,
}

#[derive(Facet)]
struct LoggedFields {
    error: String,
    message: String,
}

#[test]
fn reports_remain_readable_in_terminal_records_and_json() {
    install_error_hook().expect("install the actual application report hook");

    let buffer = TerminalLogBuffer::new();
    let json = Arc::new(Mutex::new(Vec::<u8>::new()));
    let writer = CapturedWriter(Arc::clone(&json));
    let subscriber = tracing_subscriber::registry()
        .with(TerminalEventLayer::new(&buffer, Arc::new(|| true)))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(move || writer.clone()),
        )
        .with(ErrorLayer::default());

    let retained = tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("synthetic_error_scope", operation = "save_settings");
        let _entered = span.enter();
        // Retain an explicit source span even if RUST_SPANTRACE disables the
        // report hook's optional, additional capture in the host environment.
        let error = Report::new(TracedError::from(std::io::Error::other(
            "synthetic write failed\nunderlying detail",
        )))
        .wrap_err("Cannot save synthetic settings")
        .note("Synthetic settings may be retried");
        let report = DiagnosticReport::new(error);
        tracing::error!(error = report.as_error(), "Synthetic save failed");
        Arc::clone(&report.report)
    });

    let records = buffer.records_since(&mut 0);
    assert_eq!(records.len(), 1);
    let (_, value) = records[0]
        .fields
        .iter()
        .find(|(name, _)| name == "error")
        .expect("the report must be retained as an event field");
    let StructuredLogValue::Report(report) = value else {
        panic!("logging must retain the actual report rather than a formatted string");
    };
    assert!(Arc::ptr_eq(&report.report, &retained));
    let report = report.to_string();
    assert!(report.contains("Cannot save synthetic settings"));
    assert!(report.contains("synthetic write failed\n"));
    assert!(report.contains("underlying detail"));
    assert!(report.contains("synthetic_error_scope"));
    assert!(report.contains("Synthetic settings may be retried"));
    assert!(!report.contains('\x1b'));
    assert!(!report.contains("\\u{1b}"));

    let terminal = buffer.formatted_records_since(&mut 0);
    assert!(terminal[0].contains("Cannot save synthetic settings"));
    assert!(terminal[0].contains("underlying detail"));
    assert!(!terminal[0].contains("\\u{1b}"));

    let json = String::from_utf8(json.lock().expect("captured log lock").clone())
        .expect("JSON logs must be UTF-8");
    assert_eq!(json.lines().count(), 1);
    assert!(!json.contains('\x1b'));
    assert!(!json.contains("\\u{1b}"));
    let event: LoggedEvent = facet_json::from_str(json.trim_end()).expect("one valid JSON event");
    assert_eq!(event.fields.message, "Synthetic save failed");
    assert_eq!(event.fields.error, report);
}

#[test]
fn buffering_and_cloning_reports_do_not_render_them() {
    #[derive(Debug)]
    struct CountedError(Arc<AtomicUsize>);

    impl fmt::Display for CountedError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fetch_add(1, Ordering::Relaxed);
            formatter.write_str("rendered only by a presentation surface")
        }
    }

    impl std::error::Error for CountedError {}

    let calls = Arc::new(AtomicUsize::new(0));
    let buffer = TerminalLogBuffer::new();
    let subscriber =
        tracing_subscriber::registry().with(TerminalEventLayer::new(&buffer, Arc::new(|| true)));
    let report = DiagnosticReport::new(Report::new(CountedError(Arc::clone(&calls))));
    let before = calls.load(Ordering::Relaxed);
    tracing::subscriber::with_default(subscriber, || {
        tracing::error!(error = report.as_error(), "Queued diagnostic");
    });
    drop(report);
    let records = buffer.records_since(&mut 0);
    let cloned = records.clone();
    assert_eq!(calls.load(Ordering::Relaxed), before);
    assert!(matches!(
        cloned[0].fields[0].1,
        StructuredLogValue::Report(_)
    ));

    let rendered = buffer.formatted_records_since(&mut 0);
    assert!(rendered[0].contains("rendered only by a presentation surface"));
    assert!(calls.load(Ordering::Relaxed) > before);
}
