use crate::actions::prelude::apply_processed;
use crate::actions::prelude::build_group_imports;
use crate::actions::prelude::build_policy_imports;
use crate::actions::prelude::build_resource_group_imports;
use crate::actions::prelude::clean;
use crate::actions::prelude::clean_imports;
use crate::actions::prelude::clean_processed;
use crate::actions::prelude::init_processed;
use crate::actions::prelude::jump_to_block;
use crate::actions::prelude::perform_import;
use crate::actions::prelude::process_generated;
use anyhow::Result;
use tokio::fs;
use tracing::instrument;
#[derive(Debug)]
pub enum Action {
    BuildPolicyImports,
    BuildGroupImports,
    BuildResourceGroupImports,
    PerformImport,
    ProcessGenerated,
    Clean,
    CleanImports,
    CleanProcessed,
    InitProcessed,
    ApplyProcessed,
    JumpToBlock,
}
impl Action {
    pub fn name(&self) -> &str {
        match self {
            Action::BuildPolicyImports => "imports - create policy_imports.tf",
            Action::BuildResourceGroupImports => "imports - create resource_group_imports.tf",
            Action::BuildGroupImports => "imports - create group_imports.tf",
            Action::PerformImport => "imports - tf plan -generate-config-out generated.tf",
            Action::ProcessGenerated => "processed - create",
            Action::Clean => "clean all",
            Action::CleanImports => "clean imports",
            Action::CleanProcessed => "clean processed",
            Action::InitProcessed => "processed - tf init",
            Action::ApplyProcessed => "processed - tf apply",
            Action::JumpToBlock => "jump to block",
        }
    }
    #[instrument]
    pub async fn invoke(&self) -> Result<()> {
        match self {
            Action::BuildPolicyImports => build_policy_imports().await,
            Action::BuildGroupImports => build_group_imports().await,
            Action::BuildResourceGroupImports => build_resource_group_imports().await,
            Action::PerformImport => perform_import().await,
            Action::ProcessGenerated => process_generated().await,
            Action::Clean => clean().await,
            Action::CleanImports => clean_imports().await,
            Action::CleanProcessed => clean_processed().await,
            Action::InitProcessed => init_processed().await,
            Action::ApplyProcessed => apply_processed().await,
            Action::JumpToBlock => jump_to_block().await,
        }
    }
    pub fn variants() -> Vec<Action> {
        vec![
            Action::JumpToBlock,
            Action::ApplyProcessed,
            Action::InitProcessed,
            Action::ProcessGenerated,
            Action::PerformImport,
            Action::BuildPolicyImports,
            Action::BuildGroupImports,
            Action::BuildResourceGroupImports,
            Action::CleanProcessed,
            Action::CleanImports,
            Action::Clean,
        ]
    }
    pub fn should_pause(&self) -> bool {
        !matches!(self, Action::JumpToBlock)
    }

    /// Some actions don't make sense if files are missing from expected locations.
    pub async fn is_available(&self) -> bool {
        match self {
            Action::PerformImport => {
                fs::try_exists("ignore/imports/policy_imports.tf")
                    .await
                    .unwrap_or(false)
                    || fs::try_exists("ignore/imports/group_imports.tf")
                        .await
                        .unwrap_or(false)
                    || fs::try_exists("ignore/imports/resource_group_imports.tf")
                        .await
                        .unwrap_or(false)
            }
            Action::ProcessGenerated => fs::try_exists("ignore/imports/generated.tf")
                .await
                .unwrap_or(false),
            Action::Clean => fs::try_exists("ignore").await.unwrap_or(false),
            Action::CleanImports => fs::try_exists("ignore/imports").await.unwrap_or(false),
            Action::CleanProcessed => fs::try_exists("ignore/processed").await.unwrap_or(false),
            Action::InitProcessed => fs::try_exists("ignore/processed/generated.tf")
                .await
                .unwrap_or(false),
            Action::ApplyProcessed => fs::try_exists("ignore/processed/.terraform.lock.hcl")
                .await
                .unwrap_or(false),
            Action::JumpToBlock => fs::try_exists("ignore/processed/generated.tf")
                .await
                .unwrap_or(false),
            _ => true,
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
