use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CommandsConfig {
    pub azure_cli: String,
    pub tofu: String,
    pub terraform: String,
    pub vscode: String,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            azure_cli: "az.cmd".to_string(),
            tofu: "tofu.exe".to_string(),
            terraform: "terraform.exe".to_string(),
            vscode: "code.cmd".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Config for CommandsConfig {
    const FILE_SLUG: &'static str = "commands";
}
