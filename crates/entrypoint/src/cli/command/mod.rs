pub mod add_work_dir;
pub mod azure;
pub mod azure_devops;
pub mod clean;
pub mod copy_results;
pub mod dump_azure_devops;
pub mod dump_everything;
pub mod egui;
pub mod get_path;
pub mod perform_code_generation_from_imports;
pub mod ratatui;
pub mod terraform;
pub mod write_all_imports;

pub use add_work_dir::AddWorkDirArgs;
pub use azure::AzureArgs;
pub use azure::AzureCommand;
pub use azure::AzureGroupArgs;
pub use azure::AzureGroupBrowseArgs;
pub use azure::AzureGroupCommand;
pub use azure::AzureGroupListArgs;
pub use azure_devops::AzureDevOpsArgs;
pub use azure_devops::AzureDevOpsCommand;
use clap::Subcommand;
pub use clean::CleanArgs;
pub use copy_results::CopyResultsArgs;
pub use dump_azure_devops::DumpAzureDevOpsArgs;
pub use dump_everything::DumpEverythingArgs;
pub use egui::EguiArgs;
use eyre::Result;
pub use get_path::GetPathArgs;
pub use perform_code_generation_from_imports::PerformCodeGenerationFromImportsArgs;
pub use ratatui::RatatuiArgs;
pub use terraform::TerraformArgs;
pub use terraform::TerraformCommand;
pub use write_all_imports::WriteAllImportsArgs;

/// All top-level commands for the cloud-terrastodon CLI.
#[derive(Subcommand, Debug)]
pub enum CloudTerrastodonCommand {
    /// Launch the Ratatui-based interface.
    Ratatui(RatatuiArgs),
    /// Launch the egui-based interface.
    Egui(EguiArgs),
    /// Remove generated artifacts.
    Clean(CleanArgs),
    /// Write Terraform import definitions for known resources.
    WriteAllImports(WriteAllImportsArgs),
    /// Perform code generation from existing import definitions.
    PerformCodeGenerationFromImports(PerformCodeGenerationFromImportsArgs),
    /// Dump all available data to disk for diagnostics.
    DumpEverything(DumpEverythingArgs),
    /// Dump Azure DevOps metadata to disk.
    DumpAzureDevOps(DumpAzureDevOpsArgs),
    /// Print the path to a well-known application directory.
    GetPath(GetPathArgs),
    /// Copy the latest run results to another location.
    CopyResults(CopyResultsArgs),
    /// Register a directory as a working directory.
    AddWorkDir(AddWorkDirArgs),
    /// Perform Terraform-specific operations.
    #[command(alias = "tf")]
    Terraform(TerraformArgs),
    /// Perform Azure DevOps-specific operations.
    AzureDevOps(AzureDevOpsArgs),
    /// Perform Azure-specific operations.
    #[command(alias = "az")]
    Azure(AzureArgs),
}

impl CloudTerrastodonCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            CloudTerrastodonCommand::Ratatui(args) => args.invoke().await,
            CloudTerrastodonCommand::Egui(args) => args.invoke().await,
            CloudTerrastodonCommand::Clean(args) => args.invoke().await,
            CloudTerrastodonCommand::WriteAllImports(args) => args.invoke().await,
            CloudTerrastodonCommand::PerformCodeGenerationFromImports(args) => args.invoke().await,
            CloudTerrastodonCommand::DumpEverything(args) => args.invoke().await,
            CloudTerrastodonCommand::DumpAzureDevOps(args) => args.invoke().await,
            CloudTerrastodonCommand::GetPath(args) => args.invoke().await,
            CloudTerrastodonCommand::CopyResults(args) => args.invoke().await,
            CloudTerrastodonCommand::AddWorkDir(args) => args.invoke().await,
            CloudTerrastodonCommand::Terraform(args) => args.invoke().await,
            CloudTerrastodonCommand::AzureDevOps(args) => args.invoke().await,
            CloudTerrastodonCommand::Azure(args) => args.invoke().await,
        }
    }
}
