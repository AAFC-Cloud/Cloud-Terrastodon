use crate::config::Config;
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary, facet::Facet, Clone, PartialEq)]
pub struct CommandsConfig {
    pub azure_cli: String,
    pub tofu: String,
    pub terraform: String,
    pub vscode: String,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            #[cfg(windows)]
            azure_cli: "az.cmd".to_string(),
            #[cfg(not(windows))]
            azure_cli: "az".to_string(),
            #[cfg(windows)]
            tofu: "tofu.exe".to_string(),
            #[cfg(not(windows))]
            tofu: "tofu".to_string(),
            #[cfg(windows)]
            terraform: "terraform.exe".to_string(),
            #[cfg(not(windows))]
            terraform: "terraform".to_string(),
            #[cfg(windows)]
            vscode: "code.cmd".to_string(),
            #[cfg(not(windows))]
            vscode: "code".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Config for CommandsConfig {
    const FILE_SLUG: &'static str = "commands";
}

cloud_terrastodon_registry::register_thing!(CommandsConfig);
cloud_terrastodon_registry::register_arbitrary!(CommandsConfig);
