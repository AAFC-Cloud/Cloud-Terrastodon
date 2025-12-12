use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_hcl::discovery::DiscoveryDepth;
use cloud_terrastodon_hcl::discovery::discover_hcl;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_hcl::reflow::reflow_hcl;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::{self};
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
    let hcl = discover_hcl(&workspace_path, DiscoveryDepth::Shallow).await?;
    let hcl = reflow_hcl(hcl).await?;

    // Write files
    let mut error_count = 0;
    for (path, contents) in hcl {
        error_count += HclWriter::new(path).overwrite(contents).await.is_err() as usize;
    }

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
