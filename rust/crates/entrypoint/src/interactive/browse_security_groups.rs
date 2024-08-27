use anyhow::Result;
use cloud_terrasotodon_core_azure::prelude::fetch_all_security_groups;
use cloud_terrasotodon_core_fzf::pick_many;
use cloud_terrasotodon_core_fzf::Choice;
use cloud_terrasotodon_core_fzf::FzfArgs;
use itertools::Itertools;
use tracing::info;

pub async fn browse_security_groups() -> Result<()> {
    info!("Fetching security_groups");
    let security_groups = fetch_all_security_groups().await?;
    let security_groups = pick_many(FzfArgs {
        choices: security_groups
            .into_iter()
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
