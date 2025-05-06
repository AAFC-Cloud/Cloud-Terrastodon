use cloud_terrastodon_azure_types::prelude::Group;
use cloud_terrastodon_user_input::Choice;
use itertools::Itertools;

use crate::prelude::fetch_all_security_groups;

pub async fn get_security_group_choices() -> eyre::Result<Vec<Choice<Group>>> {
    let security_groups = fetch_all_security_groups().await?;
    let choices = security_groups
        .into_iter()
        .sorted_by(|x, y| x.display_name.cmp(&y.display_name))
        .map(|u| Choice {
            key: format!("{} {}", u.id, u.display_name),
            value: u,
        })
        .collect_vec();
    Ok(choices)
}
