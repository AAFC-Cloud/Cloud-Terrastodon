use std::path::PathBuf;

use cloud_terrastodon_command::prelude::CommandBuilder;
use cloud_terrastodon_command::prelude::CommandKind;
use cloud_terrastodon_command::prelude::OutputBehaviour;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use tracing::warn;
pub async fn plan_processed() -> Result<()> {
    let processed_dir: PathBuf = AppDir::Processed.into();
    let result = CommandBuilder::new(CommandKind::Tofu)
        .arg("plan")
        .use_run_dir(processed_dir)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await;
    if let Err(e) = result {
        warn!("Tofu plan failed D: {e:?}")
    }
    Ok(())
}
