use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_resources;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_resources_menu(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    info!("Fetching resources");
    let choices = fetch_all_resources(auth_context, executor)
        .await?
        .into_iter()
        .map(|x| Choice {
            key: x.id.expanded_form().to_owned(),
            value: x,
        });
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .set_header("Resources")
        .pick_many(choices)
        .await
        .into_chosen_and_maybe_error()?;
    info!("You chose:");
    for value in &chosen {
        info!("{:#?}", value);
    }
    maybe_error
}
