use std::env;

use crate::actions::prelude::apply_processed;
use crate::actions::prelude::build_group_imports;
use crate::actions::prelude::build_imports_from_existing;
use crate::actions::prelude::build_policy_imports;
use crate::actions::prelude::build_resource_group_imports;
use crate::actions::prelude::build_role_assignment_imports;
use crate::actions::prelude::clean_all;
use crate::actions::prelude::clean_imports;
use crate::actions::prelude::clean_processed;
use crate::actions::prelude::init_processed;
use crate::actions::prelude::jump_to_block;
use crate::actions::prelude::list_imports;
use crate::actions::prelude::perform_import;
use crate::actions::prelude::pim_activate;
use crate::actions::prelude::populate_cache;
use crate::actions::prelude::process_generated;
use anyhow::Result;
use azure::prelude::evaluate_policy_assignment_compliance;
use azure::prelude::remediate_policy_assignment;
use command::prelude::USE_TERRAFORM_FLAG_KEY;
use pathing_types::IgnoreDir;
use tokio::fs;
#[derive(Debug)]
pub enum Action {
    BuildPolicyImports,
    BuildGroupImports,
    BuildResourceGroupImports,
    BuildRoleAssignmentImports,
    BuildImportsFromExisting,
    PerformImport,
    ProcessGenerated,
    Clean,
    CleanImports,
    CleanProcessed,
    InitProcessed,
    ApplyProcessed,
    JumpToBlock,
    ListImports,
    RemediatePolicyAssignment,
    EvaluatePolicyAssignmentCompliance,
    UseTerraform,
    UseTofu,
    PopulateCache,
    PimActivate,
    Quit,
}

#[derive(Eq, PartialEq, Debug)]
pub enum ActionResult {
    QuitApplication,
    Continue,
    PauseAndContinue,
}

impl Action {
    pub fn name(&self) -> &str {
        match self {
            Action::BuildPolicyImports => "build imports - create policy_imports.tf",
            Action::BuildResourceGroupImports => "build imports - create resource_group_imports.tf",
            Action::BuildGroupImports => "build imports - create group_imports.tf",
            Action::BuildRoleAssignmentImports => "build imports - create role_assignments.tf",
            Action::BuildImportsFromExisting => "build imports - build from existing",
            Action::PerformImport => "perform import - tf plan -generate-config-out generated.tf",
            Action::ProcessGenerated => "processed - create from generated.tf",
            Action::Clean => "clean all",
            Action::CleanImports => "clean imports",
            Action::CleanProcessed => "clean processed",
            Action::InitProcessed => "processed - tf init",
            Action::ApplyProcessed => "processed - tf apply",
            Action::JumpToBlock => "jump to block",
            Action::ListImports => "list imports",
            Action::RemediatePolicyAssignment => "remediate policy assignment",
            Action::EvaluatePolicyAssignmentCompliance => "evaluate policy assignment complaince",
            Action::UseTerraform => "use terraform",
            Action::UseTofu => "use tofu",
            Action::PopulateCache => "populate cache",
            Action::PimActivate => "pim activate",
            Action::Quit => "quit",
        }
    }
    
    pub async fn invoke(&self) -> Result<ActionResult> {
        match self {
            Action::BuildPolicyImports => build_policy_imports().await?,
            Action::BuildGroupImports => build_group_imports().await?,
            Action::BuildResourceGroupImports => build_resource_group_imports().await?,
            Action::BuildRoleAssignmentImports => build_role_assignment_imports().await?,
            Action::BuildImportsFromExisting => build_imports_from_existing().await?,
            Action::PerformImport => perform_import().await?,
            Action::ProcessGenerated => process_generated().await?,
            Action::Clean => clean_all().await?,
            Action::CleanImports => clean_imports().await?,
            Action::CleanProcessed => clean_processed().await?,
            Action::InitProcessed => init_processed().await?,
            Action::ApplyProcessed => apply_processed().await?,
            Action::PimActivate => pim_activate().await?,
            Action::JumpToBlock => {
                jump_to_block(IgnoreDir::Processed.into()).await?;
                return Ok(ActionResult::Continue);
            }
            Action::ListImports => {
                list_imports().await?;
                return Ok(ActionResult::Continue);
            }
            Action::RemediatePolicyAssignment => remediate_policy_assignment().await?,
            Action::EvaluatePolicyAssignmentCompliance => {
                evaluate_policy_assignment_compliance().await?
            }
            Action::UseTerraform => env::set_var(USE_TERRAFORM_FLAG_KEY, "1"),
            Action::UseTofu => env::remove_var(USE_TERRAFORM_FLAG_KEY),
            Action::PopulateCache => populate_cache().await?,
            Action::Quit => return Ok(ActionResult::QuitApplication),
        }
        Ok(ActionResult::PauseAndContinue)
    }
    pub fn variants() -> Vec<Action> {
        vec![
            Action::UseTerraform,
            Action::UseTofu,
            Action::Clean,
            Action::CleanImports,
            Action::CleanProcessed,
            Action::Quit,
            Action::PopulateCache,
            Action::PimActivate,
            Action::RemediatePolicyAssignment,
            Action::EvaluatePolicyAssignmentCompliance,
            Action::BuildResourceGroupImports,
            Action::BuildRoleAssignmentImports,
            Action::BuildGroupImports,
            Action::BuildPolicyImports,
            Action::BuildImportsFromExisting,
            Action::ListImports,
            Action::PerformImport,
            Action::ProcessGenerated,
            Action::InitProcessed,
            Action::ApplyProcessed,
            Action::JumpToBlock,
        ]
    }

    /// Some actions don't make sense if files are missing from expected locations.
    pub async fn is_available(&self) -> bool {
        let all_exist = async |required_files| -> bool {
            for path in required_files {
                if !fs::try_exists(path).await.unwrap_or(false) {
                    return false;
                }
            }
            true
        };

        let any_exist = async |required_files| -> bool {
            for path in required_files {
                if fs::try_exists(path).await.unwrap_or(false) {
                    return true;
                }
            }
            false
        };

        match self {
            Action::PerformImport => {
                any_exist([
                    IgnoreDir::Imports.join("policy_imports.tf"),
                    IgnoreDir::Imports.join("group_imports.tf"),
                    IgnoreDir::Imports.join("resource_group_imports.tf"),
                    IgnoreDir::Imports.join("role_assignment_imports.tf"),
                    IgnoreDir::Imports.join("existing.tf"),
                ])
                .await
            }
            Action::ListImports => all_exist([IgnoreDir::Imports.into()]).await,
            Action::ProcessGenerated => all_exist([IgnoreDir::Imports.join("generated.tf")]).await,
            Action::Clean => all_exist([IgnoreDir::Root.into()]).await,
            Action::CleanImports => all_exist([IgnoreDir::Imports.into()]).await,
            Action::CleanProcessed => all_exist([IgnoreDir::Processed.into()]).await,
            Action::InitProcessed => all_exist([IgnoreDir::Processed.join("generated.tf")]).await,
            Action::ApplyProcessed => {
                all_exist([IgnoreDir::Processed.join(".terraform.lock.hcl")]).await
            }
            Action::JumpToBlock => all_exist([IgnoreDir::Processed.join("generated.tf")]).await,
            Action::UseTerraform => env::var(USE_TERRAFORM_FLAG_KEY).is_err(),
            Action::UseTofu => env::var(USE_TERRAFORM_FLAG_KEY).is_ok(),
            _ => true,
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
