use eyre::Result;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::Directive;

pub use tracing::Level;

pub fn init_tracing(level: impl Into<Directive>) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy(),
        )
        .with_file(true)
        .with_target(false)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .without_time()
        .init();
    Ok(())
}
