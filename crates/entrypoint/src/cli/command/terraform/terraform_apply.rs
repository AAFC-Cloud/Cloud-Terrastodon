use clap::Args;
use cloud_terrastodon_azure::prelude::RolePermissionAction;
use cloud_terrastodon_azure::prelude::UnifiedRoleDefinitionsAndAssignmentsIterTools;
use cloud_terrastodon_azure::prelude::fetch_all_unified_role_definitions_and_assignments;
use cloud_terrastodon_azure::prelude::fetch_current_user;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_hcl::prelude::TerraformChangeAction;
use cloud_terrastodon_hcl::prelude::TerraformPlan;
use eyre::Result;
use eyre::bail;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Not;
use std::path::PathBuf;
use tracing::debug;

use crate::interactive::prelude::pim_activate_entra;
use crate::menu::press_enter_to_continue;

/// Reflow generated Terraform source files.
#[derive(Args, Debug, Clone)]
pub struct TerraformApplyArgs {
    #[arg(default_value = ".")]
    pub source_dir: PathBuf,
}

impl TerraformApplyArgs {
    pub async fn invoke(self) -> Result<()> {
        // Generate the plan file
        let plan_file = "apply.tfplan";
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.use_run_dir(&self.source_dir);
        cmd.args(["plan", "-out", plan_file]);
        cmd.use_output_behaviour(OutputBehaviour::Display);
        cmd.should_announce(true);
        cmd.run_raw().await?;

        // Verify the plan file exists
        let plan_file_path = self.source_dir.join(plan_file);
        if tokio::fs::try_exists(&plan_file_path)
            .await
            .unwrap_or_default()
            != true
        {
            bail!(
                "Terraform plan file not found: {}",
                plan_file_path.display()
            );
        }

        // Convert the plan file to JSON
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.use_run_dir(&self.source_dir);
        cmd.should_announce(true);
        cmd.args(["show", "--json", plan_file]);
        let plan_json = cmd.run::<TerraformPlan>().await?;

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
        let entra_rbac = fetch_all_unified_role_definitions_and_assignments().await?;
        let current_user = fetch_current_user().await?;
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
                        if role_definition.satisfies(&[action.clone()]) {
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
            pim_activate_entra().await?;
        }

        // Apply the plan
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.use_run_dir(&self.source_dir);
        cmd.args(["apply", plan_file]);
        cmd.use_output_behaviour(OutputBehaviour::Display);
        cmd.should_announce(true);
        cmd.run_raw().await?;

        Ok(())
    }
}
