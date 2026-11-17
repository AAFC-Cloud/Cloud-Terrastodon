//! Install CT's subscriber, destinations, filtering, and terminal policy.
//!
//! Reusable capture, retained reports, and presentation live in `teamy-tracing`.
use chrono::Local;
use eyre::Result;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use teamy_tracing::Diagnostics;
use teamy_tracing::capture::CaptureLayer;
use teamy_tracing::capture::LogBuffer;
use teamy_tracing::capture::LogRecord;
use teamy_tracing::capture::format_record;
use tracing::Metadata;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::filter::FilterExt;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

type InactiveWriter = Arc<dyn Fn(&LogRecord) + Send + Sync>;

fn exclude_tracy_frame_mark(meta: &Metadata<'_>) -> bool {
    meta.fields().field("tracy.frame_mark").is_none()
}

#[cfg(feature = "tracy")]
fn tracy_log_filter() -> Result<EnvFilter> {
    Ok(EnvFilter::builder().parse("trace")?)
}

/// Install independently filtered human output, optional JSON file output,
/// and optional application-owned Debug-through-Error history.
///
/// The supplied history is not replayed to stderr. Keep its owner alive until
/// all viewers and admitted jobs have finished; subscriber layers hold it weakly.
pub fn init_tracing(
    level: impl Into<Directive>,
    file_level: Option<impl Into<Directive>>,
    json_path: Option<impl AsRef<Path>>,
    capture: Option<LogBuffer>,
) -> Result<()> {
    init_tracing_with_terminal(level, file_level, json_path, None, None, capture)
}

/// Install CT's process subscriber with caller-owned terminal capture.
///
/// Human logs buffer only while the terminal activity probe reports an owner.
/// Debug history uses its own filter and buffer, independently of human replay.
pub fn init_tracing_with_terminal(
    level: impl Into<Directive>,
    file_level: Option<impl Into<Directive>>,
    json_path: Option<impl AsRef<Path>>,
    terminal_buffer: Option<LogBuffer>,
    terminal_active: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    capture: Option<LogBuffer>,
) -> Result<()> {
    build_subscriber(
        EnvFilter::builder()
            .with_default_directive(level.into())
            .from_env_lossy(),
        file_level.map(Into::into),
        json_path.map(|path| path.as_ref().to_path_buf()),
        terminal_buffer,
        terminal_active,
        capture,
        Arc::new(|record| eprintln!("{}", format_record(record))),
    )?
    .try_init()?;

    #[cfg(all(feature = "tracy", not(test)))]
    info!("Tracy profiling layer added, memory usage will increase until a client is connected");
    Ok(())
}

