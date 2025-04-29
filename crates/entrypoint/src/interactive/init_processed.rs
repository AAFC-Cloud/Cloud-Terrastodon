use std::path::PathBuf;

use cloud_terrastodon_command::prelude::CommandBuilder;
use cloud_terrastodon_command::prelude::CommandKind;
use cloud_terrastodon_command::prelude::OutputBehaviour;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
pub async fn init_processed() -> Result<()> {
    let processed_dir: PathBuf = AppDir::Processed.into();
    CommandBuilder::new(CommandKind::Tofu)
        .arg("init")
        .use_run_dir(processed_dir)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await?;
    Ok(())
}
