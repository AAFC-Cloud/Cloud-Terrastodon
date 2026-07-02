use crate::BuildTimestamp;
use crate::Version;
use crate::build_timestamp::set_build_timestamp;
use crate::cli::Cli;
use crate::cli::CloudTerrastodonCommand;
use crate::git_revision::GitRevision;
use crate::git_revision::set_git_revision;
use crate::version::full_version;
use crate::version::set_version;
use cloud_terrastodon_tracing::init_tracing;
use eyre::Result;
use figue::Driver;
use std::str::FromStr;
use teamy_cancellation::CtrlCHandler;
use tracing::level_filters::LevelFilter;

pub fn entrypoint(
    version: Version,
    git_rev: GitRevision,
    build_timestamp: BuildTimestamp,
) -> Result<()> {
    // Track version information globally
    set_git_revision(git_rev);
    set_build_timestamp(build_timestamp);
    set_version(version);

    color_eyre::install()?;

    // Parse command line arguments.
    let cli: Cli = Driver::new(
        figue::builder::<Cli>()
            .expect("CLI schema should be valid")
            .cli(|cli| cli.args_os(std::env::args_os().skip(1)).strict())
            .help(|help| help.version(full_version().to_string()))
            .build(),
    )
    .run()
    .unwrap();

    // Configure backtrace-always
    if cli.global_args.debug {
        unsafe { std::env::set_var("RUST_BACKTRACE", "full") };
        // std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Configure tracing
    init_tracing(
        match cli.global_args.debug {
            true => LevelFilter::DEBUG,
            false => LevelFilter::from_str(&cli.global_args.log_filter)?,
        },
        cli.global_args.log_file.as_ref(),
        matches!(cli.command.as_ref(), Some(CloudTerrastodonCommand::Egui(_))),
    )?;

    // Configure terminal colour support
    #[cfg(windows)]
    let _ = crate::windows_support::windows_ansi::enable_ansi_support();

    // Warn if UTF-8 support is not enabled on Windows.
    #[cfg(windows)]
    if !crate::windows_support::windows_utf8::is_system_utf8() {
        tracing::warn!("The current system codepage is not UTF-8. This may cause '�' problems.");
        tracing::warn!(
            "See https://github.com/Azure/azure-cli/issues/22616#issuecomment-1147061949"
        );
        tracing::warn!(
            "Control panel -> Clock and Region -> Region -> Administrative -> Change system locale -> Check Beta: Use Unicode UTF-8 for worldwide language support."
        );
    }

    let cancellation_token = CtrlCHandler::default().install()?;

    // Build async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(cli.invoke(&cancellation_token))?;
    Ok(())
}
