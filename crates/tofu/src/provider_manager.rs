use crate::writer::TofuWriter;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_name;
use cloud_terrastodon_command::prelude::CommandBuilder;
use cloud_terrastodon_command::prelude::CommandKind;
use cloud_terrastodon_command::prelude::OutputBehaviour;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_tofu_types::prelude::AsTofuString;
use cloud_terrastodon_tofu_types::prelude::ProviderAvailability;
use cloud_terrastodon_tofu_types::prelude::TFProviderHostname;
use cloud_terrastodon_tofu_types::prelude::TFProviderNamespace;
use cloud_terrastodon_tofu_types::prelude::TofuProviderBlock;
use cloud_terrastodon_tofu_types::prelude::TofuProviderKind;
use cloud_terrastodon_tofu_types::prelude::TofuTerraformBlock;
use cloud_terrastodon_tofu_types::prelude::TofuTerraformRequiredProvidersBlock;
use directories_next::BaseDirs;
use eyre::Context;
use eyre::OptionExt;
use eyre::bail;
use hcl::edit::structure::Block;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env::{self};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;
use tokio::io::AsyncReadExt;
use tracing::debug;

/// Helper to address concurrency issues.
///
/// https://github.com/hashicorp/terraform/issues/33321
/// > "terraform providers mirror" skip downloading packages that are already present in the mirror directory
///
/// https://github.com/hashicorp/terraform/issues/31964
/// > Allow multiple Terraform instances to write to plugin_cache_dir concurrently
pub struct ProviderManager {
    pub local_mirror_dir: PathBuf,
}

impl ProviderManager {
    /// https://developer.hashicorp.com/terraform/cli/config/config-file#provider-plugin-cache
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

    /// https://developer.hashicorp.com/terraform/cli/config/config-file#implied-local-mirror-directories
    pub fn get_local_mirror_dir() -> eyre::Result<PathBuf> {
        #[cfg(windows)]
        {
            // Windows: %APPDATA%/terraform.d/plugins
            return Ok(BaseDirs::new()
                .ok_or_eyre("Failed to get base dirs")?
                .config_dir()
                .join("terraform.d/plugins")
                .to_path_buf());
        }
        #[cfg(not(windows))]
        {
            // Not(Windows): $HOME/.terraform.d/plugins
            return Ok(BaseDirs::new()
                .ok_or_eyre("Failed to get base dirs")?
                .home_dir()
                .join(".terraform.d/plugins")
                .to_path_buf());
        }
    }

    pub fn try_new() -> eyre::Result<Self> {
        Ok(ProviderManager {
            local_mirror_dir: Self::get_local_mirror_dir()?,
        })
    }

    pub async fn list_cached_providers(&self) -> eyre::Result<HashSet<ProviderAvailability>> {
        let mut rtn = HashSet::default();
        if !matches!(
            tokio::fs::try_exists(&self.local_mirror_dir).await,
            Ok(true)
        ) {
            return Ok(HashSet::new());
        }
        let mut cache_children = tokio::fs::read_dir(&self.local_mirror_dir).await?;
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

    pub async fn populate_provider_cache(
        &self,
        desired_providers: &TofuTerraformRequiredProvidersBlock,
    ) -> eyre::Result<Option<TempDir>> {
        let provider_manager = self;
        let available_providers = provider_manager.list_cached_providers().await?;
        let missing_providers = desired_providers.identify_missing(&available_providers);
        if missing_providers.0.is_empty() {
            debug!("All required providers are already available");
            return Ok(None);
        }

        let terraform_block = TofuTerraformBlock {
            backend: None,
            required_providers: Some(missing_providers),
            other: vec![],
        };
        let terraform_block: Block = terraform_block.into();
        let boilerplate_tf = terraform_block.as_tofu_string();
        debug!("Mirroring providers using this terraform:\n{boilerplate_tf}");
        let app_temp_dir = AppDir::Temp.as_path_buf();
        app_temp_dir.ensure_dir_exists().await?;
        let temp_dir = tempfile::Builder::new().tempdir_in(&app_temp_dir)?;
        let boilerplate_tf_path = temp_dir.path().join("boilerplate.tf");
        TofuWriter::new(boilerplate_tf_path)
            .overwrite(boilerplate_tf)
            .await?
            .format_file()
            .await?;

        let mut cmd = CommandBuilder::new(CommandKind::Tofu);
        // let tf_plugin_cache_dir = std::env::var("TF_PLUGIN_CACHE_DIR")?;
        // PathBuf::from(&tf_plugin_cache_dir)
        //     .ensure_dir_exists()
        //     .await?;
        cmd.args([
            "providers",
            "mirror",
            &self
                .local_mirror_dir
                .display()
                .to_string()
                .replace("\\", "/"),
        ]);
        cmd.use_output_behaviour(OutputBehaviour::Display);
        cmd.use_run_dir(temp_dir.path());
        cmd.run_raw().await?;
        Ok(Some(temp_dir))
    }

    pub async fn write_default_provider_configs(
        &self,
        work_dir: impl AsRef<Path>,
    ) -> eyre::Result<()> {
        let work_dir = work_dir.as_ref();
        // Get devops url
        let org_service_url = format!(
            "https://dev.azure.com/{name}/",
            name = get_default_organization_name().await?
        );

        // Open boilerplate file
        debug!("Writing default provider configs in {}", work_dir.display());
        let boilerplate_path = work_dir.join("boilerplate.tf");
        TofuWriter::new(boilerplate_path)
            .merge(vec![TofuTerraformBlock {
                required_providers: Some(TofuTerraformRequiredProvidersBlock::common()),
                ..Default::default()
            }])
            .await
            .wrap_err("Writing terraform block")?
            .merge(vec![TofuProviderBlock::AzureDevOps {
                alias: None,
                org_service_url,
            }])
            .await
            .wrap_err("Writing default provider blocks")?
            .format_file()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::ProviderManager;
    use cloud_terrastodon_tofu_types::prelude::TofuTerraformRequiredProvidersBlock;
    use eyre::bail;
    use hcl::edit::structure::Body;

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
        let temp_dir = provider_manager
            .populate_provider_cache(&required_providers)
            .await?;
        let temp_dir = match temp_dir {
            None => {
                bail!("All required providers are already installed");
            }
            Some(x) => x,
        };

        let persist = temp_dir.into_path();
        println!("Persisting dir for testing at {}", persist.display());
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    pub async fn install_default_providers() -> eyre::Result<()> {
        let required_providers = TofuTerraformRequiredProvidersBlock::common();
        let provider_manager = ProviderManager::try_new()?;
        let temp_dir = provider_manager
            .populate_provider_cache(&required_providers)
            .await?;
        let temp_dir = match temp_dir {
            None => {
                bail!("All required providers are already installed");
            }
            Some(x) => x,
        };

        let persist = temp_dir.into_path();
        println!("Persisting dir for testing at {}", persist.display());
        Ok(())
    }
}
