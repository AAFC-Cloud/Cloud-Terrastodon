use std::path::PathBuf;

use anyhow::Result;
use cloud_terrasotodon_core_command::prelude::CommandBuilder;
use cloud_terrasotodon_core_command::prelude::CommandKind;
use cloud_terrasotodon_core_command::prelude::OutputBehaviour;
use cloud_terrasotodon_core_pathing::AppDir;
use tracing::warn;
pub async fn apply_processed() -> Result<()> {
    let processed_dir: PathBuf = AppDir::Processed.into();
    let result = CommandBuilder::new(CommandKind::Tofu)
        .arg("apply")
        .use_run_dir(processed_dir)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await;
    if let Err(e) = result {
        warn!("Tofu apply failed, did you say no? {e:?}")
    }
    Ok(())
}
