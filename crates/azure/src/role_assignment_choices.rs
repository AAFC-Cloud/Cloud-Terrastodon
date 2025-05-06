use crate::prelude::PrincipalId;
use crate::prelude::RoleDefinition;
use crate::prelude::RoleDefinitionId;
use crate::prelude::ThinRoleAssignment;
use crate::prelude::fetch_all_role_assignments;
use crate::prelude::fetch_all_role_definitions;
use crate::prelude::fetch_all_users;
use cloud_terrastodon_user_input::Choice;
use std::collections::HashMap;
use tokio::try_join;
use tracing::warn;

pub async fn get_role_assignment_choices() -> eyre::Result<Vec<Choice<ThinRoleAssignment>>> {
    let (role_assignments, role_definitions, users) = try_join!(
        fetch_all_role_assignments(),
        fetch_all_role_definitions(),
        fetch_all_users()
    )?;

    let role_definition_lookup: HashMap<&RoleDefinitionId, &RoleDefinition> =
        role_definitions.iter().map(|ra| (&ra.id, ra)).collect();
    let user_lookup = users
        .iter()
        .map(|u| (u.id.into(), u))
        .collect::<HashMap<PrincipalId, _>>();

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

    Ok(choices)
}
