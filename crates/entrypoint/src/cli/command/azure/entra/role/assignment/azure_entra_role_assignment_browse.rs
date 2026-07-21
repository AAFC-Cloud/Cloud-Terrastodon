use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Principal;
use cloud_terrastodon_azure::UnifiedRoleAssignment;
use cloud_terrastodon_azure::UnifiedRoleDefinition;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_all_unified_role_definitions_and_assignments;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::borrow::Cow;
use std::io::Write;
use tokio::try_join;
use tracing::info;

/// Arguments for browsing Entra role assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraRoleAssignmentBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

#[derive(facet::Facet)]
struct EntraRoleAssignmentBrowseOutput<'a> {
    role_assignment: &'a UnifiedRoleAssignment,
    role_definition: &'a UnifiedRoleDefinition,
    principal: Option<&'a Principal>,
}

impl AzureEntraRoleAssignmentBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Entra role assignments, definitions, and principals");
        let (rbac, principals) = try_join!(
            fetch_all_unified_role_definitions_and_assignments(tenant_id),
            fetch_all_principals(tenant_id)
        )?;

        let mut choices = rbac
            .iter_role_assignments()
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

        let chosen = PickerTui::<_>::new()
            .set_header("Entra role assignments")
            .pick_many(choices)
            .await?
            .into_iter()
            .map(
                |(role_assignment, role_definition, principal)| EntraRoleAssignmentBrowseOutput {
                    role_assignment,
                    role_definition,
                    principal,
                },
            )
            .collect::<Vec<_>>();

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
