use anyhow::Context;
use anyhow::Result;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_command::prelude::OutputBehaviour;
use cloud_terrastodon_core_fzf::pick;
use cloud_terrastodon_core_fzf::FzfArgs;
use cloud_terrastodon_core_tofu::prelude::list_blocks;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;

pub async fn jump_to_block(dir: PathBuf) -> Result<()> {
    // We don't use `as_single_body` because we need to track the files that each block comes from
    let mut files = fs::read_dir(dir).await.context("reading files")?;
    let mut choices = Vec::new();
    let mut num_files = 0;
    while let Some(tf_file) = files.next_entry().await.context("reading entry")? {
        let kind = tf_file.file_type().await.context("getting file type")?;
        if !kind.is_file() {
            continue;
        }
        let path = tf_file.path();
        if path
            .extension()
            .filter(|ext| ext.to_string_lossy() == "tf")
            .is_none()
        {
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
