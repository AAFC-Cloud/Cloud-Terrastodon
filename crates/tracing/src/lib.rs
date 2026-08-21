use chrono::Local;
// TODO(EGUI-TRACING)
// use egui_tracing::tracing::collector::EventCollector;
use eyre::Result;
use std::fs::OpenOptions;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
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

mod structured_log;

pub use structured_log::*;

type InactiveWriter = Arc<dyn Fn(&StructuredLogRecord) + Send + Sync>;

fn exclude_tracy_frame_mark(meta: &Metadata<'_>) -> bool {
    meta.fields().field("tracy.frame_mark").is_none()
}

#[cfg(feature = "tracy")]
fn tracy_log_filter_directive() -> &'static str {
    "trace"
}

#[cfg(feature = "tracy")]
fn tracy_log_filter() -> Result<EnvFilter> {
    EnvFilter::builder()
        .parse(tracy_log_filter_directive())
        .map_err(Into::into)
}

// TODO(EGUI-TRACING)
// static EVENT_COLLECTOR: LazyLock<EventCollector> = LazyLock::new(EventCollector::default);

// TODO(EGUI-TRACING)
// /// Return a clone of the global EventCollector for use in GUI widgets
// pub fn event_collector() -> EventCollector {
//     EVENT_COLLECTOR.clone()
// }

/// Initialize tracing for the whole application, registering independently filtered stderr and
/// optional JSON file layers, and the GUI event collector so that `egui_tracing::Logs` works.
pub fn init_tracing(
    level: impl Into<Directive>,
    file_level: Option<impl Into<Directive>>,
    json_path: Option<impl AsRef<Path>>,
    enable_egui_collector: bool,
) -> Result<()> {
    init_tracing_with_terminal(
        level,
        file_level,
        json_path,
        enable_egui_collector,
        None,
        None,
    )
}

pub fn init_tracing_with_terminal(
    level: impl Into<Directive>,
    file_level: Option<impl Into<Directive>>,
    json_path: Option<impl AsRef<Path>>,
    #[expect(unused)] enable_egui_collector: bool,
    terminal_log_buffer: Option<TerminalLogBuffer>,
    terminal_active: Option<TerminalActivityProbe>,
) -> Result<()> {
    let level = level.into();
    let file_level = file_level.map(Into::into);
    let json_path = json_path.map(|path| path.as_ref().to_path_buf());
    let subscriber = build_subscriber(
        level,
        file_level,
        json_path,
        terminal_log_buffer,
        terminal_active,
        None,
    )?;

    subscriber.try_init()?;

    #[cfg(all(feature = "tracy", not(test)))]
    info!("Tracy profiling layer added, memory usage will increase until a client is connected");

    Ok(())
}

