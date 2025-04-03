use crate::clap::Cli;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub fn init_tracing(cli: &Cli) {
    let level = if cli.debug {
        std::env::set_var("RUST_BACKTRACE", "1");
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    let env_filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_file(true)
        .with_target(false)
        .with_line_number(true)
        .without_time()
        .init();
}
