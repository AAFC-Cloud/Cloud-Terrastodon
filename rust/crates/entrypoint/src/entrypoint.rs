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
use crate::tracing::init_tracing;
use clap::CommandFactory;
use clap::FromArgMatches;
use cloud_terrastodon_core_config::Config;
use cloud_terrastodon_core_pathing::AppDir;
use cloud_terrastodon_ui_egui::egui_main;
use cloud_terrastodon_ui_ratatui::prelude::ui_main;
use eyre::Context;
use eyre::Result;
use std::fs::canonicalize;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::io::stdout;

pub async fn entrypoint(version: Version) -> Result<()> {
    let mut cmd = Cli::command();
    cmd = cmd.version(version.to_string());
    let cli = Cli::from_arg_matches(&cmd.get_matches())?;

    init_tracing(&cli);

    #[cfg(windows)]
    let _ = crate::windows_support::windows_ansi::enable_ansi_support();

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

    match cli.command {
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
            Commands::AddScanDir { mut dir } => {
                if !dir.is_absolute() {
                    dir = canonicalize(&dir)
                        .context(format!("failed to make path absolute: {}", dir.display()))?;
                }
                Config::modify_and_save_active_config(|config| {
                    config.scan_dirs.insert(dir);
                })?;
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
