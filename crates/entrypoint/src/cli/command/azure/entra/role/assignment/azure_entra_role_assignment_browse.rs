use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_all_unified_role_definitions_and_assignments;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use serde_json::json;
use std::borrow::Cow;
use std::io::Write;
use tokio::try_join;
use tracing::info;

/// Arguments for browsing Entra role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraRoleAssignmentBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
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
                    value: (role_assignment, role_definition, principals.get(&role_assignment.principal_id)),
                }
            })
            .collect::<Vec<_>>();
        choices.sort_unstable_by(|left, right| left.key.cmp(&right.key));

        let chosen = PickerTui::new()
            .set_header("Entra role assignments")
            .pick_many(choices)?
            .into_iter()
            .map(|(role_assignment, role_definition, principal)| {
                json!({
                    "role_assignment": role_assignment,
                    "role_definition": role_definition,
                    "principal": principal,
                })
            })
            .collect::<Vec<_>>();

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
