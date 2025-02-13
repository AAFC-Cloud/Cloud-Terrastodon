use crate::interactive::prelude::apply_processed;
use crate::interactive::prelude::browse_oauth2_permission_grants;
use crate::interactive::prelude::browse_policy_assignments;
use crate::interactive::prelude::browse_policy_definitions;
use crate::interactive::prelude::browse_resource_groups;
use crate::interactive::prelude::browse_resources_menu;
use crate::interactive::prelude::browse_role_assignments;
use crate::interactive::prelude::browse_security_groups;
use crate::interactive::prelude::browse_users;
use crate::interactive::prelude::build_group_imports;
use crate::interactive::prelude::build_imports_from_existing;
use crate::interactive::prelude::build_policy_imports;
use crate::interactive::prelude::build_resource_group_imports;
use crate::interactive::prelude::build_role_assignment_imports;
use crate::interactive::prelude::bulk_user_id_lookup;
use crate::interactive::prelude::clean_all_menu;
use crate::interactive::prelude::clean_imports;
use crate::interactive::prelude::clean_processed;
use crate::interactive::prelude::copy_azurerm_backend_menu;
use crate::interactive::prelude::create_new_action_variant;
use crate::interactive::prelude::create_role_assignment_menu;
use crate::interactive::prelude::dump_tags;
use crate::interactive::prelude::dump_work_items;
use crate::interactive::prelude::find_resource_owners_menu;
use crate::interactive::prelude::init_processed;
use crate::interactive::prelude::jump_to_block;
use crate::interactive::prelude::list_imports;
use crate::interactive::prelude::open_dir;
use crate::interactive::prelude::pim_activate;
use crate::interactive::prelude::plan_processed;
use crate::interactive::prelude::populate_cache;
use crate::interactive::prelude::remove_oauth2_permission_grants;
use crate::interactive::prelude::resource_group_import_wizard_menu;
use crate::interactive::prelude::run_query_menu;
use crate::interactive::prelude::tag_empty_resource_group_menu;
use crate::interactive::prelude::tag_resources_menu;
use crate::noninteractive::prelude::dump_security_groups_as_json;
use crate::noninteractive::prelude::perform_import;
use crate::noninteractive::prelude::process_generated;
use crate::noninteractive::prelude::write_imports_for_all_resource_groups;
use crate::noninteractive::prelude::write_imports_for_all_role_assignments;
use crate::noninteractive::prelude::write_imports_for_all_security_groups;
use cloud_terrastodon_core_azure::prelude::evaluate_policy_assignment_compliance;
use cloud_terrastodon_core_azure::prelude::remediate_policy_assignment;
use cloud_terrastodon_core_command::prelude::USE_TERRAFORM_FLAG_KEY;
use cloud_terrastodon_core_pathing::AppDir;
use eyre::Result;
use itertools::Itertools;
use std::env;
use std::path::PathBuf;
use strum::VariantArray;
use tokio::fs;
pub const THIS_FILE: &str = file!();
#[derive(Debug, VariantArray)]
pub enum MenuAction {
    BuildPolicyImports,
    BuildGroupImports,
    BuildResourceGroupImports,
    BuildRoleAssignmentImports,
    ResourceGraphQuery,
    BuildImportsFromExisting,
    BuildImportsWizard,
    BrowseResourceGroups,
    BrowseRoleAssignments,
    BrowseUsers,
    BrowseSecurityGroups,
    BrowseResources,
    DumpTags,
    TagResources,
    PerformImport,
    ProcessGenerated,
    Clean,
    BuildAllImports,
    CleanImports,
    CleanProcessed,
    CopyAzureRMBackend,
    CreateRoleAssignment,
    InitProcessed,
    ApplyProcessed,
    PlanProcessed,
    JumpToBlock,
    ListImports,
    FindResourceOwners,
    RemediatePolicyAssignment,
    EvaluatePolicyAssignmentCompliance,
    UseTerraform,
    UseTofu,
    PopulateCache,
    PimActivate,
    OpenDir,
    TagEmptyResourceGroups,
    Quit,
    CreateNewActionVariant,
    BrowsePolicyAssignments,
    DumpSecurityGroups,
    BrowsePolicyDefinitions,
    BulkUserIdLookup,
    DumpWorkItems,
    BrowseOAuth2PermissionGrants,
    RemoveOAuth2PermissionGrants,
}
#[derive(Eq, PartialEq, Debug)]
pub enum MenuActionResult {
    QuitApplication,
    Continue,
    PauseAndContinue,
}
impl MenuAction {
    pub fn name(&self) -> &str {
        match self {
            MenuAction::BuildImportsWizard => "build imports - import wizard",
            MenuAction::BuildAllImports => "build imports - import all",
            MenuAction::CopyAzureRMBackend => "copy azurerm backend",
            MenuAction::BrowseResourceGroups => "browse resource groups",
            MenuAction::BrowseUsers => "browse users",
            MenuAction::BrowseRoleAssignments => "browse role assignments",
            MenuAction::BrowseSecurityGroups => "browse security groups",
            MenuAction::BuildPolicyImports => "build imports - create policy_imports.tf",
            MenuAction::BuildResourceGroupImports => {
                "build imports - create resource_group_imports.tf"
            }
            MenuAction::BuildGroupImports => "build imports - create group_imports.tf",
            MenuAction::BuildRoleAssignmentImports => "build imports - create role_assignments.tf",
            MenuAction::BuildImportsFromExisting => "build imports - build from existing",
            MenuAction::PerformImport => {
                "perform import - tf plan -generate-config-out generated.tf"
            }
            MenuAction::ProcessGenerated => "processed - create from generated.tf",
            MenuAction::Clean => "clean all",
            MenuAction::CleanImports => "clean imports",
            MenuAction::CleanProcessed => "clean processed",
            MenuAction::InitProcessed => "processed - tf init",
            MenuAction::ApplyProcessed => "processed - tf apply",
            MenuAction::PlanProcessed => "processed - tf plan",
            MenuAction::JumpToBlock => "jump to block",
            MenuAction::ListImports => "list imports",
            MenuAction::RemediatePolicyAssignment => "remediate policy assignment",
            MenuAction::EvaluatePolicyAssignmentCompliance => {
                "evaluate policy assignment complaince"
            }
            MenuAction::CreateRoleAssignment => "create role assignment",
            MenuAction::UseTerraform => "use terraform",
            MenuAction::UseTofu => "use tofu",
            MenuAction::PopulateCache => "populate cache",
            MenuAction::PimActivate => "pim activate",
            MenuAction::OpenDir => "open dir",
            MenuAction::Quit => "quit",
            MenuAction::TagEmptyResourceGroups => "tag empty resource groups",
            MenuAction::TagResources => "tag resources",
            MenuAction::BrowseResources => "browse resources",
            MenuAction::DumpTags => "dump tags",
            MenuAction::ResourceGraphQuery => "resource graph query",
            MenuAction::FindResourceOwners => "find resource owners",
            MenuAction::CreateNewActionVariant => "create new action variant",
            MenuAction::BrowsePolicyAssignments => "Browse policy assignments",
            MenuAction::DumpSecurityGroups => "dump security groups as json",
            MenuAction::BrowsePolicyDefinitions => "browse policy definitions",
            MenuAction::BulkUserIdLookup => "bulk user id lookup",
            MenuAction::DumpWorkItems => "dump work items",
            MenuAction::BrowseOAuth2PermissionGrants => "browse oauth2 permission grants",
            MenuAction::RemoveOAuth2PermissionGrants => "remove oauth2 permission grants",
        }
    }
    pub async fn invoke(&self) -> Result<MenuActionResult> {
        match self {
            MenuAction::BuildImportsWizard => resource_group_import_wizard_menu().await?,
            MenuAction::CopyAzureRMBackend => copy_azurerm_backend_menu().await?,
            MenuAction::BrowseResourceGroups => browse_resource_groups().await?,
            MenuAction::BrowseRoleAssignments => browse_role_assignments().await?,
            MenuAction::BuildAllImports => {
                write_imports_for_all_resource_groups().await?;
                write_imports_for_all_security_groups().await?;
                write_imports_for_all_role_assignments().await?;
            }
            MenuAction::BrowseUsers => browse_users().await?,
            MenuAction::BrowseSecurityGroups => browse_security_groups().await?,
            MenuAction::BuildPolicyImports => build_policy_imports().await?,
            MenuAction::BuildGroupImports => build_group_imports().await?,
            MenuAction::BuildResourceGroupImports => build_resource_group_imports().await?,
            MenuAction::BuildRoleAssignmentImports => build_role_assignment_imports().await?,
            MenuAction::BuildImportsFromExisting => build_imports_from_existing().await?,
            MenuAction::PerformImport => perform_import().await?,
            MenuAction::ProcessGenerated => process_generated().await?,
            MenuAction::Clean => clean_all_menu().await?,
            MenuAction::CreateRoleAssignment => create_role_assignment_menu().await?,
            MenuAction::CleanImports => clean_imports().await?,
            MenuAction::CleanProcessed => clean_processed().await?,
            MenuAction::InitProcessed => init_processed().await?,
            MenuAction::ApplyProcessed => apply_processed().await?,
            MenuAction::PlanProcessed => plan_processed().await?,
            MenuAction::PimActivate => pim_activate().await?,
            MenuAction::JumpToBlock => {
                jump_to_block(AppDir::Processed.into()).await?;
                return Ok(MenuActionResult::Continue);
            }
            MenuAction::ListImports => {
                list_imports().await?;
                return Ok(MenuActionResult::Continue);
            }
            MenuAction::RemediatePolicyAssignment => remediate_policy_assignment().await?,
            MenuAction::EvaluatePolicyAssignmentCompliance => {
                evaluate_policy_assignment_compliance().await?
            }
            MenuAction::UseTerraform => env::set_var(USE_TERRAFORM_FLAG_KEY, "1"),
            MenuAction::UseTofu => env::remove_var(USE_TERRAFORM_FLAG_KEY),
            MenuAction::PopulateCache => populate_cache().await?,
            MenuAction::OpenDir => open_dir().await?,
            MenuAction::Quit => return Ok(MenuActionResult::QuitApplication),
            MenuAction::TagEmptyResourceGroups => tag_empty_resource_group_menu().await?,
            MenuAction::TagResources => tag_resources_menu().await?,
            MenuAction::BrowseResources => browse_resources_menu().await?,
            MenuAction::DumpTags => dump_tags().await?,
            MenuAction::ResourceGraphQuery => run_query_menu().await?,
            MenuAction::FindResourceOwners => find_resource_owners_menu().await?,
            MenuAction::CreateNewActionVariant => create_new_action_variant().await?,
            MenuAction::BrowsePolicyAssignments => browse_policy_assignments().await?,
            MenuAction::DumpSecurityGroups => dump_security_groups_as_json().await?,
            MenuAction::BrowsePolicyDefinitions => browse_policy_definitions().await?,
            MenuAction::BulkUserIdLookup => bulk_user_id_lookup().await?,
            MenuAction::DumpWorkItems => dump_work_items().await?,
            MenuAction::BrowseOAuth2PermissionGrants => browse_oauth2_permission_grants().await?,
            MenuAction::RemoveOAuth2PermissionGrants => remove_oauth2_permission_grants().await?,
        }
        Ok(MenuActionResult::PauseAndContinue)
    }
    /// Some actions don't make sense if files are missing from expected locations.
    pub async fn is_available(&self) -> bool {
        async fn all_exist(required_files: impl IntoIterator<Item = PathBuf>) -> bool {
            for path in required_files {
                if !fs::try_exists(path).await.unwrap_or(false) {
                    return false;
                }
            }
            true
        }
        async fn any_exist(required_files: impl IntoIterator<Item = PathBuf>) -> bool {
            for path in required_files {
                if fs::try_exists(path).await.unwrap_or(false) {
                    return true;
                }
            }
            false
        }
        match self {
            MenuAction::PerformImport => {
                any_exist([
                    AppDir::Imports.join("policy_imports.tf"),
                    AppDir::Imports.join("group_imports.tf"),
                    AppDir::Imports.join("resource_group_imports.tf"),
                    AppDir::Imports.join("role_assignment_imports.tf"),
                    AppDir::Imports.join("existing.tf"),
                ])
                .await
            }
            MenuAction::ListImports => all_exist([AppDir::Imports.into()]).await,
            MenuAction::ProcessGenerated => all_exist([AppDir::Imports.join("generated.tf")]).await,
            MenuAction::Clean => {
                any_exist(
                    AppDir::ok_to_clean()
                        .into_iter()
                        .map(|x| x.as_path_buf())
                        .collect_vec(),
                )
                .await
            }
            MenuAction::CleanImports => all_exist([AppDir::Imports.into()]).await,
            MenuAction::CleanProcessed => all_exist([AppDir::Processed.into()]).await,
            MenuAction::InitProcessed => all_exist([AppDir::Processed.join("generated.tf")]).await,
            MenuAction::ApplyProcessed | MenuAction::PlanProcessed => {
                all_exist([AppDir::Processed.join(".terraform.lock.hcl")]).await
            }
            MenuAction::JumpToBlock => all_exist([AppDir::Processed.join("generated.tf")]).await,
            MenuAction::UseTerraform => env::var(USE_TERRAFORM_FLAG_KEY).is_err(),
            MenuAction::UseTofu => env::var(USE_TERRAFORM_FLAG_KEY).is_ok(),
            #[cfg(not(debug_assertions))]
            MenuAction::CreateNewActionVariant => false,
            _ => true,
        }
    }
}
impl std::fmt::Display for MenuAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
