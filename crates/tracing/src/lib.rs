use chrono::Local;
use egui_tracing::tracing::collector::EventCollector;
use eyre::Result;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

static EVENT_COLLECTOR: LazyLock<EventCollector> = LazyLock::new(EventCollector::default);

/// Return a clone of the global EventCollector for use in GUI widgets
pub fn event_collector() -> EventCollector {
    EVENT_COLLECTOR.clone()
}

/// Initialize tracing for the whole application, registering a stderr layer, an
/// optional JSON file writer, an env filter, and the GUI event collector so that
/// `egui_tracing::Logs` works.
pub fn init_tracing(
    level: impl Into<Directive>,
    json_path: Option<impl AsRef<Path>>,
    enable_egui_collector: bool,
) -> Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(cfg!(debug_assertions))
                .with_target(false)
                .with_line_number(cfg!(debug_assertions))
                .with_writer(std::io::stderr)
                .pretty()
                .without_time(),
        )
        .with({
            if enable_egui_collector {
                Some(event_collector().clone())
            } else {
                None
            }
        })
        .with({
            // Build registry with optional JSON layer; Option<Layer> implements Layer so
            // the resulting type is the same whether the layer is Some(_) or None.
            if let Some(path) = json_path {
                let path = path.as_ref();
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
                    .event_format(tracing_subscriber::fmt::format().json())
                    .with_writer(json_writer)
                    .boxed();

                info!(?json_log_path, "JSON log output initialized");
                Some(json_layer)
            } else {
                None
            }
        })
        .try_init()?;

    Ok(())
}
