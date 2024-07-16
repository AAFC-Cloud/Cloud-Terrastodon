use std::fmt::Debug;

use crate::menu::menu_loop;
use crate::noninteractive::prelude::perform_import;
use crate::noninteractive::prelude::process_generated;
use crate::noninteractive::prelude::write_imports_for_all_resource_groups;
use crate::prelude::Version;
use clap::CommandFactory;
use clap::FromArgMatches;
use clap::Parser;
use clap::Subcommand;
use pathing::AppDir;
use tokio::fs::remove_dir_all;
use tokio::io::stdout;
use tokio::io::AsyncWriteExt;
use tracing::info;
use tracing::warn;
#[derive(Parser, Debug)]
#[command(name = "cloud_terrastodon", about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Clean,
    WriteAllImports,
    PerformCodeGenerationFromImports,
    GetPath {
        dir: AppDir
    }
}

pub async fn main(version: Version) -> anyhow::Result<()> {
    // Set the version
    let mut cmd = Cli::command();
    cmd = cmd.version(version.to_string());

    // Parse the command-line arguments
    let cli = Cli::from_arg_matches(&cmd.get_matches())?;

    match cli.command {
        None => {
            menu_loop().await?;
        }
        Some(command) => match command {
            Commands::Clean => {
                for dir in AppDir::ok_to_clean() {
                    info!("Cleaning {dir}...");
                    if let Err(e) = remove_dir_all(dir.as_path_buf()).await {
                        warn!("Ignoring error encountered cleaning {dir}: {e:?}");
                    }
                }
            }
            Commands::WriteAllImports => {
                write_imports_for_all_resource_groups().await?;
            }
            Commands::PerformCodeGenerationFromImports => {
                perform_import().await?;
                process_generated().await?;
            }
            Commands::GetPath { dir } => {
                let mut out = stdout();
                out.write_all(dir.as_path_buf().as_os_str().as_encoded_bytes()).await?;
                out.flush().await?;
            }
        },
    }
    Ok(())
}
