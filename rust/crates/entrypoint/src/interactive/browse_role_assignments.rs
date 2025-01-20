use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::ensure_logged_in;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_assignments_v2;
use cloud_terrastodon_core_azure::prelude::fetch_all_role_definitions;
use cloud_terrastodon_core_azure::prelude::fetch_all_users;
use cloud_terrastodon_core_azure::prelude::PrincipalId;
use cloud_terrastodon_core_azure::prelude::RoleDefinition;
use cloud_terrastodon_core_azure::prelude::RoleDefinitionId;
use cloud_terrastodon_core_azure::prelude::ThinRoleAssignment;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use std::collections::HashMap;
use tokio::try_join;
use tracing::info;
use tracing::warn;

pub async fn browse_role_assignments() -> Result<()> {
    info!("Ensuring CLI is authenticated");
    ensure_logged_in().await?;

    info!("Fetching role assignments and definitions and principals");
    let (role_assignments, role_definitions, users) = try_join!(
        fetch_all_role_assignments_v2(),
        fetch_all_role_definitions(),
        fetch_all_users()
    )?;

    info!("Building lookup tables");
    let role_definition_lookup: HashMap<&RoleDefinitionId, &RoleDefinition> =
        role_definitions.iter().map(|ra| (&ra.id, ra)).collect();
    let user_lookup = users
        .iter()
        .map(|u| (u.id.into(), u))
        .collect::<HashMap<PrincipalId, _>>();

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
