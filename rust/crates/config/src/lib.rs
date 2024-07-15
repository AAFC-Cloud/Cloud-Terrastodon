#![feature(try_blocks)]
use std::fs::OpenOptions;
use std::io::Read;

use anyhow::Context;
use anyhow::Result;
use once_cell::sync::Lazy;
use pathing::AppDir;
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;
use tracing::warn;

static CONFIG: Lazy<Config> = Lazy::new(|| {
    let from_disk: Result<Config> = try {
        let config_path = AppDir::Config.as_path_buf().join("config.json");
        let mut file = OpenOptions::new()
            .read(true)
            .open(&config_path)
            .context(format!("opening config file {}", config_path.display()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).context(format!(
            "reading file contents from {}",
            config_path.display()
        ))?;	
        let config = serde_json::from_str(&contents).context("parsing contents as config")?;
        debug!("Successfully loaded config from {}", config_path.display());
        config
    };
    match from_disk {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load config, using default. Error: {:?}", e);
            Config::default()
        }
    }
});

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub commands: CommandsConfig,
}
impl Config {
    pub fn get_active_config() -> &'static Self {
        &CONFIG
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandsConfig {
    pub powershell: String,
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
                powershell: "pwsh".to_string(),
                azure_cli: "az.cmd".to_string(),
                tofu: "tofu.exe".to_string(),
                terraform: "terraform.exe".to_string(),
                vscode: "code.cmd".to_string(),
            },
        }
    }
    #[cfg(not(windows))]
    fn default() -> Self {
        Self {
            commands: CommandsConfig {
                powershell: "pwsh".to_string(),
                azure_cli: "az".to_string(),
                tofu: "tofu".to_string(),
                terraform: "terraform".to_string(),
                vscode: "code".to_string(),
            },
        }
    }
}