fn build_subscriber(
    stderr_filter: EnvFilter,
    file_level: Option<Directive>,
    json_path: Option<PathBuf>,
    terminal_buffer: Option<LogBuffer>,
    terminal_active: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    capture: Option<LogBuffer>,
    inactive_writer: InactiveWriter,
) -> Result<impl tracing::Subscriber + Send + Sync + 'static> {
    let file_filter = file_level
        .map(|level| {
            EnvFilter::builder()
                .with_default_directive(level)
                .parse_lossy("")
        })
        .unwrap_or_else(|| stderr_filter.clone());
    // Keep diagnostic history regardless of terminal ownership. The always-true
    // probe sends every event admitted by this layer's filter to the history buffer.
    let capture_layer = capture.as_ref().map(|buffer| {
        CaptureLayer::new(buffer, Arc::new(|| true), |_| {}).with_filter(
            tracing_subscriber::filter::LevelFilter::DEBUG.and(filter_fn(exclude_tracy_frame_mark)),
        )
    });
    // Queue human output while an interactive view owns the terminal. Otherwise,
    // send each record straight to the inactive sink (stderr in production).
    let buffer = terminal_buffer.unwrap_or_default();
    let human_layer = CaptureLayer::new(
        &buffer,
        terminal_active.unwrap_or_else(|| Arc::new(|| false)),
        move |record| inactive_writer(record),
    )
    .with_filter(stderr_filter.and(filter_fn(exclude_tracy_frame_mark)));

    let json_layer = if let Some(path) = json_path {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let json_log_path = if path.is_dir() {
            path.join(format!("cloud_terrastodon_log_{timestamp}.ndjson"))
        } else {
            path
        };
        if let Some(parent) = json_log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&json_log_path)?;
        let layer = teamy_tracing::json_layer(BoxMakeWriter::new(file), Diagnostics::default())
            .boxed()
            .with_filter(file_filter.and(filter_fn(exclude_tracy_frame_mark)));
        info!(?json_log_path, "JSON log output initialized");
        Some(layer)
    } else {
        None
    };
    let subscriber = tracing_subscriber::registry()
        .with(capture_layer)
        .with(human_layer)
        .with(json_layer)
        .with(tracing_error::ErrorLayer::default());
    #[cfg(all(feature = "tracy", not(test)))]
    let subscriber =
        subscriber.with(tracing_tracy::TracyLayer::default().with_filter(tracy_log_filter()?));
    Ok(subscriber)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use teamy_tracing::DiagnosticReport;
    use teamy_tracing::capture::LogValue;

    #[test]
    fn history_and_terminal_replay_have_independent_filters_and_retention() {
        let terminal = LogBuffer::new();
        let history = LogBuffer::bounded(NonZeroUsize::new(3).unwrap());
        let subscriber = tracing_subscriber::registry()
            .with(
                CaptureLayer::new(&history, Arc::new(|| true), |_| {})
                    .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
            )
            .with(
                CaptureLayer::new(&terminal, Arc::new(|| true), |_| {})
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
            );
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("capture_test", phase = "running");
            let _entered = span.enter();
            tracing::debug!("expired");
            tracing::info!("visible");
            tracing::debug!("history only");
            tracing::warn!("latest");
        });
        let mut cursor = 0;
        let captured = history.records_since(&mut cursor);
        assert_eq!(cursor, 4);
        assert_eq!(
            captured
                .iter()
                .map(|record| record.message.as_ref())
                .collect::<Vec<_>>(),
            ["visible", "history only", "latest"]
        );
        let mut output = Vec::new();
        terminal.replay_to(&mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("visible"));
        assert!(!output.contains("history only"));
        terminal.records_since(&mut cursor);
        terminal.acknowledge_through(cursor);
        assert!(terminal.records_since(&mut 0).is_empty());
        assert_eq!(history.records_since(&mut 0).len(), 3);
    }

    #[test]
    fn installed_destinations_preserve_reports_and_route_logs_independently() {
        let terminal = LogBuffer::new();
        let history = LogBuffer::bounded(NonZeroUsize::new(8).unwrap());
        let active = Arc::new(AtomicBool::new(false));
        let human = Arc::new(Mutex::new(Vec::<String>::new()));
        let path = std::env::temp_dir().join(format!("ct-tracing-{}.ndjson", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let subscriber = build_subscriber(
            EnvFilter::new("info"),
            Some(tracing_subscriber::filter::LevelFilter::DEBUG.into()),
            Some(path.clone()),
            Some(terminal.clone()),
            Some({
                let active = Arc::clone(&active);
                Arc::new(move || active.load(Ordering::Acquire))
            }),
            Some(history.clone()),
            {
                let human = Arc::clone(&human);
                Arc::new(move |record| human.lock().unwrap().push(format_record(record)))
            },
        )
        .unwrap();
        let report = DiagnosticReport::new(
            eyre::eyre!("synthetic failure").wrap_err("settings save failed"),
        );
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("synthetic_request", operation = "save");
            let _entered = span.enter();
            assert_eq!(
                tracing_error::SpanTrace::capture().status(),
                tracing_error::SpanTraceStatus::CAPTURED
            );
            tracing::info!("normal human output");
            active.store(true, Ordering::Release);
            tracing::error!(error = report.as_error(), "guarded human output");
            tracing::debug!("history and file only");
            tracing::info!(tracy.frame_mark = true, "excluded frame mark");
        });
        let records = terminal.records_since(&mut 0);
        assert_eq!(records.len(), 1);
        let LogValue::Report(retained) = &records[0].fields[0].1 else {
            panic!("typed report lost")
        };
        assert!(Arc::ptr_eq(&retained.report, &report.report));
        assert_eq!(human.lock().unwrap().len(), 1);
        assert!(human.lock().unwrap()[0].contains("normal human output"));
        assert_eq!(history.records_since(&mut 0).len(), 3);
        let file = std::fs::read_to_string(&path).unwrap();
        assert!(file.contains("normal human output"));
        assert!(file.contains("guarded human output"));
        assert!(file.contains("history and file only"));
        assert!(!file.contains("excluded frame mark"));
        assert!(!file.contains("\\u{1b}"));
        std::fs::remove_file(path).unwrap();
    }
}
