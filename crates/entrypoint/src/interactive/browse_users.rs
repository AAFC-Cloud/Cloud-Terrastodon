use cloud_terrastodon_azure::pick_entra_users;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::PickResultExt;
use eyre::Result;
use tracing::info;

pub async fn browse_users(auth_context: &AzureTenantAuthContext) -> Result<()> {
    let (chosen, maybe_error) = pick_entra_users(auth_context)
        .await
        .into_chosen_and_maybe_error()?;
    if chosen.is_empty() {
        info!("You chose no users.");
    } else {
        info!("You chose:");
        for user in chosen {
            println!(
                "- {} {:64} {}",
                user.id, user.display_name, user.user_principal_name
            );
        }
    }
    maybe_error
}
