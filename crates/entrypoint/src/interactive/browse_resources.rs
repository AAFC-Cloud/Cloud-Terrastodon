use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn browse_resources_menu() -> Result<()> {
    info!("Fetching resources");
    let choices = fetch_all_resources()
        .await?
        .into_iter()
        .map(|x| Choice {
            key: x.id.expanded_form().to_owned(),
            value: x,
        })
        .collect_vec();
    let chosen = pick_many(FzfArgs {
        choices,
        prompt: Some("Resources: ".to_string()),
        ..Default::default()
    })?;
    info!("You chose:");
    for choice in chosen {
        info!("{:#?}", choice.value);
    }
    Ok(())
}