fn build_subscriber(
    level: Directive,
    file_level: Option<Directive>,
    json_path: Option<PathBuf>,
    terminal_log_buffer: Option<TerminalLogBuffer>,
    terminal_active: Option<TerminalActivityProbe>,
    inactive_writer: Option<InactiveWriter>,
) -> Result<impl tracing::Subscriber + Send + Sync + 'static> {
    let stderr_filter = EnvFilter::builder()
        .with_default_directive(level.clone())
        .from_env_lossy();
    let file_filter = file_level
        .map(|level| {
            // An explicitly supplied file filter is independent of RUST_LOG. When the option is
            // omitted, the file layer below reuses the effective stderr filter instead.
            EnvFilter::builder()
                .with_default_directive(level)
                .parse_lossy("")
        })
        .unwrap_or_else(|| stderr_filter.clone());

    let human_layer = if let (Some(buffer), Some(active)) = (terminal_log_buffer, terminal_active) {
        let layer = TerminalEventLayer::new(buffer, active);
        let layer = if let Some(inactive_writer) = inactive_writer {
            layer.with_inactive_writer(move |record| inactive_writer(record))
        } else {
            layer
        };
        layer
            .boxed()
            .with_filter(
                stderr_filter
                    .clone()
                    .and(filter_fn(exclude_tracy_frame_mark)),
            )
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            // Keep the human-readable field cache distinct from the JSON layer's
            // `DefaultFields`; otherwise the stderr layer's ANSI-formatted fields can be
            // reparsed by the JSON formatter as though they were JSON.
            .fmt_fields(tracing_subscriber::fmt::format::PrettyFields::default())
            .with_file(cfg!(debug_assertions))
            .with_target(false)
            .with_line_number(cfg!(debug_assertions))
            .with_writer(std::io::stderr)
            .pretty()
            .without_time()
            .with_ansi(std::io::stderr().is_terminal())
            .with_filter(stderr_filter.and(filter_fn(exclude_tracy_frame_mark)))
            .boxed()
    };

    let subscriber = tracing_subscriber::registry()
        .with(human_layer)
        // TODO(EGUI-TRACING)
        // .with({
        //     if enable_egui_collector {
        //         Some(event_collector().clone())
        //     } else {
        //         None
        //     }
        // })
        .with({
            // Build registry with optional JSON layer; Option<Layer> implements Layer so
            // the resulting type is the same whether the layer is Some(_) or None.
            if let Some(path) = json_path {
                let path = path.as_path();
                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                let json_log_path = if path.exists() && path.is_dir() {
                    path.join(format!("cloud_terrastodon_log_{}.ndjson", timestamp))
                } else {
                    path.to_path_buf()
                };

                if let Some(parent) = json_log_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&json_log_path)?;
                let file = Arc::new(Mutex::new(file));
                let json_writer = {
                    let file = Arc::clone(&file);
                    BoxMakeWriter::new(move || {
                        file.lock()
                            .expect("failed to lock json log file")
                            .try_clone()
                            .expect("failed to clone json log file handle")
                    })
                };

                let json_layer = tracing_subscriber::fmt::layer()
                    // Use the JSON layer constructor so span fields use JsonFields as well as
                    // the event formatter. Configuring only event_format(json()) leaves the
                    // DefaultFields formatter in place, which can contain ANSI text from another
                    // formatting layer and then gets reparsed as JSON.
                    .json()
                    .with_writer(json_writer)
                    .with_ansi(false)
                    .boxed()
                    .with_filter(file_filter.and(filter_fn(exclude_tracy_frame_mark)));

                info!(?json_log_path, "JSON log output initialized");
                Some(json_layer)
            } else {
                None
            }
        });

    #[cfg(all(feature = "tracy", not(test)))]
    let subscriber =
        subscriber.with(tracing_tracy::TracyLayer::default().with_filter(tracy_log_filter()?));

    Ok(subscriber)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::SubscriberExt;

    #[cfg(feature = "tracy")]
    #[test]
    fn tracy_filter_is_always_trace() {
        assert_eq!(tracy_log_filter_directive(), "trace");
        tracy_log_filter().expect("the fixed Tracy filter should parse");
    }

    #[test]
    fn json_layer_does_not_reparse_human_span_fields() {
        let human_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(std::io::sink)
            .with_ansi(true)
            .boxed();
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(std::io::sink)
            .boxed();
        let subscriber = tracing_subscriber::registry()
            .with(human_layer)
            .with(json_layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!(
                "command_run_raw",
                summary = "az account list --output json --debug",
                location = "crates/azure/src/accounts.rs:15:19",
            );
            let _entered = span.enter();
            tracing::info!("request started");
        });
    }

    #[test]
    fn json_style_output_keeps_the_frame_mark_filter() {
        use std::io::Write;
        use std::sync::Mutex;

        #[derive(Clone)]
        struct SharedWriter(Arc<Mutex<Vec<u8>>>);

        impl Write for SharedWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0
                    .lock()
                    .expect("shared writer lock")
                    .extend_from_slice(bytes);
                Ok(bytes.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let output = Arc::new(Mutex::new(Vec::new()));
        let output_for_writer = Arc::clone(&output);
        let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
            SharedWriter(Arc::clone(&output_for_writer))
        });
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .boxed()
            .with_filter(filter_fn(exclude_tracy_frame_mark));
        let subscriber = tracing_subscriber::registry().with(json_layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(tracy.frame_mark = true, "frame mark");
            tracing::info!(source = "test", "normal event");
        });

        let output = String::from_utf8(output.lock().expect("shared writer lock").clone())
            .expect("JSON output should be UTF-8");
        assert!(!output.contains("frame mark"), "{output}");
        assert!(output.contains("normal event"), "{output}");
    }

    #[test]
    fn structured_layer_retains_event_and_span_fields() {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;
        use tracing_subscriber::layer::SubscriberExt;

        let active = Arc::new(AtomicBool::new(true));
        let buffer = TerminalLogBuffer::new();
        let layer = TerminalEventLayer::new(
            buffer.clone(),
            Arc::new({
                let active = Arc::clone(&active);
                move || active.load(Ordering::Acquire)
            }),
        );
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!(
                "picker_handler",
                event = "query_changed",
                message = "handler scope",
            );
            let _entered = span.enter();
            tracing::warn!(query = "smith", "search failed");
        });

        let mut cursor = 0;
        let records = buffer.records_since(&mut cursor);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message.as_ref(), "search failed");
        assert_eq!(
            records[0].fields,
            [("query".to_string(), "smith".to_string())]
        );
        assert_eq!(records[0].spans[0].name.as_ref(), "picker_handler");
        assert_eq!(
            records[0].spans[0].fields,
            [
                ("event".to_string(), "query_changed".to_string()),
                ("message".to_string(), "handler scope".to_string())
            ]
        );
    }

    #[test]
    fn built_terminal_subscriber_routes_human_output_and_file_output_independently() {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;

        let active = Arc::new(AtomicBool::new(false));
        let buffer = TerminalLogBuffer::new();
        let human_output = Arc::new(Mutex::new(Vec::<String>::new()));
        let log_path = std::env::temp_dir().join(format!(
            "cloud_terrastodon_tracing_test_{}.ndjson",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&log_path);

        let subscriber = build_subscriber(
            tracing_subscriber::filter::LevelFilter::INFO.into(),
            Some(tracing_subscriber::filter::LevelFilter::INFO.into()),
            Some(log_path.clone()),
            Some(buffer.clone()),
            Some({
                let active = Arc::clone(&active);
                Arc::new(move || active.load(Ordering::Acquire))
            }),
            Some({
                let human_output = Arc::clone(&human_output);
                Arc::new(move |record| {
                    human_output
                        .lock()
                        .expect("human output lock")
                        .push(format_record(record));
                })
            }),
        )
        .expect("subscriber should build");

        tracing::subscriber::with_default(subscriber, || {
            tracing::warn!(surface = "stderr", "normal human output");
            active.store(true, Ordering::Release);
            tracing::error!(surface = "picker", "guarded human output");
            active.store(false, Ordering::Release);
        });

        let human_output = human_output.lock().expect("human output lock");
        assert_eq!(human_output.len(), 1);
        assert!(human_output[0].contains("normal human output"));
        assert!(!human_output[0].contains("guarded human output"));
        drop(human_output);

        let mut cursor = 0;
        let buffered = buffer.records_since(&mut cursor);
        assert_eq!(buffered.len(), 1);
        assert_eq!(buffered[0].message.as_ref(), "guarded human output");

        let file_output = std::fs::read_to_string(&log_path).expect("JSON log should be written");
        assert!(file_output.contains("normal human output"));
        assert!(file_output.contains("guarded human output"));
        let _ = std::fs::remove_file(log_path);
    }

    #[test]
    fn frame_mark_events_are_filtered_from_human_structured_records() {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;

        let active = Arc::new(AtomicBool::new(true));
        let buffer = TerminalLogBuffer::new();
        let layer = TerminalEventLayer::new(
            buffer.clone(),
            Arc::new({
                let active = Arc::clone(&active);
                move || active.load(Ordering::Acquire)
            }),
        )
        .with_filter(filter_fn(exclude_tracy_frame_mark));
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(tracy.frame_mark = true, "frame mark");
            tracing::info!("normal event");
        });

        let mut cursor = 0;
        let records = buffer.records_since(&mut cursor);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message.as_ref(), "normal event");
    }
}
