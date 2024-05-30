use std::path::PathBuf;

use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use pathing_types::IgnoreDir;
use tracing::warn;
pub async fn apply_processed() -> Result<()> {
    let processed_dir: PathBuf = IgnoreDir::Processed.into();
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
