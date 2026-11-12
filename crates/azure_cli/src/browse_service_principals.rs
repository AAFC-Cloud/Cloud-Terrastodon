use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraServicePrincipalListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_service_principals(
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<CliOutput> {
    info!("Fetching service principals");
    let service_principals = EntraServicePrincipalListRequest { authentication }
        .execute(executor)
        .await?;
    let (service_principals, maybe_error) = PickerTui::<_>::new()
        .set_header("Service Principals")
        .pick_many(
            service_principals.iter().cloned().map(|sp| Choice {
                key: format!("{} {:64} {}", sp.id, sp.display_name, sp.app_id),
                value: sp,
            }),
            executor.cancellation(),
        )
        .await
        .into_chosen_and_maybe_error()?;
    info!("You chose:");
    Ok(
        CliOutput::facet_with_text(service_principals, |principals, writer, cancellation| {
            for sp in principals {
                cancellation.bail_if_cancelled()?;
                writeln!(writer, "- {} {:64} {}", sp.id, sp.display_name, sp.app_id)?;
            }
            Ok(())
        })
        .with_result(maybe_error),
    )
}
