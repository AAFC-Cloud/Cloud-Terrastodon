use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use azure::prelude::fetch_resource_groups;
use fzf::pick;
use fzf::FzfArgs;
use itertools::Itertools;
use std::path::PathBuf;
use tofu::prelude::AsTofuString;
use tokio::fs::create_dir_all;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub async fn build_resource_group_imports() -> Result<()> {
    info!("Fetching resource groups");
    let resource_groups = fetch_resource_groups().await?;

    let chosen = pick(FzfArgs {
        choices: resource_groups,
        many: true,
        prompt: Some("Groups to import: ".to_string()),
        header: None,
    })?;

    let imports = chosen.into_iter().map(|x| x.into()).collect_vec();

    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    // Prepare imports dir
    let imports_dir = PathBuf::from("ignore").join("imports");
    if !imports_dir.exists() {
        info!("Creating {:?}", imports_dir);
        create_dir_all(&imports_dir).await?;
    } else if !imports_dir.is_dir() {
        return Err(anyhow!("Path exists but isn't a dir!"))
            .context(imports_dir.to_string_lossy().into_owned());
    }

    // Write imports.tf
    let imports_path = imports_dir.join("resource_group_imports.tf");
    let mut imports_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&imports_path)
        .await?;
    info!("Writing {:?}", imports_path);
    imports_file
        .write_all(imports.as_tofu_string().as_bytes())
        .await?;

    Ok(())
}
