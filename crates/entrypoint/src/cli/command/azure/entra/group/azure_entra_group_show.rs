use clap::Args;
use cloud_terrastodon_azure::prelude::EntraGroupId;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;
use tracing::info;

/// Show a single Entra (Azure AD) group by id.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupShowArgs {
    /// Group identifier (UUID).
    #[arg(long = "group-id")]
    pub group_id: EntraGroupId,
}

impl AzureEntraGroupShowArgs {
    pub async fn invoke(self) -> Result<()> {
        info!(group_id = %self.group_id, "Fetching Entra group");

        let group: cloud_terrastodon_azure_types::prelude::EntraGroup = CommandBuilder::new(CommandKind::AzureCLI)
            .args([
                "ad",
                "group",
                "show",
                "--group",
                self.group_id.to_string().as_str(),
                "--output",
                "json",
            ])
            .run()
            .await?;

        to_writer_pretty(stdout(), &group)?;
        println!();
        Ok(())
    }
}
