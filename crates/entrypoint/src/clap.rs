use clap::Parser;
use clap::Subcommand;
use cloud_terrastodon_pathing::AppDir;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cloud_terrastodon", about, long_about = None)]
pub struct Cli {
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,

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
    GetPath {
        dir: AppDir,
    },
    CopyResults {
        dest: PathBuf,
    },
    AddWorkDir {
        dir: PathBuf,
    },
    Terraform {
        #[command(subcommand)]
        command: TerraformCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum TerraformCommand {
    Import {
        #[arg(long, default_value = ".")]
        work_dir: PathBuf,
    },
}
