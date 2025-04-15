pub mod egui_config;
pub mod iconfig;

use cloud_terrastodon_core_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::debug;
use tracing::error;
use tracing::warn;

static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(get_or_create_config()));

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct Config {
    pub commands: CommandsConfig,
    pub scan_dirs: HashSet<PathBuf>,
}
impl Config {
    pub fn get_config_path() -> PathBuf {
        AppDir::Config.as_path_buf().join("config.json")
    }
    pub fn get_active_config() -> std::sync::MutexGuard<'static, Config> {
        CONFIG.lock().expect("Failed to lock the CONFIG mutex")
    }
    pub fn modify_and_save_active_config<F>(modifier: F) -> Result<()>
    where
        F: FnOnce(&mut Config),
    {
        let mut config = Config::get_active_config();

        // Capture the original state to compare later
        let original_config = (*config).clone();

        // Apply the modification closure
        modifier(&mut config);

        // Check if the config was modified
        if *config != original_config {
            // Save the modified config to disk
            write_config_to_disk(&config, &Config::get_config_path())?;
            debug!("Config was modified and saved to disk.");
        } else {
            debug!("Config was not modified, no need to save.");
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Clone)]
pub struct CommandsConfig {
    pub azure_cli: String,
    pub tofu: String,
    pub terraform: String,
    pub vscode: String,
}

impl Default for Config {
    #[cfg(windows)]
    fn default() -> Self {
        Self {
            commands: CommandsConfig {
                azure_cli: "az.cmd".to_string(),
                tofu: "tofu.exe".to_string(),
                terraform: "terraform.exe".to_string(),
                vscode: "code.cmd".to_string(),
            },
            scan_dirs: [AppDir::Processed.as_path_buf()].into(),
        }
    }
    #[cfg(not(windows))]
    fn default() -> Self {
        Self {
            commands: CommandsConfig {
                azure_cli: "az".to_string(),
                tofu: "tofu".to_string(),
                terraform: "terraform".to_string(),
                vscode: "code".to_string(),
            },
            scan_folders: [AppDir::Processed.as_path_buf()].into(),
        }
    }
}

fn get_or_create_config() -> Config {
    let config_path = Config::get_config_path();
    match load_config_from_disk(&config_path) {
        Ok(config) => config,
        Err(e) => {
            warn!(
                "Failed to load config, using default and writing it to disk. Error: {:?}",
                e
            );

            if config_path.exists() {
                if let Err(e) = backup_config(&config_path) {
                    error!("Failed to backup existing config! {:?}", e);
                }
            }

            let config = Config::default();
            if let Err(e) = write_config_to_disk(&config, &config_path) {
                error!("Failed to write default config to disk! {:?}", e);
            }

            config
        }
    }
}

fn load_config_from_disk(config_path: &PathBuf) -> Result<Config> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(config_path)
        .context(format!(
            "opening config file for reading \"{}\"",
            config_path.display()
        ))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).context(format!(
        "reading file contents from {}",
        config_path.display()
    ))?;
    let config = serde_json::from_str::<Config>(&contents).context("parsing contents as config")?;
    debug!("Successfully loaded config from {}", config_path.display());
    Ok(config)
}

fn backup_config(config_path: &PathBuf) -> Result<()> {
    let mut backup_path = config_path.with_extension("bad");
    let mut counter = 1;

    while backup_path.exists() {
        backup_path = config_path.with_extension(format!("{}{}", counter, ".bad"));
        counter += 1;
    }

    std::fs::rename(config_path, &backup_path).context(format!(
        "renaming config file \"{}\" to \"{}\"",
        config_path.display(),
        backup_path.display()
    ))?;

    Ok(())
}

fn write_config_to_disk(config: &Config, config_path: &PathBuf) -> Result<()> {
    let Some(config_dir) = config_path.parent() else {
        bail!(
            "Config path doesn't have a parent dir? path={}",
            config_path.display()
        );
    };
    std::fs::create_dir_all(config_dir).context(format!(
        "ensuring config dir \"{}\" exists",
        config_dir.display()
    ))?;
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(config_path)
        .context(format!(
            "opening config file for writing \"{}\"",
            config_path.display()
        ))?;
    let content = serde_json::to_string_pretty(&config).context("serializing config")?;
    file.write_all(content.as_bytes())
        .context(format!("writing bytes to \"{}\"", config_path.display()))?;
    Ok(())
}
