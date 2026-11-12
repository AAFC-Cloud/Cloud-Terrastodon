use crate::interactive::pim_activate_entra;
use crate::menu::press_enter_to_continue;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraCurrentUserRequest;
use cloud_terrastodon_azure::EntraUnifiedRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::RolePermissionAction;
use cloud_terrastodon_azure::UnifiedRoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_hcl::TerraformChangeAction;
use cloud_terrastodon_hcl::TerraformPlan;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Not;
use std::path::PathBuf;
use tracing::debug;

/// Reflow generated Terraform source files.
#[derive(Facet, Debug, Clone)]
pub struct TerraformApplyArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    #[facet(figue::positional, default = PathBuf::from("."))]
    pub source_dir: PathBuf,
}

impl TerraformApplyArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let cancellation = executor.cancellation();
        let authentication = self.authentication.resolve(executor).await?;
        // Generate the plan file
        let plan_file = "apply.tfplan";
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.use_run_dir(&self.source_dir);
        cmd.args(["plan", "-out", plan_file]);
        cmd.use_output_behaviour(OutputBehaviour::Display);
        cmd.should_announce(true);
        cmd.run_raw(cancellation).await?;

        // Verify the plan file exists
        let plan_file_path = self.source_dir.join(plan_file);
        if !tokio::fs::try_exists(&plan_file_path)
            .await
            .unwrap_or_default()
        {
            bail!(
                "Terraform plan file not found: {}",
                plan_file_path.display()
            );
        }

        // Convert the plan file to JSON
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.cancellation_safe();
        cmd.use_run_dir(&self.source_dir);
        cmd.should_announce(true);
        cmd.args(["show", "--json", plan_file]);
        let plan_json = cmd.run::<TerraformPlan>(cancellation).await?;

        // Identify RBAC roles required by the plan
        #[derive(Debug, Eq, PartialEq, Hash)]
        pub enum RequiredPermission {
            Entra(RolePermissionAction),
            // Arm(RolePermissions),
        }
        let mut required_roles = HashSet::new();
        for resource_change in &plan_json.resource_changes {
            let is_create_action = resource_change
                .change
                .actions
                .contains(&TerraformChangeAction::Create);
            match resource_change.r#type.as_ref() {
                "azuread_application_registration" if is_create_action => {
                    required_roles.insert(RequiredPermission::Entra(RolePermissionAction::new(
                        "microsoft.directory/applications/create",
                    )));
                }
                "azuread_user" if is_create_action => {
                    required_roles.insert(RequiredPermission::Entra(RolePermissionAction::new(
                        "microsoft.directory/users/create",
                    )));
                }
                _ => {}
            }
        }

        println!("This plan requires: {:#?}", required_roles);

        // Identify RBAC roles for the current principal
        let entra_rbac = EntraUnifiedRoleDefinitionsAndAssignmentsListRequest { authentication }
            .execute(executor)
            .await?;
        let current_user = EntraCurrentUserRequest { authentication }
            .execute(executor)
            .await?;
        let current_user_rbac = entra_rbac
            .iter_role_assignments()
            .filter_principal(&current_user.id)
            .collect_vec();

        // Prepare to check requirement satisfaction
        let mut requirement_satisfaction = required_roles
            .iter()
            .map(|req| (req, false))
            .collect::<HashMap<_, _>>();

        // Update satisfaction given current RBAC
        for (requirement, satisfied) in &mut requirement_satisfaction {
            match requirement {
                RequiredPermission::Entra(action) => {
                    for (_role_assignment, role_definition) in &current_user_rbac {
                        if role_definition.satisfies(std::slice::from_ref(action)) {
                            *satisfied = true;
                            debug!(
                                "Requirement {:?} satisfied by role {}",
                                requirement, role_definition.display_name
                            );
                            break;
                        }
                    }
                }
            }
        }

        // Identify unsatisfied requirements
        let unsatisfied_requirements = requirement_satisfaction
            .iter()
            .filter_map(|(req, satisfied)| satisfied.not().then_some(req))
            .collect_vec();

        // Perform PIM activations for unsatisfied requirements
        if !unsatisfied_requirements.is_empty() {
            println!(
                "The following requirements are not currently satisfied: {:#?}",
                unsatisfied_requirements
            );
            press_enter_to_continue().await?;
            pim_activate_entra(authentication, executor).await?;
        }

        // Apply the plan
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.use_run_dir(&self.source_dir);
        cmd.args(["apply", plan_file]);
        cmd.use_output_behaviour(OutputBehaviour::Display);
        cmd.should_announce(true);
        cmd.run_raw(cancellation).await?;

        Ok(())
    }
}
