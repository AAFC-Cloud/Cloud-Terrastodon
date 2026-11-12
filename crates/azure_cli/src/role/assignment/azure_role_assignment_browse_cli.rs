use super::azure_role_assignment_browse_output::AzureRoleAssignmentBrowseOutput;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureRoleDefinitionsAndAssignmentsListRequest;
use cloud_terrastodon_azure::EntraPrincipalListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use itertools::Itertools;
use std::borrow::Cow;
use std::sync::Arc;
use tracing::info;

/// Arguments for browsing Azure role assignments.
#[derive(Facet, Debug, Clone)]
pub struct AzureRoleAssignmentBrowseArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureRoleAssignmentBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        info!("Fetching Azure role assignments and principals");
        let authentication = self.authentication.resolve(executor).await?;
        let (rbac, principals) = executor
            .join2(
                async |child| {
                    AzureRoleDefinitionsAndAssignmentsListRequest { authentication }
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

        info!(
            count = rbac.role_assignments.len(),
            "Fetched Azure role assignments"
        );
        info!(count = principals.len(), "Fetched Azure principals");

        let mut choices = Vec::new();
        for role_assignment in rbac.role_assignments.values() {
            let Some(role_definition) = rbac
                .role_definitions
                .get(&role_assignment.role_definition_id)
            else {
                continue;
            };
            let principal = principals.get(&role_assignment.principal_id);
            let principal_name = principal
                .map(|p| Cow::Borrowed(p.display_name()))
                .unwrap_or_else(|| Cow::Borrowed("Unknown Principal"));

            choices.push(Choice {
                key: format!(
                    "principal name: {}\nrole definition: {}\nrole definition id: {}\nscope: {}\nprincipal id: {}\nrole assignment id: {}",
                    principal_name,
                    role_definition.display_name,
                    role_definition.id.expanded_form(),
                    role_assignment.scope,
                    role_assignment.principal_id,
                    role_assignment.id.expanded_form()
                ),
                value: (
                    role_assignment,
                    role_definition,
                    principal,
                ),
            });
        }

        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many(choices, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;
        let chosen = chosen
            .into_iter()
            .map(
                |(role_assignment, role_definition, principal)| AzureRoleAssignmentBrowseOutput {
                    role_assignment: Arc::clone(role_assignment),
                    role_definition: Arc::clone(role_definition),
                    principal: principal.cloned(),
                },
            )
            .collect_vec();

        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}
