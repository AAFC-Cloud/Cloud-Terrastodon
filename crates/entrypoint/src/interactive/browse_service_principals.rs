use cloud_terrastodon_azure::prelude::fetch_all_service_principals;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_service_principals() -> Result<()> {
    info!("Fetching service principals");
    let service_principals = fetch_all_service_principals().await?;
    let service_principals = PickerTui::new()
        .set_header("Service Principals")
        .pick_many(service_principals.into_iter().map(|sp| Choice {
            key: format!("{} {:64} {}", sp.id, sp.display_name, sp.app_id),
            value: sp,
        }))?;
    info!("You chose:");
    for sp in service_principals {
        println!("- {} {:64} {}", sp.id, sp.display_name, sp.app_id);
    }
    Ok(())
}
