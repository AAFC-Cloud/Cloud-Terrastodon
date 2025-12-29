use cloud_terrastodon_azure::prelude::Resource;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_resources_menu() -> Result<()> {
    info!("Fetching resources");
    let choices = fetch_all_resources().await?.into_iter().map(|x| Choice {
        key: x.id.expanded_form().to_owned(),
        value: x,
    });
    let chosen: Vec<Resource> = PickerTui::new()
        .set_header("Resources")
        .pick_many(choices)?;
    info!("You chose:");
    for value in chosen {
        info!("{:#?}", value);
    }
    Ok(())
}
