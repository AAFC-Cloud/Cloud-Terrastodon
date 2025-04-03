use clap::Parser;
use clap::Subcommand;
use cloud_terrastodon_core_pathing::AppDir;
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
    V2,
    Clean,
    WriteAllImports,
    PerformCodeGenerationFromImports,
    DumpEverything,
    GetPath { dir: AppDir },
    CopyResults { dest: PathBuf },
    AddScanDir { dir: PathBuf },
}
