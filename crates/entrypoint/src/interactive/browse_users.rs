use cloud_terrastodon_azure::prelude::User;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_users() -> Result<()> {
    info!("Fetching users");
    let users = fetch_all_users().await?;
    let users = PickerTui::<User>::new(users.into_iter().map(|u| Choice {
        key: format!("{} {:64} {}", u.id, u.display_name, u.user_principal_name),
        value: u,
    }))
    .set_header("Users")
    .pick_many()?;
    info!("You chose:");
    for user in users {
        println!(
            "- {} {:64} {}",
            user.id, user.display_name, user.user_principal_name
        );
    }
    Ok(())
}
