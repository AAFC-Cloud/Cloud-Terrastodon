use cloud_terrastodon_azure::prelude::fetch_all_service_principals;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use itertools::Itertools;
use tracing::info;

pub async fn browse_service_principals() -> Result<()> {
    info!("Fetching service principals");
    let service_principals = fetch_all_service_principals().await?;
    let service_principals = pick_many(FzfArgs {
        choices: service_principals
            .into_iter()
            .map(|sp| Choice {
                key: format!("{} {:64} {}", sp.id, sp.display_name, sp.app_id),
                value: sp,
            })
            .collect_vec(),
        prompt: Some("Service Principals: ".to_string()),
        ..Default::default()
    })?;
    info!("You chose:");
    for sp in service_principals {
        println!("- {}", sp.key);
    }
    Ok(())
}
