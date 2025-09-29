use clap::Parser;
use clap::Subcommand;
use cloud_terrastodon_pathing::AppDir;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cloud_terrastodon", about, long_about = None)]
pub struct Cli {
    /// Enable debug logging, including backtraces on panics
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,

    /// Write a json file of the structured logs to the current directory
    #[arg(long, global = true, default_value = "false")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Ratatui,
    Egui,
    Clean,
    WriteAllImports,
    PerformCodeGenerationFromImports,
    DumpEverything,
    DumpAzureDevOps,
    GetPath {
        dir: AppDir,
    },
    CopyResults {
        dest: PathBuf,
    },
    AddWorkDir {
        dir: PathBuf,
    },
    #[command(alias = "tf")]
    Terraform {
        #[command(subcommand)]
        command: TerraformCommand,
    },
    AzureDevOps {
        #[command(subcommand)]
        command: AzureDevOpsCommand,
    },
    Azure {
        #[command(subcommand)]
        command: AzureCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum TerraformCommand {
    Import {
        #[arg(long, default_value = ".")]
        work_dir: PathBuf,
    },
    /// Identify if any providers have been specified as required but are not being used.
    ///
    /// Identify if any providers are not using the latest version.
    Audit {
        #[arg(default_value = ".")]
        source_dir: PathBuf,
        #[arg(
            long,
            default_value_t = false,
            help = "Recursively audit subdirectories"
        )]
        recursive: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum AzureDevOpsCommand {
    Audit,
}

#[derive(Subcommand, Debug)]
pub enum AzureCommand {
    Audit,
}
