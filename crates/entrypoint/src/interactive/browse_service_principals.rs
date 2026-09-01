use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_service_principals(auth_context: &AzureTenantAuthContext) -> Result<()> {
    info!("Fetching service principals");
    let service_principals = fetch_all_service_principals(auth_context).await?;
    let service_principals = PickerTui::<_>::new()
        .set_header("Service Principals")
        .pick_many(service_principals.into_iter().map(|sp| Choice {
            key: format!("{} {:64} {}", sp.id, sp.display_name, sp.app_id),
            value: sp,
        }))
        .await?;
    info!("You chose:");
    for sp in service_principals {
        println!("- {} {:64} {}", sp.id, sp.display_name, sp.app_id);
    }
    Ok(())
}
