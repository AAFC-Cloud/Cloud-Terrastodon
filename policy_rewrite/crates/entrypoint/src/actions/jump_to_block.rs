use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use fzf::pick;
use fzf::FzfArgs;
use tofu::prelude::list_blocks;
use tokio::fs;
pub async fn jump_to_block() -> Result<()> {
    let tf_file = PathBuf::from_iter(["ignore", "processed", "generated.tf"]);
    let content = fs::read(&tf_file).await?;
    let content = String::from_utf8(content)?;
    let blocks = list_blocks(content.as_str())?;
    let chosen = pick(FzfArgs {
        choices: blocks,
        many: false,
        prompt: None,
        header: Some("Blocks".to_string()),
    })?;
    let chosen = chosen.first().ok_or(anyhow!("pick one failed"))?;
    CommandBuilder::new(CommandKind::VSCode)
        .args([
            "--goto",
            format!("{}:{}", tf_file.display(), chosen.line_number).as_str(),
        ])
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await?;
    Ok(())
}
