use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::fetch_all_security_groups;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use tracing::info;

pub async fn browse_security_groups() -> Result<()> {
    info!("Fetching security_groups");
    let security_groups = fetch_all_security_groups().await?;
    let security_groups = pick_many(FzfArgs {
        choices: security_groups
            .into_iter()
            .sorted_by(|x,y| x.display_name.cmp(&y.display_name))
            .map(|u| Choice {
                key: format!("{} {}", u.id, u.display_name),
                value: u,
            })
            .collect_vec(),
        prompt: Some("security groups: ".to_string()),
        header: None,
    })?;
    info!("You chose:");
    for security_group in security_groups {
        info!("{:#?}", security_group.value);
    }
    Ok(())
}
