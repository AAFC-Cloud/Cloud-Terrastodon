use chrono::Local;
// TODO(EGUI-TRACING)
// use egui_tracing::tracing::collector::EventCollector;
use eyre::Result;
use std::fs::OpenOptions;
use std::io::IsTerminal;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

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
    #[expect(unused)] // TODO(EGUI-TRACING)
    enable_egui_collector: bool,
) -> Result<()> {
    let level = level.into();
    let stderr_filter = EnvFilter::builder()
        .with_default_directive(level.clone())
        .from_env_lossy();
    let file_filter = file_level
        .map(Into::into)
        .map(|level| {
            // An explicitly supplied file filter is independent of RUST_LOG. When the option is
            // omitted, the file layer below reuses the effective stderr filter instead.
            EnvFilter::builder()
                .with_default_directive(level)
                .parse_lossy("")
        })
        .unwrap_or_else(|| stderr_filter.clone());

    tracing_subscriber::registry()
        .with(
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
                .with_filter(stderr_filter),
        )
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
                    // Use the JSON layer constructor so span fields use JsonFields as well as
                    // the event formatter. Configuring only event_format(json()) leaves the
                    // DefaultFields formatter in place, which can contain ANSI text from another
                    // formatting layer and then gets reparsed as JSON.
                    .json()
                    .with_writer(json_writer)
                    .with_ansi(false)
                    .boxed()
                    .with_filter(file_filter);

                info!(?json_log_path, "JSON log output initialized");
                Some(json_layer)
            } else {
                None
            }
        })
        .try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::Layer;
    use tracing_subscriber::layer::SubscriberExt;

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
}
