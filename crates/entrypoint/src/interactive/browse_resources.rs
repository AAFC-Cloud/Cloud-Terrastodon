use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_resources_menu(tenant_id: AzureTenantId) -> Result<()> {
    info!("Fetching resources");
    let choices = fetch_all_resources(tenant_id)
        .await?
        .into_iter()
        .map(|x| Choice {
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
