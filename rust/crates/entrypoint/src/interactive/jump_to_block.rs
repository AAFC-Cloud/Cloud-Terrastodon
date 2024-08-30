use anyhow::Context;
use anyhow::Result;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_command::prelude::OutputBehaviour;
use cloud_terrastodon_core_fzf::pick;
use cloud_terrastodon_core_fzf::FzfArgs;
use cloud_terrastodon_core_tofu::prelude::list_blocks_for_dir;
use std::path::PathBuf;
use tracing::info;

pub async fn jump_to_block(dir: PathBuf) -> Result<()> {
    // We don't use `as_single_body` because we need to track the files that each block comes from
    let choices = list_blocks_for_dir(dir).await?;
    info!("Found {} blocks", choices.len());

    let chosen = pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Blocks".to_string()),
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
