use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_hcl::prelude::reflow_workspace;
use eyre::Result;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::fs::{self};
use tokio::io::AsyncWriteExt;
use tracing::error;
use tracing::info;
use tracing::instrument;

#[instrument(level = "debug")]
pub async fn process_generated() -> Result<()> {
    // Determine output directory
    let out_dir: PathBuf = AppDir::Processed.into();

    // Cleanup
    if !out_dir.exists() {
        fs::create_dir(&out_dir).await?;
    } else {
        // Cleanup the directory except for specified exclusions
        remove_dir_contents_except(&out_dir, &[".terraform", ".terraform.lock.hcl"]).await?;
    }

    // Read generated code
    let workspace_path: PathBuf = AppDir::Imports.into();

    // Determine output files
    let reflowed = reflow_workspace(&workspace_path).await?;

    // Write files
    let error_count = write_many_contents(reflowed.get_file_contents(&out_dir)?).await?;

    // Format the files
    CommandBuilder::new(CommandKind::Terraform)
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
