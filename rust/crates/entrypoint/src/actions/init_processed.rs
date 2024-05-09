use std::path::PathBuf;

use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
pub async fn init_processed() -> Result<()> {
    let processed_dir = PathBuf::from("ignore").join("processed");
    CommandBuilder::new(CommandKind::Tofu)
        .arg("init")
        .use_run_dir(processed_dir)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await?;
    Ok(())
}
