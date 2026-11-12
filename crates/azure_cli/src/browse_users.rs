use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraUserPickRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::PickResultExt;
use eyre::Result;
use tracing::info;

pub async fn browse_users(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<CliOutput> {
    let (chosen, maybe_error) = EntraUserPickRequest { authentication }
        .execute(executor)
        .await
        .into_chosen_and_maybe_error()?;
    if chosen.is_empty() {
        info!("You chose no users.");
    } else {
        info!("You chose:");
    }
    Ok(
        CliOutput::facet_with_text(chosen, |users, writer, cancellation| {
            for user in users {
                cancellation.bail_if_cancelled()?;
                writeln!(
                    writer,
                    "- {} {:64} {}",
                    user.id, user.display_name, user.user_principal_name
                )?;
            }
            Ok(())
        })
        .with_result(maybe_error),
    )
}
