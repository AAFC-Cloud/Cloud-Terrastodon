use 	anyhow::Result;
use azure::prelude::fetch_all_role_assignments_v2;
use azure::prelude::fetch_all_role_definitions;
use azure::prelude::fetch_all_users;
use azure::prelude::uuid::Uuid;
use azure::prelude::RoleDefinition;
use azure::prelude::RoleDefinitionId;
use azure::prelude::ThinRoleAssignment;
use azure::prelude::User;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use std::collections::HashMap;
use tokio::try_join;
use tracing::info;
use tracing::warn;

pub async fn browse_role_assignments() -> Result<()> {
    info!("Fetching role assignments and definitions and principals");
    let (role_assignments, role_definitions, users) = try_join!(
        fetch_all_role_assignments_v2(),
        fetch_all_role_definitions(),
        fetch_all_users()
    )?;

    info!("Building lookup tables");
    let role_definition_lookup: HashMap<&RoleDefinitionId, &RoleDefinition> =
        role_definitions.iter().map(|ra| (&ra.id, ra)).collect();
    let user_lookup: HashMap<&Uuid, &User> = users.iter().map(|u| (&u.id, u)).collect();

    info!("Building choices");
    let mut choices: Vec<Choice<ThinRoleAssignment>> = Vec::new();
    for ra in role_assignments {
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
        choices.push(Choice { key, value: ra });
    }

    info!("Picking");
    let chosen = pick_many(FzfArgs {
        choices,
        prompt: Some("Role assignments: ".to_string()),
        header: None,
    })?;

    info!("You chose:");
    for choice in chosen {
        info!("{:#?}", choice.value);
    }
    Ok(())
}