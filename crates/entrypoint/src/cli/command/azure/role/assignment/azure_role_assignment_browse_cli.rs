use clap::Args;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_principals;
use cloud_terrastodon_azure::prelude::fetch_all_role_definitions_and_assignments;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use itertools::Itertools;
use serde_json::json;
use std::borrow::Cow;
use std::io::Write;
use tokio::try_join;
use tracing::info;

/// Arguments for browsing Azure role assignments.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleAssignmentBrowseArgs {}

impl AzureRoleAssignmentBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching Azure role assignments and principals");
        let (rbac, principals) = try_join!(
            fetch_all_role_definitions_and_assignments(),
            fetch_all_principals()
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
                value: (role_assignment, role_definition, principal),
            });
        }

        let chosen = PickerTui::new().pick_many(choices)?
        .into_iter()
        .map(|(role_assignment, role_definition, principal)| {
            json!({"role_assignment": role_assignment, "role_definition": role_definition, "principal": principal})
        })
        .collect_vec();

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
