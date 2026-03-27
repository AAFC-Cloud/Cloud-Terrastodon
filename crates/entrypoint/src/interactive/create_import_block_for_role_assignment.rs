use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_hcl::AsHclString;
use cloud_terrastodon_hcl::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl::HclImportBlock;
use cloud_terrastodon_hcl::HclProviderReference;
use cloud_terrastodon_hcl::ResourceBlockReference;
use cloud_terrastodon_hcl::Sanitizable;
use cloud_terrastodon_ui_ratatui::role_assignment_picker_app::RoleAssignmentPickerApp;
use cloud_terrastodon_ui_ratatui::role_assignment_picker_app::RoleAssignmentPickerAppResult;
use eyre::Result;
use eyre::bail;
use tracing::info;

pub async fn create_import_block_for_role_assignment(tenant_id: AzureTenantId) -> Result<()> {
    match RoleAssignmentPickerApp::new(tenant_id).run().await? {
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
                let import_block = HclImportBlock {
                    provider: HclProviderReference::Inherited,
                    id: role_assignment_id.expanded_form(),
                    to: ResourceBlockReference::AzureRM {
                        kind: AzureRmResourceBlockKind::RoleAssignment,
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
