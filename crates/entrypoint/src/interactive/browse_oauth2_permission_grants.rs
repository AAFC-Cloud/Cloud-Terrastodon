use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::pick_oauth2_permission_grants;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;

pub async fn browse_oauth2_permission_grants(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let (chosen, maybe_error) = pick_oauth2_permission_grants(auth_context, executor)
        .await
        .into_chosen_and_maybe_error()?;
    // todo!("fix sorting by service principal clientid, add id in parens");
    // todo!("commit changes");
    println!("You chose {} items", chosen.len());
    for item in chosen {
        println!("{item:#?}");
    }
    maybe_error
}
