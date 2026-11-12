use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraApplicationRegistrationListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_application_registrations(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<CliOutput> {
    info!(tenant_id = %authentication.tenant_id, "Fetching application registrations");
    let applications = EntraApplicationRegistrationListRequest { authentication }
        .execute(executor)
        .await?;
    let (applications, maybe_error) = PickerTui::<_>::new()
        .set_header("Application Registrations")
        .pick_many(
            applications.into_iter().map(|application| Choice {
                key: format!(
                    "{} {:64} {}",
                    application.id, application.display_name, application.app_id
                ),
                value: application,
            }),
            executor.cancellation(),
        )
        .await
        .into_chosen_and_maybe_error()?;
    info!(
        count = applications.len(),
        "You chose application registrations"
    );
    Ok(
        CliOutput::facet_with_text(applications, |applications, writer, cancellation| {
            for application in applications {
                cancellation.bail_if_cancelled()?;
                writeln!(
                    writer,
                    "- {} {:64} {}",
                    application.id, application.display_name, application.app_id
                )?;
            }
            Ok(())
        })
        .with_result(maybe_error),
    )
}
