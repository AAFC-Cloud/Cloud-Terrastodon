use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Principal;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_principals;
use cloud_terrastodon_azure::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use itertools::Itertools;
use std::borrow::Cow;
use std::io::Write;
use tokio::try_join;
use tracing::info;

/// Arguments for browsing Azure role assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleAssignmentBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

#[derive(facet::Facet)]
struct RoleAssignmentBrowseOutput<'a> {
    role_assignment: &'a RoleAssignment,
    role_definition: &'a RoleDefinition,
    principal: Option<&'a Principal>,
}

impl AzureRoleAssignmentBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure role assignments and principals");
        let tenant_id = self.tenant.resolve().await?;
        let (rbac, principals) = try_join!(
            fetch_all_role_definitions_and_assignments(tenant_id),
            fetch_all_principals(tenant_id)
        )?;

        info!(
            count = rbac.role_assignments.len(),
            "Fetched Azure role assignments"
        );
        info!(count = principals.len(), "Fetched Azure principals");

        let mut choices = Vec::new();
        for (role_assignment, role_definition) in rbac.iter_role_assignments() {
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

        let chosen = PickerTui::<_>::new()
            .pick_many(choices).await?
            .into_iter()
            .map(
                |(role_assignment, role_definition, principal)| RoleAssignmentBrowseOutput {
                    role_assignment,
                    role_definition,
                    principal,
                },
            )
            .collect_vec();

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
