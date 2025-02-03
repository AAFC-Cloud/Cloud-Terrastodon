use cloud_terrastodon_core_azure::prelude::fetch_all_users;
use cloud_terrastodon_core_azure::prelude::UserId;
use cloud_terrastodon_core_user_input::prelude::prompt_line;
use eyre::Result;
use indexmap::IndexSet;
use std::collections::HashMap;
use tracing::info;

pub async fn bulk_user_id_lookup() -> Result<()> {
    info!("Enter the user IDs, one per line. Enter a blank line to proceed.");
    let mut user_ids = IndexSet::new();
    loop {
        let x = prompt_line("Enter user ID: ").await?;
        if x.is_empty() {
            break;
        } else {
            user_ids.insert(x.parse::<UserId>()?);
        }
    }

    let users = fetch_all_users()
        .await?
        .into_iter()
        .filter(|x| user_ids.contains(&x.id))
        .map(|x| (x.id.clone(), x))
        .collect::<HashMap<_,_>>();

    // we want to print in the order which the IDs were provided
    for user_id in user_ids {
        let Some(user) = users.get(&user_id) else {
            println!("{} - no user found", user_id);
            continue;
        };
        println!(
            "{} - {}",
            user_id,
            user.user_principal_name
        )
    }

    Ok(())
}
