use clap::Args;
use cloud_terrastodon_hcl::discovery::DiscoveryDepth;
use cloud_terrastodon_hcl::discovery::discover_hcl;
use cloud_terrastodon_hcl::prelude::HclWriter;
use cloud_terrastodon_hcl::reflow::reflow_hcl;
use cloud_terrastodon_pathing::Existy;
use eyre::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::debug;
use tracing::info;

/// Reflow generated Terraform source files.
#[derive(Args, Debug, Clone)]
pub struct TerraformReflowArgs {
    #[arg(default_value = ".")]
    pub source_dir: PathBuf,
    #[arg(
        long,
        default_value_t = false,
        help = "Recursively reflow source in subdirectories"
    )]
    pub recursive: bool,
}

impl TerraformReflowArgs {
    pub async fn invoke(self) -> Result<()> {
        let hcl = discover_hcl(&self.source_dir, DiscoveryDepth::Shallow).await?;
        let old_paths = hcl.keys().cloned().collect::<HashSet<_>>();

        info!(count = hcl.len(), "Discovered HCL files for reflowing");
        let hcl = reflow_hcl(hcl).await?;
        let new_paths = hcl.keys().cloned().collect::<HashSet<_>>();

        info!(count = hcl.len(), "Reflowed HCL files");
        for (path, contents) in hcl {
            debug!(path=%path.display(), "Writing reflowed HCL file");
            HclWriter::new(path)
                // .format_on_write()
                .overwrite(contents)
                .await?;
        }

        let removed_paths = old_paths.difference(&new_paths);
        let trash_dir = self.source_dir.join(".trash");
        for path in removed_paths {
            info!(path=%path.display(), "Removing obsolete HCL file");
            let path_relative_to_source_dir = path.strip_prefix(&self.source_dir)?;
            let base_new_path = trash_dir.join(path_relative_to_source_dir);
            base_new_path.ensure_parent_dir_exists().await?;

            // while new path exists, suffix with a number
            let mut candidate = base_new_path.clone();
            if candidate.exists_async().await? {
                let parent = candidate.parent().ok_or(eyre::eyre!(
                    "Could not determine parent for {}",
                    candidate.display()
                ))?;
                let stem = candidate
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let extension = candidate
                    .extension()
                    .map(|s| s.to_string_lossy().to_string());
                let mut suffix = 1u32;
                loop {
                    let new_name = match &extension {
                        Some(ext) => format!("{}_{}.{}", stem, suffix, ext),
                        None => format!("{}_{}", stem, suffix),
                    };
                    let next_candidate = parent.join(new_name);
                    if !next_candidate.exists_async().await? {
                        candidate = next_candidate;
                        break;
                    }
                    suffix = suffix.saturating_add(1);
                }
            }

            tokio::fs::rename(path, candidate).await?;
        }

        Ok(())
    }
}
