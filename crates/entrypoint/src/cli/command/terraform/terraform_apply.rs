use clap::Args;
use cloud_terrastodon_azure::prelude::GovernanceRoleDefinitionName;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_hcl::prelude::TerraformChangeAction;
use cloud_terrastodon_hcl::prelude::TerraformPlan;
use eyre::Result;
use eyre::bail;
use std::collections::HashSet;
use std::path::PathBuf;

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
            Entra(GovernanceRoleDefinitionName),
            // Arm(RolePermissions),
        }
        let mut required_roles = HashSet::new();
        for resource_change in &plan_json.resource_changes {
            if resource_change.r#type == "azuread_application_registration" {
                if resource_change
                    .change
                    .actions
                    .contains(&TerraformChangeAction::Create)
                {
                    required_roles.insert(RequiredPermission::Entra(
                        GovernanceRoleDefinitionName::try_new("Application Administrator")?,
                    ));
                }
            }
        }

        println!("This plan requires: {:#?}", required_roles);

        // TODO: finish implementing

        // Identify RBAC roles for the current principal

        // Identify gaps
        // Propose activations
        // Apply activations
        // Apply the plan

        Ok(())
    }
}
