use chrono::Local;
use eyre::Result;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
pub use tracing::Level;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing(
    level: impl Into<Directive>,
    json_path: Option<impl AsRef<Path>>,
) -> Result<()> {
    let default_directive: Directive = level.into();
    let make_env_filter = || {
        EnvFilter::builder()
            .with_default_directive(default_directive.clone())
            .from_env_lossy()
    };
    let make_stderr_layer = || {
        tracing_subscriber::fmt::layer()
            .with_file(cfg!(debug_assertions))
            .with_target(false)
            .with_line_number(cfg!(debug_assertions))
            .with_writer(std::io::stderr)
            .pretty()
            .without_time()
    };

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
            .with_writer(json_writer);

        tracing_subscriber::registry()
            .with(make_env_filter())
            .with(make_stderr_layer())
            .with(json_layer)
            .try_init()?;

        info!(?json_log_path, "JSON log output initialized");
    } else {
        tracing_subscriber::registry()
            .with(make_env_filter())
            .with(make_stderr_layer())
            .try_init()?;
    }

    Ok(())
}
