use chrono::Local;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::EntraSecurityGroupListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub async fn dump_security_groups_as_json(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    info!("Fetching security_groups");
    let security_groups = EntraSecurityGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let mut security_groups = security_groups.iter().collect::<Vec<_>>();
    security_groups.sort_by(|x, y| x.display_name.cmp(&y.display_name));
    let content =
        facet_json::to_string_pretty(&security_groups).map_err(|error| eyre::eyre!("{error:?}"))?;
    let date = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let destination_file = PathBuf::from(format!("Security Groups {date}.json"));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&destination_file)
        .await?;
    file.write_all(content.as_bytes()).await?;
    info!(
        "Wrote {} security groups to \"{}\"",
        security_groups.len(),
        destination_file.display()
    );
    Ok(())
}
