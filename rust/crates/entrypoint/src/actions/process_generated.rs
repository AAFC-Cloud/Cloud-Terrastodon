use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use pathing_types::Existy;
use pathing_types::IgnoreDir;
use std::path::Path;
use std::path::PathBuf;
use tofu::prelude::reflow_workspace;
use tokio::fs::OpenOptions;
use tokio::fs::{self};
use tokio::io::AsyncWriteExt;
use tracing::error;
use tracing::info;
use tracing::instrument;

#[instrument(level = "debug")]
pub async fn process_generated() -> Result<()> {
    // Determine output directory
    let out_dir: PathBuf = IgnoreDir::Processed.into();

    // Cleanup
    if !out_dir.exists() {
        fs::create_dir(&out_dir).await?;
    } else {
        // Cleanup the directory except for specified exclusions
        remove_dir_contents_except(&out_dir, &[".terraform", ".terraform.lock.hcl"]).await?;
    }

    // Read generated code
    let workspace_path: PathBuf = IgnoreDir::Imports.into();

    // Determine output files
    let files = reflow_workspace(&workspace_path, &out_dir).await?;

    // Write files
    let error_count = write_many_contents(files).await?;

    // Format the files
    CommandBuilder::new(CommandKind::Tofu)
        .should_announce(true)
        .use_run_dir(out_dir)
        .args(["fmt", "-recursive"])
        .run_raw()
        .await?;

    info!("Processing finished with {} problems.", error_count);

    Ok(())
}

async fn remove_dir_contents_except(dir: &Path, exclude: &[&str]) -> Result<(), std::io::Error> {
    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if exclude.contains(&file_name) {
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(&path).await?;
        } else {
            fs::remove_file(&path).await?;
        }
    }

    Ok(())
}

pub async fn write_many_contents(files: Vec<(impl AsRef<Path>, String)>) -> Result<usize> {
    let mut error_count = 0;
    // Write the files
    for (path, content) in files {
        let path = path.as_ref();

        // Ensure parent dir exists
        path.ensure_parent_dir_exists().await?;

        // Open the file
        let mut file = match OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
        {
            Ok(x) => x,
            Err(e) => {
                error!("Failed to open {:?}: {:?}", path, e);
                error_count += 1;
                continue;
            }
        };

        // Write the content
        file.write_all(content.as_bytes()).await?;
    }
    Ok(error_count)
}
