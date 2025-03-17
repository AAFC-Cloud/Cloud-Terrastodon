use cloud_terrastodon_core_tofu_types::prelude::ProviderAvailability;
use cloud_terrastodon_core_tofu_types::prelude::TFProviderHostname;
use cloud_terrastodon_core_tofu_types::prelude::TFProviderNamespace;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderKind;
use eyre::bail;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env::{self};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::io::AsyncReadExt;
use tracing::debug;

/// Helper to address https://github.com/hashicorp/terraform/issues/33321
/// 
/// > "terraform providers mirror" skip downloading packages that are already present in the mirror directory
pub struct ProviderManager {
    pub tf_plugin_cache_dir: PathBuf,
}

impl ProviderManager {
    pub fn get_default_tf_plugin_cache_dir() -> eyre::Result<PathBuf> {
        if let Ok(path) = env::var("TF_PLUGIN_CACHE_DIR") {
            return Ok(PathBuf::from(path));
        };

        #[allow(deprecated)]
        // https://github.com/rust-lang/libs-team/issues/372
        let home_dir = env::home_dir();

        if let Some(home_dir) = home_dir {
            let mut path = PathBuf::from(home_dir);
            path.push(".terraform.d/plugin-cache");
            return Ok(path);
        }

        bail!(
            "Failed to acquire TF_PLUGIN_CACHE_DIR from environment variable and failed to find home directory"
        );
    }
    pub fn try_new() -> eyre::Result<Self> {
        Ok(ProviderManager {
            tf_plugin_cache_dir: Self::get_default_tf_plugin_cache_dir()?,
        })
    }

