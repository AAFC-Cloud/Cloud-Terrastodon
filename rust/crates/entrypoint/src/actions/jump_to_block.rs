use anyhow::Context;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use fzf::pick;
use fzf::FzfArgs;
use pathing_types::IgnoreDir;
use std::path::PathBuf;
use tofu::prelude::list_blocks;
use tokio::fs;
use tracing::info;

pub async fn jump_to_block() -> Result<()> {
    let dir: PathBuf = IgnoreDir::Processed.into();
    let mut files = fs::read_dir(dir).await.context("reading files")?;
    let mut choices = Vec::new();
    let mut num_files = 0;
    while let Some(tf_file) = files.next_entry().await.context("reading entry")? {
        let kind = tf_file.file_type().await.context("getting file type")?;
        if !kind.is_file() {
            continue;
        }
        let path = tf_file.path();
        if path.extension().filter(|ext| ext.to_string_lossy() == "tf").is_none() {
            continue;
        }

        num_files += 1;
        info!("Gathering blocks from {}", path.display());

        let mut blocks = list_blocks(path).await.context("listing blocks")?;
        choices.append(&mut blocks);
    }

    info!("Found {} blocks across {} files", choices.len(), num_files);

    let chosen = pick(FzfArgs {
        choices,
        prompt: None,
        header: Some("Blocks".to_string()),
    })
    .context("picking")?;
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
