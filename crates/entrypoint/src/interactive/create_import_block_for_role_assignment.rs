use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_hcl::prelude::AsHCLString;
use cloud_terrastodon_hcl::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl::prelude::HCLImportBlock;
use cloud_terrastodon_hcl::prelude::HCLProviderReference;
use cloud_terrastodon_hcl::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl::prelude::Sanitizable;
use cloud_terrastodon_ui_ratatui::role_assignment_picker_app::RoleAssignmentPickerApp;
use cloud_terrastodon_ui_ratatui::role_assignment_picker_app::RoleAssignmentPickerAppResult;
use eyre::Result;
use eyre::bail;
use tracing::info;

pub async fn create_import_block_for_role_assignment() -> Result<()> {
    match RoleAssignmentPickerApp::new().run().await? {
        RoleAssignmentPickerAppResult::Cancelled => {
            info!("Operation cancelled by user.");
            Ok(())
        }
        RoleAssignmentPickerAppResult::Some {
            chosen_role_assignment_ids,
            role_definitions_and_assignments,
            principals,
        } => {
            for role_assignment_id in chosen_role_assignment_ids {
                let Some(role_assignment) = role_definitions_and_assignments
                    .role_assignments
                    .get(&role_assignment_id)
                else {
                    bail!("Chosen role assignment ID should exist in map");
                };
                let principal_display_name = principals
                    .get(&role_assignment.principal_id)
                    .map(|principal| principal.display_name().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let import_block = HCLImportBlock {
                    provider: HCLProviderReference::Inherited,
                    id: role_assignment_id.expanded_form(),
                    to: ResourceBlockReference::AzureRM {
                        kind: AzureRMResourceBlockKind::RoleAssignment,
                        name: format!(
                            "{}_{}",
                            role_assignment.scope.short_form(),
                            principal_display_name
                        )
                        .sanitize(),
                    },
                };
                
                println!("{}", import_block.as_hcl_string());
            }

            Ok(())
        }
    }
}
