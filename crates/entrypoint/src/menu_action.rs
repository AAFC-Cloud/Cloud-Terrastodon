use crate::cli::azure::resource_group::browse_resource_groups;
use crate::interactive::apply_processed;
use crate::interactive::azure_devops_project_import_wizard_menu;
use crate::interactive::browse_azure_devops_project_teams;
use crate::interactive::browse_azure_devops_projects;
use crate::interactive::browse_oauth2_permission_grants;
use crate::interactive::browse_policy_assignments;
use crate::interactive::browse_policy_definitions;
use crate::interactive::browse_resources_menu;
use crate::interactive::browse_role_assignments;
use crate::interactive::browse_security_groups;
use crate::interactive::browse_service_principals;
use crate::interactive::browse_storage_accounts;
use crate::interactive::browse_users;
use crate::interactive::build_group_imports;
use crate::interactive::build_imports_from_existing;
use crate::interactive::build_policy_imports;
use crate::interactive::build_resource_group_imports;
use crate::interactive::build_role_assignment_imports;
use crate::interactive::bulk_user_id_lookup;
use crate::interactive::clean_all_menu;
use crate::interactive::clean_imports;
use crate::interactive::clean_processed;
use crate::interactive::copy_azurerm_backend_menu;
use crate::interactive::create_import_block_for_role_assignment;
use crate::interactive::create_new_action_variant;
use crate::interactive::create_oauth2_permission_grants;
use crate::interactive::create_role_assignment_menu;
use crate::interactive::dump_tags;
use crate::interactive::dump_work_items;
use crate::interactive::find_resource_owners_menu;
use crate::interactive::init_processed;
use crate::interactive::jump_to_block;
use crate::interactive::list_imports;
use crate::interactive::open_dir;
use crate::interactive::pim_activate;
use crate::interactive::plan_processed;
use crate::interactive::prompt_azure_devops_authentication;
use crate::interactive::remove_oauth2_permission_grants;
use crate::interactive::resource_group_import_wizard_menu;
use crate::interactive::run_query_menu;
use crate::interactive::tag_empty_resource_group_menu;
use crate::interactive::tag_resources_menu;
use crate::noninteractive::dump_security_groups_as_json;
use crate::noninteractive::perform_import;
use crate::noninteractive::process_generated;
use crate::noninteractive::write_imports_for_all_resource_groups;
use crate::noninteractive::write_imports_for_all_role_assignments;
use crate::noninteractive::write_imports_for_all_security_groups;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::evaluate_policy_assignment_compliance;
use cloud_terrastodon_azure::remediate_policy_assignment;
use cloud_terrastodon_command::USE_TOFU_FLAG_KEY;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_pathing::AppDir;
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
    ResourceGroupImportWizard,
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
    CreateOAuth2PermissionGrants,
    AzureDevOpsProjectImportWizard,
    BrowseAzureDevOpsProjects,
    BrowseAzureDevOpsProjectTeams,
    BrowseServicePrincipals,
    BrowseStorageAccounts,
    CreateImportBlockForRoleAssignment,
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
            MenuAction::ResourceGroupImportWizard => "build imports - resource group import wizard",
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
            MenuAction::CreateOAuth2PermissionGrants => "create oauth2 permission grants",
            MenuAction::AzureDevOpsProjectImportWizard => {
                "build imports - azure devops project import wizard"
            }
            MenuAction::BrowseAzureDevOpsProjects => "browse azure devops projects",
            MenuAction::BrowseAzureDevOpsProjectTeams => "browse azure devops project teams",
            MenuAction::BrowseServicePrincipals => "browse service principals",
            MenuAction::BrowseStorageAccounts => "browse storage accounts",
            MenuAction::CreateImportBlockForRoleAssignment => {
                "create import block for role assignment"
            }
        }
    }
    pub async fn invoke(&self, executor: &CloudTerrastodonExecutor) -> Result<MenuActionResult> {
        let cancellation = executor.cancellation();
        match self {
            MenuAction::ResourceGroupImportWizard => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                resource_group_import_wizard_menu(authentication, executor).await?
            }
            MenuAction::CopyAzureRMBackend => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                copy_azurerm_backend_menu(authentication, executor).await?
            }
            MenuAction::BrowseResourceGroups => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_resource_groups(*authentication, executor)
                    .await?
                    .emit(None, cancellation)?
            }
            MenuAction::BrowseRoleAssignments => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_role_assignments(authentication, executor).await?
            }
            MenuAction::BuildAllImports => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                write_imports_for_all_resource_groups(authentication, executor).await?;
                write_imports_for_all_security_groups(authentication, executor).await?;
                write_imports_for_all_role_assignments(authentication, executor).await?;
            }
            MenuAction::BrowseUsers => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_users(*authentication, executor)
                    .await?
                    .emit(None, cancellation)?
            }
            MenuAction::BrowseSecurityGroups => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_security_groups(authentication, executor).await?
            }
            MenuAction::BuildPolicyImports => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                build_policy_imports(authentication, executor).await?
            }
            MenuAction::BuildGroupImports => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                build_group_imports(authentication, executor).await?
            }
            MenuAction::BuildResourceGroupImports => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                build_resource_group_imports(authentication, executor).await?
            }
            MenuAction::BuildRoleAssignmentImports => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                build_role_assignment_imports(authentication, executor).await?
            }
            MenuAction::BuildImportsFromExisting => {
                build_imports_from_existing(cancellation).await?
            }
            MenuAction::PerformImport => perform_import(cancellation).await?,
            MenuAction::ProcessGenerated => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                process_generated(authentication, executor).await?
            }
            MenuAction::Clean => clean_all_menu(executor.cache()?, cancellation).await?,
            MenuAction::CreateRoleAssignment => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                create_role_assignment_menu(authentication, executor).await?
            }
            MenuAction::CleanImports => clean_imports().await?,
            MenuAction::CleanProcessed => clean_processed().await?,
            MenuAction::InitProcessed => init_processed(cancellation).await?,
            MenuAction::ApplyProcessed => apply_processed(cancellation).await?,
            MenuAction::PlanProcessed => plan_processed(cancellation).await?,
            MenuAction::PimActivate => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                pim_activate(*authentication, executor).await?
            }
            MenuAction::JumpToBlock => {
                jump_to_block(AppDir::Processed.into(), cancellation).await?;
                return Ok(MenuActionResult::Continue);
            }
            MenuAction::ListImports => {
                list_imports(cancellation).await?;
                return Ok(MenuActionResult::Continue);
            }
            MenuAction::RemediatePolicyAssignment => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                remediate_policy_assignment(authentication, executor).await?
            }
            MenuAction::EvaluatePolicyAssignmentCompliance => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                evaluate_policy_assignment_compliance(authentication, executor).await?
            }
            MenuAction::UseTofu => unsafe { env::set_var(USE_TOFU_FLAG_KEY, "1") },
            MenuAction::UseTerraform => unsafe { env::remove_var(USE_TOFU_FLAG_KEY) },
            MenuAction::OpenDir => open_dir(cancellation).await?,
            MenuAction::Quit => return Ok(MenuActionResult::QuitApplication),
            MenuAction::TagEmptyResourceGroups => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                tag_empty_resource_group_menu(authentication, executor).await?
            }
            MenuAction::TagResources => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                tag_resources_menu(authentication, executor).await?
            }
            MenuAction::BrowseResources => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_resources_menu(authentication, executor).await?
            }
            MenuAction::DumpTags => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                dump_tags(authentication, executor).await?
            }
            MenuAction::ResourceGraphQuery => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                run_query_menu(authentication, executor).await?
            }
            MenuAction::FindResourceOwners => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                find_resource_owners_menu(authentication, executor).await?
            }
            MenuAction::CreateNewActionVariant => create_new_action_variant().await?,
            MenuAction::BrowsePolicyAssignments => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_policy_assignments(*authentication, executor)
                    .await?
                    .emit(None, cancellation)?
            }
            MenuAction::DumpSecurityGroups => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                dump_security_groups_as_json(authentication, executor).await?
            }
            MenuAction::BrowsePolicyDefinitions => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_policy_definitions(authentication, executor).await?
            }
            MenuAction::BulkUserIdLookup => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                bulk_user_id_lookup(authentication, executor).await?
            }
            MenuAction::DumpWorkItems => {
                let org_url: AzureDevOpsOrganizationUrl = executor
                    .cancellation()
                    .cancellable_scope(async |_scope| {
                        prompt_line("Azure DevOps organization URL: ")
                            .await?
                            .parse()
                    })
                    .await?;
                let authentication = prompt_azure_devops_authentication(&org_url, executor).await?;
                dump_work_items(&org_url, authentication, executor).await?
            }
            MenuAction::BrowseOAuth2PermissionGrants => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_oauth2_permission_grants(authentication, executor).await?
            }
            MenuAction::RemoveOAuth2PermissionGrants => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                remove_oauth2_permission_grants(authentication, executor).await?
            }
            MenuAction::CreateOAuth2PermissionGrants => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                create_oauth2_permission_grants(authentication, executor).await?
            }
            MenuAction::AzureDevOpsProjectImportWizard => {
                let org_url: AzureDevOpsOrganizationUrl = executor
                    .cancellation()
                    .cancellable_scope(async |_scope| {
                        prompt_line("Azure DevOps organization URL: ")
                            .await?
                            .parse()
                    })
                    .await?;
                let authentication = prompt_azure_devops_authentication(&org_url, executor).await?;
                azure_devops_project_import_wizard_menu(&org_url, authentication, executor).await?
            }
            MenuAction::BrowseAzureDevOpsProjects => {
                let org_url: AzureDevOpsOrganizationUrl = executor
                    .cancellation()
                    .cancellable_scope(async |_scope| {
                        prompt_line("Azure DevOps organization URL: ")
                            .await?
                            .parse()
                    })
                    .await?;
                let authentication = prompt_azure_devops_authentication(&org_url, executor).await?;
                browse_azure_devops_projects(&org_url, authentication, executor).await?
            }
            MenuAction::BrowseAzureDevOpsProjectTeams => {
                let org_url: AzureDevOpsOrganizationUrl = executor
                    .cancellation()
                    .cancellable_scope(async |_scope| {
                        prompt_line("Azure DevOps organization URL: ")
                            .await?
                            .parse()
                    })
                    .await?;
                let authentication = prompt_azure_devops_authentication(&org_url, executor).await?;
                browse_azure_devops_project_teams(&org_url, authentication, executor).await?
            }
            MenuAction::BrowseServicePrincipals => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_service_principals(*authentication, executor)
                    .await?
                    .emit(None, cancellation)?
            }
            MenuAction::BrowseStorageAccounts => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                browse_storage_accounts(authentication, executor).await?
            }
            MenuAction::CreateImportBlockForRoleAssignment => {
                let authentication = &AzureAuthenticationArgs::default().resolve(executor).await?;
                create_import_block_for_role_assignment(authentication, executor).await?
            }
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
                    AppDir::Imports.join("azure_devops_project_imports.tf"),
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
            MenuAction::UseTofu => env::var(USE_TOFU_FLAG_KEY).is_err(),
            MenuAction::UseTerraform => env::var(USE_TOFU_FLAG_KEY).is_ok(),
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
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_user_input::prompt_line;
