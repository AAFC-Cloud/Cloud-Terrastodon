use std::path::PathBuf;

use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use tracing::warn;
pub async fn apply_processed() -> Result<()> {
    let processed_dir: PathBuf = AppDir::Processed.into();
    let result = CommandBuilder::new(CommandKind::Terraform)
        .arg("apply")
        .use_run_dir(processed_dir)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await;
    if let Err(e) = result {
        warn!("Terraform apply failed, did you say no? {e:?}")
    }
    Ok(())
}
