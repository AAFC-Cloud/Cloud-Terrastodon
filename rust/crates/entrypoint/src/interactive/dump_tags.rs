use cloud_terrastodon_core_azure::prelude::ResourceGraphHelper;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::fs::try_exists;
use tokio::io::AsyncWriteExt;
use tracing::info;
use tracing::warn;

pub async fn dump_tags() -> eyre::Result<()> {
    let path = PathBuf::from("resource_tags.json");
    if try_exists(&path).await.unwrap_or(false) {
        let yes = "yes";
        let no = "no";
        if pick(FzfArgs {
            choices: vec![&yes, &no],
            prompt: Some(format!(
                "Output path {} already exists, overwrite?",
                path.display()
            )),
            ..Default::default()
        })? == &no
        {
            warn!("Chose not to overwrite, no action taken!");
            return Ok(());
        }
    }

    let data = ResourceGraphHelper::new(
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
        CacheBehaviour::None,
    )
    .collect_all::<Value>()
    .await?;

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await?;

    file.write_all(serde_json::to_string_pretty(&data)?.as_bytes())
        .await?;
    info!("Dumped resources to {}", path.display());
    warn!("YOU PROBABLY WANT TO GITIGNORE THIS!");
    Ok(())
}
