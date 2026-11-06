use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::fetch_all_service_principals;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_service_principals(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    info!("Fetching service principals");
    let service_principals = fetch_all_service_principals(
        auth_context.tenant_id,
        auth_context.auth_source()?,
        executor,
    )
    .await?;
    let (service_principals, maybe_error) = PickerTui::<_>::new()
        .set_header("Service Principals")
        .pick_many(service_principals.iter().cloned().map(|sp| Choice {
            key: format!("{} {:64} {}", sp.id, sp.display_name, sp.app_id),
            value: sp,
        }))
        .await
        .into_chosen_and_maybe_error()?;
    info!("You chose:");
    for sp in &service_principals {
        println!("- {} {:64} {}", sp.id, sp.display_name, sp.app_id);
    }
    maybe_error
}
