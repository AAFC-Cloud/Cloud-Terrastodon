use crate::cli::Cli;
use crate::cli::CloudTerrastodonCommand;
use crate::git_revision::GitRevision;
use crate::git_revision::set_git_revision;
use crate::prelude::Version;
use crate::version::full_version;
use crate::version::set_version;
use clap::CommandFactory;
use clap::FromArgMatches;
use cloud_terrastodon_tracing::init_tracing;
use eyre::Result;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;

pub fn entrypoint(version: Version, git_rev: GitRevision) -> Result<()> {
    // Track version information globally
    set_git_revision(git_rev);
    set_version(version);

    color_eyre::install()?;

    // Parse command line arguments
    let mut cmd = Cli::command();
    cmd = cmd.version(full_version().to_string());
    let cli = Cli::from_arg_matches(&cmd.get_matches())?;

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
        matches!(cli.command, Some(CloudTerrastodonCommand::Egui(_))),
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

    // Build async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(cli.invoke())?;
    Ok(())
}