    pub async fn list_cached_providers(&self) -> eyre::Result<HashSet<ProviderAvailability>> {
        let mut rtn = HashSet::default();
        let mut cache_children = tokio::fs::read_dir(&self.tf_plugin_cache_dir).await?;
        while let Some(registry) = cache_children.next_entry().await? {
            let mut registry_children = tokio::fs::read_dir(&registry.path()).await?;
            while let Some(author) = registry_children.next_entry().await? {
                let mut author_children = tokio::fs::read_dir(&author.path()).await?;
                while let Some(provider) = author_children.next_entry().await? {
                    // Find index.json to parse compressed providers
                    let index_json = tokio::fs::OpenOptions::new()
                        .read(true)
                        .create(false)
                        .open(provider.path().join("index.json"))
                        .await;
                    if let Ok(mut index_json_file) = index_json {
                        let mut index_json_str = String::new();
                        index_json_file.read_to_string(&mut index_json_str).await?;
                        // Parse JSON into structs
                        #[derive(Debug, Deserialize)]
                        struct IndexJson {
                            pub versions: HashMap<String, HashMap<(), ()>>,
                        }
                        let index_json: IndexJson = serde_json::from_str(&index_json_str)?;
                        for version in index_json.versions.into_keys() {
                            let registry_name = registry.file_name();
                            let registry_name = registry_name.to_string_lossy().into_owned();
                            let author_name = author.file_name();
                            let author_name = author_name.to_string_lossy().into_owned();
                            let provider_name = provider.file_name();
                            let provider_name = provider_name.to_string_lossy().into_owned();
                            debug!("Found json {registry_name}/{author_name}/{version}");
                            rtn.insert(ProviderAvailability {
                                hostname: TFProviderHostname(registry_name),
                                namespace: TFProviderNamespace(author_name),
                                kind: TofuProviderKind::from_str(&provider_name)?,
                                version: version.parse()?,
                            });
                        }
                    }

                    // Iterate directories to parse uncompressed providers
                    let mut provider_children = tokio::fs::read_dir(&provider.path()).await?;
                    while let Some(version) = provider_children.next_entry().await? {
                        if version.file_type().await?.is_dir() {
                            let mut version_children = tokio::fs::read_dir(&version.path()).await?;
                            while let Some(platform) = version_children.next_entry().await? {
                                if platform.file_type().await?.is_dir() {
                                    let mut platform_children =
                                        tokio::fs::read_dir(&platform.path()).await?;
                                    while let Some(file) = platform_children.next_entry().await? {
                                        if file.file_type().await?.is_file()
                                            && PathBuf::from(file.path())
                                                .extension()
                                                .filter(|x| *x == "exe")
                                                .is_some()
                                        {
                                            let registry_name = registry.file_name();
                                            let registry_name =
                                                registry_name.to_string_lossy().into_owned();
                                            let author_name = author.file_name();
                                            let author_name =
                                                author_name.to_string_lossy().into_owned();
                                            let provider_name = provider.file_name();
                                            let provider_name =
                                                provider_name.to_string_lossy().into_owned();
                                            let version_name = version.file_name();
                                            let version_name =
                                                version_name.to_string_lossy().into_owned();
                                            let platform_name = platform.file_name();
                                            let platform_name =
                                                platform_name.to_string_lossy().into_owned();
                                            let exe_name = file.file_name();
                                            let exe_name = exe_name.to_string_lossy().into_owned();
                                            debug!(
                                                "Found exe {registry_name}/{author_name}/{provider_name}/{version_name}/{platform_name}/{exe_name}",
                                            );
                                            rtn.insert(ProviderAvailability {
                                                hostname: TFProviderHostname(registry_name),
                                                namespace: TFProviderNamespace(author_name),
                                                kind: TofuProviderKind::from_str(&provider_name)?,
                                                version: version_name.parse()?,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(rtn)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::ProviderManager;
    use crate::prelude::TofuImporter;
    use crate::writer::TofuWriter;
    use cloud_terrastodon_core_command::prelude::CommandBuilder;
    use cloud_terrastodon_core_command::prelude::CommandKind;
    use cloud_terrastodon_core_command::prelude::OutputBehaviour;
    use cloud_terrastodon_core_pathing::AppDir;
    use cloud_terrastodon_core_pathing::Existy;
    use cloud_terrastodon_core_tofu_types::prelude::AsTofuString;
    use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformBlock;
    use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformRequiredProvidersBlock;
    use eyre::bail;
    use hcl::edit::structure::Block;
    use hcl::edit::structure::Body;
    use tempfile::Builder;
    use tokio::task::JoinSet;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let provider_manager = ProviderManager::try_new()?;
        let found = provider_manager.list_cached_providers().await?;
        dbg!(found);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    pub async fn install_missing_providers() -> eyre::Result<()> {
        let required_providers = TofuTerraformRequiredProvidersBlock::try_from(
            r#"
                required_providers {
                    azurerm = {
                        source = "hashicorp/azurerm"
                        version = ">=4.18.0"
                    }
                }
            "#
            .parse::<Body>()?
            .into_blocks()
            .next()
            .unwrap(),
        )?;
        let provider_manager = ProviderManager::try_new()?;
        let available_providers = provider_manager.list_cached_providers().await?;
        let missing_providers = required_providers.identify_missing(&available_providers);
        if missing_providers.0.is_empty() {
            bail!("All required providers are already available");
        }

        let terraform_block = TofuTerraformBlock {
            backend: None,
            required_providers: Some(missing_providers),
            other: vec![],
        };
        let terraform_block: Block = terraform_block.into();
        let boilerplate_tf = terraform_block.as_tofu_string();
        let app_temp_dir = AppDir::Temp.as_path_buf();
        app_temp_dir.ensure_dir_exists().await?;
        let temp_dir = Builder::new().tempdir_in(&app_temp_dir)?;
        let boilerplate_tf_path = temp_dir.path().join("boilerplate.tf");
        TofuWriter::new(boilerplate_tf_path)
            .overwrite(boilerplate_tf)
            .await?
            .format()
            .await?;

        let mut cmd = CommandBuilder::new(CommandKind::Tofu);
        let tf_plugin_cache_dir = std::env::var("TF_PLUGIN_CACHE_DIR")?;
        cmd.args(["providers", "mirror", &tf_plugin_cache_dir]);
        cmd.use_output_behaviour(OutputBehaviour::Display);
        cmd.use_run_dir(temp_dir.path());
        cmd.run_raw().await?;

        let persist = temp_dir.into_path();
        println!("Persisting dir for testing at {}", persist.display());

        Ok(())
    }

    #[tokio::test]
    pub async fn terraform_concurrent_init() -> eyre::Result<()> {
        let temp_dir = Builder::new().tempdir_in(AppDir::Temp.as_path_buf())?;
        let num_workspaces = 25;
        let mut join_set: JoinSet<eyre::Result<()>> = JoinSet::new();
        for i in 0..num_workspaces {
            let workspace_dir = temp_dir.path().join(format!("workspace_{i:03}"));
            join_set.spawn(async move {
                workspace_dir.ensure_dir_exists().await?;
                TofuImporter::new().using_dir(&workspace_dir).run().await?;
                Ok(())
            });
        }
        while let Some(x) = join_set.join_next().await {
            x??;
        }

        Ok(())
    }
}
