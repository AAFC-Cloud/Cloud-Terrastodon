use super::azure_entra_role_assignment_browse_output::AzureEntraRoleAssignmentBrowseOutput;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::EntraUnifiedRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::info;

/// Arguments for browsing Entra role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraRoleAssignmentBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureEntraRoleAssignmentBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(tenant_id = %authentication.tenant_id, "Fetching Entra role assignments, definitions, and principals");
        let (rbac, principals) = executor
            .join2(
                async |child| {
                    EntraUnifiedRoleDefinitionsAndAssignmentsListRequest { authentication }
                        .execute(&child)
                        .await
                },
                async |child| {
                    EntraPrincipalListRequest { authentication }
                        .execute(&child)
                        .await
                },
            )
            .await?;

        let mut choices = rbac
            .role_assignments
            .values()
            .filter_map(|assignment| {
                rbac.role_definitions
                    .get(&assignment.role_definition_id)
                    .map(|definition| (assignment, definition))
            })
            .map(|(role_assignment, role_definition)| {
                let principal_name = principals
                    .get(&role_assignment.principal_id)
                    .map(|principal| Cow::Borrowed(principal.display_name()))
                    .unwrap_or_else(|| Cow::Borrowed("Unknown Principal"));
                Choice {
                    key: format!(
                        "{}\nrole definition: {}\nrole definition id: {}\ndirectory scope: {}\nresource scope: {}\nprincipal id: {}\nrole assignment id: {}",
                        principal_name,
                        role_definition.display_name,
                        role_definition.template_id,
                        role_assignment.directory_scope_id,
                        role_assignment.resource_scope,
                        role_assignment.principal_id,
                        role_assignment.id,
                    ),
                    value: (
                        role_assignment,
                        role_definition,
                        principals.get(&role_assignment.principal_id),
                    ),
                }
            })
            .collect::<Vec<_>>();
        choices.sort_unstable_by(|left, right| left.key.cmp(&right.key));

        let (chosen, maybe_error) = PickerTui::<_>::new()
            .set_header("Entra role assignments")
            .pick_many(choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;
        let chosen = chosen
            .into_iter()
            .map(|(role_assignment, role_definition, principal)| {
                AzureEntraRoleAssignmentBrowseOutput {
                    role_assignment: Arc::clone(role_assignment),
                    role_definition: Arc::clone(role_definition),
                    principal: principal.cloned(),
                }
            })
            .collect::<Vec<_>>();

        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
