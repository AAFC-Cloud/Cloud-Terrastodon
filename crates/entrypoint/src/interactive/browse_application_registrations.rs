use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::fetch_all_application_registrations;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_application_registrations(tenant_id: AzureTenantId) -> Result<()> {
    info!(%tenant_id, "Fetching application registrations");
    let applications = fetch_all_application_registrations(tenant_id).await?;
    let applications = PickerTui::new()
        .set_header("Application Registrations")
        .pick_many(applications.into_iter().map(|application| Choice {
            key: format!(
                "{} {:64} {}",
                application.id, application.display_name, application.app_id
            ),
            value: application,
        }))?;
    info!(
        count = applications.len(),
        "You chose application registrations"
    );
    for application in applications {
        println!(
            "- {} {:64} {}",
            application.id, application.display_name, application.app_id
        );
    }
    Ok(())
}
