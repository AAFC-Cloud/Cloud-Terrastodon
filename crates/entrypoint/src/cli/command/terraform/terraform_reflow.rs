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
use std::path::Path;
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

    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "main.tf",
        value_name = "FILENAME",
        conflicts_with = "flat",
        help = "Write all reflowed HCL into a single Terraform file, defaulting to main.tf"
    )]
    pub single_file: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = false,
        help = "Keep the original flat layout where each block stays in its own file"
    )]
    pub flat: bool,

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
        let hcl = discover_hcl(&self.source_dir, self.discovery_depth()).await?;
        let old_paths = hcl.keys().cloned().collect::<HashSet<_>>();
        let single_file_path = self
            .single_file
            .as_ref()
            .map(|single_file| self.resolve_single_file_path(single_file));

        info!(count = hcl.len(), "Discovered HCL files for reflowing");
    let hcl = reflow_hcl(tenant_id, hcl, self.full, single_file_path, self.flat).await?;
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

    fn discovery_depth(&self) -> DiscoveryDepth {
        if self.recursive {
            DiscoveryDepth::Recursive
        } else {
            DiscoveryDepth::Shallow
        }
    }

    fn resolve_single_file_path(&self, single_file: &Path) -> PathBuf {
        if single_file.is_absolute() {
            single_file.to_path_buf()
        } else {
            self.source_dir.join(single_file)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct ParseArgs {
        #[command(flatten)]
        args: TerraformReflowArgs,
    }

    #[test]
    fn parses_single_file_flag_without_value_as_main_tf() {
        let args = ParseArgs::parse_from(["reflow", "--single-file"]).args;

        assert_eq!(args.single_file, Some(PathBuf::from("main.tf")));
    }

    #[test]
    fn parses_single_file_flag_with_value() {
        let args = ParseArgs::parse_from(["reflow", "--single-file", "merged.tf"]).args;

        assert_eq!(args.single_file, Some(PathBuf::from("merged.tf")));
    }

    #[test]
    fn parses_flat_flag() {
        let args = ParseArgs::parse_from(["reflow", "--flat"]).args;

        assert!(args.flat);
    }

    #[test]
    fn rejects_flat_with_single_file() {
        assert!(ParseArgs::try_parse_from(["reflow", "--flat", "--single-file"]).is_err());
    }
}
