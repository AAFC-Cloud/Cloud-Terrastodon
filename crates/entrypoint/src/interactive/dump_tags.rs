use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::ResourceGraphHelper;
use cloud_terrastodon_user_input::PickerTui;
use facet_json::RawJson;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::fs::try_exists;
use tokio::io::AsyncWriteExt;
use tracing::info;
use tracing::warn;

pub async fn dump_tags(tenant_id: AzureTenantId) -> eyre::Result<()> {
    let path = PathBuf::from("resource_tags.json");
    if try_exists(&path).await.unwrap_or(false) {
        let yes = "yes";
        let no = "no";
        if PickerTui::<_>::new()
            .set_header(format!(
                "Output path {} already exists, overwrite?",
                path.display()
            ))
            .pick_one(vec![yes, no]).await?
            == no
        {
            warn!("Chose not to overwrite, no action taken!");
            return Ok(());
        }
    }

    let data = ResourceGraphHelper::new(
        tenant_id,
        r#"
resources 
| union resourcecontainers
| project
    id,
    ['kind'] = type,
    name,
    display_name=properties.displayName,
    tags
"#,
        None,
    )
    .collect_all::<RawJson<'static>>()
    .await?;

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await?;

    file.write_all(cloud_terrastodon_command::to_vec_pretty(&data)?.as_slice())
        .await?;
    info!("Dumped resources to {}", path.display());
    warn!("YOU PROBABLY WANT TO GITIGNORE THIS!");
    Ok(())
}
