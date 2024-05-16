use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use fzf::pick;
use fzf::FzfArgs;
use std::path::PathBuf;
use tofu::prelude::list_blocks;
use tokio::fs;

pub async fn jump_to_block() -> Result<()> {
    let dir = PathBuf::from_iter(["ignore", "processed"]);
    let mut files = fs::read_dir(dir).await.context("reading files")?;
    let mut choices = Vec::new();
    while let Some(tf_file) = files.next_entry().await.context("reading entry")? {
        let kind = tf_file.file_type().await.context("getting file type")?;
        if !kind.is_file() {
            continue;
        }
        let mut blocks = list_blocks(tf_file.path())
            .await
            .context("listing blocks")?;
        choices.append(&mut blocks);
    }

    let chosen = pick(FzfArgs {
        choices,
        many: false,
        prompt: None,
        header: Some("Blocks".to_string()),
    })
    .context("picking")?;
    let chosen = chosen.first().ok_or(anyhow!("pick one failed"))?;
    CommandBuilder::new(CommandKind::VSCode)
        .args([
            "--goto",
            format!("{}:{}", chosen.path.display(), chosen.line_number).as_str(),
        ])
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await
        .context("running vscode command")?;
    Ok(())
}
