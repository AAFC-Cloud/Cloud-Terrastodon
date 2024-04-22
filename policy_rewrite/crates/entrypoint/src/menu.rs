use anyhow::anyhow;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use fzf::pick;
use fzf::FzfArgs;

use crate::action::Action;

pub async fn menu() -> Result<()> {
    let mut choices = Vec::new();
    let mut some_unavailable = false;
    for action in Action::variants() {
        if action.is_available().await {
            choices.push(action);
        } else {
            some_unavailable = true;
        }
    }
    let chosen = pick(FzfArgs {
        choices,
        header: Some(
            if !some_unavailable {
                "Actions"
            } else {
                "Actions (some unavailable items omitted)"
            }
            .to_string(),
        ),
        prompt: None,
        many: false,
    })?;
    chosen
        .first()
        .ok_or(anyhow!("menu choice failed"))?
        .invoke()
        .await?;

    CommandBuilder::new(CommandKind::Pause)
        .use_output_behaviour(OutputBehaviour::Display)
        .run_raw()
        .await?;
    Ok(())
}

pub async fn menu_loop() -> Result<()> {
    loop {
        menu().await?;
    }
}
