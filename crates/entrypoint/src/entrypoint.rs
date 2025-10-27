use crate::cli::Cli;
use crate::prelude::Version;
use crate::tracing::init_tracing;
use clap::CommandFactory;
use clap::FromArgMatches;
use eyre::Result;
use tracing::level_filters::LevelFilter;

pub fn entrypoint(version: Version) -> Result<()> {
    // let panic_hook = std::panic::take_hook();
    // std::panic::set_hook(Box::new(move |info| {
    //     tracing::error!(
    //         "Panic encountered at {}",
    //         info.location()
    //             .map(|x| x.to_string())
    //             .unwrap_or("unknown location".to_string())
    //     );
    //     panic_hook(info);
    // }));

    color_eyre::install()?;

    // Parse command line arguments
    let mut cmd = Cli::command();
    cmd = cmd.version(version.to_string());
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
            false => LevelFilter::INFO,
        },
        cli.global_args.json,
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
