use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::OutputBehaviour;
use cloud_terrastodon_hcl::prelude::list_blocks_for_dir;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use eyre::Context;
use eyre::Result;
use std::path::PathBuf;
use tracing::info;

pub async fn jump_to_block(dir: PathBuf) -> Result<()> {
    // We don't use `as_single_body` because we need to track the files that each block comes from
    let choices = list_blocks_for_dir(dir).await?;
    info!("Found {} blocks", choices.len());

    let chosen = pick(FzfArgs {
        choices,

        header: Some("Blocks".to_string()),
        ..Default::default()
    })
    .context("picking")?;
    CommandBuilder::new(CommandKind::VSCode)
        .args([
            "--goto",
            format!("{}:{}", chosen.path.display(), chosen.line_col.0).as_str(),
        ])
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await
        .context("running vscode command")?;
    Ok(())
}
