use crate::EntraPrincipalId;
use crate::RoleAssignment;
use crate::RoleDefinition;
use crate::RoleDefinitionId;
use crate::fetch_all_entra_users;
use crate::fetch_all_role_assignments;
use crate::fetch_all_role_definitions;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

pub async fn get_role_assignment_choices(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<Choice<Arc<RoleAssignment>>>> {
    let auth_source = auth_context.auth_source()?;
    let (role_assignments, role_definitions, users) = executor
        .join3(
            |child| async move {
                fetch_all_role_assignments(auth_context.tenant_id, auth_source, &child).await
            },
            |child| async move {
                fetch_all_role_definitions(auth_context.tenant_id, auth_source, &child).await
            },
            |child| async move {
                fetch_all_entra_users(auth_context.tenant_id, auth_source, &child).await
            },
        )
        .await?;

    let role_definition_lookup: HashMap<&RoleDefinitionId, &RoleDefinition> = role_definitions
        .iter()
        .map(|ra| (&ra.id, ra.as_ref()))
        .collect();
    let user_lookup = users
        .iter()
        .map(|u| (u.id.into(), u))
        .collect::<HashMap<EntraPrincipalId, _>>();

    let mut choices: Vec<Choice<Arc<RoleAssignment>>> = Vec::new();
    for ra in role_assignments.iter() {
        let Some(rd) = role_definition_lookup.get(&ra.role_definition_id) else {
            warn!("Could not identify role definition for {ra:?}");
            continue;
        };
        let user = user_lookup.get(&ra.principal_id);
        let role_name = &rd.display_name;
        let scope = &ra.scope;
        let principal_display_name = user
            .map(|u| u.display_name.to_string())
            .unwrap_or_else(|| ra.principal_id.to_string());
        let key = format!("{role_name:41} {principal_display_name:36} {scope}");
        choices.push(Choice {
            key,
            value: Arc::clone(ra),
        });
    }

    Ok(choices)
}
