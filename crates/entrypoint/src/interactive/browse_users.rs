use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_user_input::prelude::Choice;
use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick_many;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn browse_users() -> Result<()> {
    info!("Fetching users");
    let users = fetch_all_users().await?;
    let users = pick_many(FzfArgs {
        choices: users
            .into_iter()
            .map(|u| Choice {
                key: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
                value: u,
            })
            .collect_vec(),
        prompt: Some("Users: ".to_string()),
        ..Default::default()
    })?;
    info!("You chose:");
    for user in users {
        println!("- {}", user.key);
    }
    Ok(())
}
