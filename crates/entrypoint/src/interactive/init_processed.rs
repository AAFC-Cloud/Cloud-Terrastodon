use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::path::PathBuf;
pub async fn init_processed() -> Result<()> {
    let processed_dir: PathBuf = AppDir::Processed.into();
    CommandBuilder::new(CommandKind::Terraform)
        .arg("init")
        .use_run_dir(processed_dir)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await?;
    Ok(())
}
