use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_hcl::HclWriter;
use cloud_terrastodon_hcl::discovery::DiscoveryDepth;
use cloud_terrastodon_hcl::discovery::discover_hcl;
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
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    #[arg(
        long,
        default_value_t = false,
        help = "Run full reflow, including principal lookup and principal id comment insertion"
    )]
    pub full: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Keep obsolete files in the .trash directory instead of deleting them"
    )]
    pub keep_trash: bool,

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
        let tenant_id = self.tenant.resolve().await?;
        let hcl = discover_hcl(&self.source_dir, DiscoveryDepth::Shallow).await?;
        let old_paths = hcl.keys().cloned().collect::<HashSet<_>>();

        info!(count = hcl.len(), "Discovered HCL files for reflowing");
        let hcl = reflow_hcl(tenant_id, hcl, self.full).await?;
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
        for path in removed_paths {
            if self.keep_trash {
                let trash_dir = self.source_dir.join(".trash");
                info!(path=%path.display(), trash_dir=%trash_dir.display(), "Moving obsolete HCL file to trash");
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
            } else {
                info!(path=%path.display(), "Deleting obsolete HCL file");
                tokio::fs::remove_file(path).await?;
            }
        }

        Ok(())
    }
}
