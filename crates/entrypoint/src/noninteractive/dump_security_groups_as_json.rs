use chrono::Local;
use cloud_terrastodon_azure::prelude::fetch_all_security_groups;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub async fn dump_security_groups_as_json() -> eyre::Result<()> {
    info!("Fetching security_groups");
    let mut security_groups = fetch_all_security_groups().await?;
    security_groups.sort_by(|x, y| x.display_name.cmp(&y.display_name));
    let content = serde_json::to_string_pretty(&security_groups)?;
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
