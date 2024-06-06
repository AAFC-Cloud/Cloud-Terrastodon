use crate::actions::prelude::apply_processed;
use crate::actions::prelude::build_group_imports;
use crate::actions::prelude::build_imports_from_existing;
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
use pathing_types::IgnoreDir;
use tokio::fs;
use tracing::instrument;
#[derive(Debug)]
pub enum Action {
    BuildPolicyImports,
    BuildGroupImports,
    BuildResourceGroupImports,
    BuildImportsFromExisting,
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
            Action::BuildPolicyImports => "build imports - create policy_imports.tf",
            Action::BuildResourceGroupImports => "build imports - create resource_group_imports.tf",
            Action::BuildGroupImports => "build imports - create group_imports.tf",
            Action::BuildImportsFromExisting => "build imports - build from existing",
            Action::PerformImport => "perform import - tf plan -generate-config-out generated.tf",
            Action::ProcessGenerated => "processed - create from generated.tf",
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
            Action::BuildImportsFromExisting => build_imports_from_existing().await,
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
            Action::Clean,
            Action::CleanImports,
            Action::CleanProcessed,
            Action::BuildResourceGroupImports,
            Action::BuildGroupImports,
            Action::BuildPolicyImports,
            Action::BuildImportsFromExisting,
            Action::PerformImport,
            Action::ProcessGenerated,
            Action::InitProcessed,
            Action::ApplyProcessed,
            Action::JumpToBlock,
        ]
    }
    pub fn should_pause(&self) -> bool {
        !matches!(self, Action::JumpToBlock)
    }

    /// Some actions don't make sense if files are missing from expected locations.
    pub async fn is_available(&self) -> bool {
        let all_exist = async |required_files| -> bool {
            for path in required_files {
                if !fs::try_exists(path).await.unwrap_or(false) {
                    return false;
                }
            }
            return true;
        };

        let any_exist = async |required_files| -> bool {
            for path in required_files {
                if fs::try_exists(path).await.unwrap_or(false) {
                    return true;
                }
            }
            return false;
        };

        match self {
            Action::PerformImport => {
                any_exist([
                    IgnoreDir::Imports.join("policy_imports.tf"),
                    IgnoreDir::Imports.join("group_imports.tf"),
                    IgnoreDir::Imports.join("resource_group_imports.tf"),
                    IgnoreDir::Imports.join("existing.tf"),
                ])
                .await
            }
            Action::ProcessGenerated => all_exist([IgnoreDir::Imports.join("generated.tf")]).await,
            Action::Clean => all_exist([IgnoreDir::Root.into()]).await,
            Action::CleanImports => all_exist([IgnoreDir::Imports.into()]).await,
            Action::CleanProcessed => all_exist([IgnoreDir::Processed.into()]).await,
            Action::InitProcessed => all_exist([IgnoreDir::Processed.join("generated.tf")]).await,
            Action::ApplyProcessed => {
                all_exist([IgnoreDir::Processed.join(".terraform.lock.hcl")]).await
            }
            Action::JumpToBlock => all_exist([IgnoreDir::Processed.join("generated.tf")]).await,
            _ => true,
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
