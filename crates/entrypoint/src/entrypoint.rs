use crate::clap::Cli;
use crate::clap::Commands;
use crate::menu::menu_loop;
use crate::noninteractive::prelude::clean;
use crate::noninteractive::prelude::dump_everything;
use crate::noninteractive::prelude::perform_import;
use crate::noninteractive::prelude::process_generated;
use crate::noninteractive::prelude::write_imports_for_all_resource_groups;
use crate::noninteractive::prelude::write_imports_for_all_role_assignments;
use crate::noninteractive::prelude::write_imports_for_all_security_groups;
use crate::prelude::Version;
use clap::CommandFactory;
use clap::FromArgMatches;
use cloud_terrastodon_config::iconfig::IConfig;
use cloud_terrastodon_config::work_dirs_config::WorkDirsConfig;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_ui_egui::egui_main;
use cloud_terrastodon_ui_ratatui::prelude::ui_main;
use eyre::Context;
use eyre::Result;
use std::fs::canonicalize;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::io::stdout;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

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
    if cli.debug {
        unsafe { std::env::set_var("RUST_BACKTRACE", "full") };
        // std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Configure tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(
                    match cli.debug {
                        true => LevelFilter::DEBUG,
                        false => LevelFilter::INFO,
                    }
                    .into(),
                )
                .from_env_lossy(),
        )
        .with_file(true)
        .with_target(false)
        .with_line_number(true)
        .without_time()
        .init();

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
    runtime.block_on(handle(cli.command))?;
    Ok(())
}

pub async fn handle(command: Option<Commands>) -> eyre::Result<()> {
    match command {
        None => {
            menu_loop().await?;
        }
        Some(command) => match command {
            Commands::Ratatui => {
                ui_main().await?;
            }
            Commands::Egui => {
                egui_main().await?;
            }
            Commands::Clean => {
                clean().await?;
            }
            Commands::WriteAllImports => {
                write_imports_for_all_resource_groups().await?;
                write_imports_for_all_security_groups().await?;
                write_imports_for_all_role_assignments().await?;
            }
            Commands::PerformCodeGenerationFromImports => {
                perform_import().await?;
                process_generated().await?;
            }
            Commands::GetPath { dir } => {
                let mut out = stdout();
                out.write_all(dir.as_path_buf().as_os_str().as_encoded_bytes())
                    .await?;
                out.flush().await?;
            }
            Commands::AddWorkDir { mut dir } => {
                if !dir.is_absolute() {
                    dir = canonicalize(&dir)
                        .context(format!("failed to make path absolute: {}", dir.display()))?;
                }
                let mut config = WorkDirsConfig::load().await?;
                config.work_dirs.insert(dir);
                config.save().await?;
            }
            Commands::CopyResults { dest } => {
                // from https://stackoverflow.com/a/78769977/11141271
                #[async_recursion::async_recursion]
                async fn copy_dir_all<S, D>(src: S, dst: D) -> Result<(), std::io::Error>
                where
                    S: AsRef<Path> + Send + Sync,
                    D: AsRef<Path> + Send + Sync,
                {
                    tokio::fs::create_dir_all(&dst).await?;
                    let mut entries = tokio::fs::read_dir(src).await?;
                    while let Some(entry) = entries.next_entry().await? {
                        let ty = entry.file_type().await?;
                        if ty.is_dir() {
                            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))
                                .await?;
                        } else {
                            tokio::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))
                                .await?;
                        }
                    }
                    Ok(())
                }
                copy_dir_all(AppDir::Processed.as_path_buf(), dest).await?;
            }
            Commands::DumpEverything => {
                dump_everything().await?;
            }
        },
    }
    Ok(())
}
