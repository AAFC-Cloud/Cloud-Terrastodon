use anyhow::Result;
use azure::prelude::fetch_all_users;
use fzf::pick_many;
use fzf::Choice;
use fzf::FzfArgs;
use itertools::Itertools;
use tracing::info;

pub async fn browse_users() -> Result<()> {
    info!("Fetching users");
    let users = fetch_all_users().await?;
    let users = pick_many(FzfArgs {
        choices: users
            .into_iter()
            .map(|u| Choice {
                display: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
                inner: u,
            })
            .collect_vec(),
        prompt: Some("Users: ".to_string()),
        header: None,
    })?;
    info!("You chose:");
    for user in users {
        info!("- {}", user.display);
    }
    Ok(())
}
