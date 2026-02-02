use clap::Args;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_hcl::prelude::TerraformPlan;
use eyre::Result;
use std::path::PathBuf;
use serde_json;
use tokio::fs;
use tracing::debug;

/// Show a Terraform plan or plan JSON as a parsed `TerraformPlan`.
#[derive(Args, Debug, Clone)]
pub struct TerraformShowArgs {
    /// Path to a Terraform plan (.tfplan) or a JSON plan file (.json)
    pub plan_file: PathBuf,
}

impl TerraformShowArgs {
    pub async fn invoke(self) -> Result<()> {
        // Determine whether the given file is JSON
        let is_json = self.plan_file.extension().and_then(|s| s.to_str()) == Some("json");

        let plan: TerraformPlan = if is_json {
            let content = fs::read_to_string(&self.plan_file).await?;
            serde_json::from_str(&content)?
        } else {
            let path_str = self
                .plan_file
                .to_str()
                .ok_or_else(|| eyre::eyre!("Plan file path is not valid UTF-8"))?;
            let mut cmd = CommandBuilder::new(CommandKind::Terraform);
            cmd.should_announce(true);
            cmd.args(["show", "--json", path_str]);
            cmd.run::<TerraformPlan>().await?
        };

        // TODO: operate on `plan` (stub stops here)
        debug!("Parsed plan: {:#?}", plan);

        Ok(())
    }
}
