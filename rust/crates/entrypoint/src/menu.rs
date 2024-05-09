use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use fzf::pick;
use fzf::FzfArgs;

use crate::action::Action;

pub async fn menu() -> Result<()> {
    // Create a container for the choices we are about to gather
    let mut choices = Vec::new();

    // Flag for showing a warning
    let mut some_unavailable = false;

    // Collect choices
    for action in Action::variants() {
        if action.is_available().await {
            choices.push(action);
        } else {
            some_unavailable = true;
        }
    }

    // Prompt user for action of choice
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

    let chosen = chosen.first().ok_or(anyhow!("menu choice failed"))?;
    chosen
        .invoke()
        .await
        .context(format!("invoking action: {chosen}"))?;

    if chosen.should_pause() {
        CommandBuilder::new(CommandKind::Pause)
            .use_output_behaviour(OutputBehaviour::Display)
            .run_raw()
            .await?;
    }

    Ok(())
}

pub async fn menu_loop() -> Result<()> {
    loop {
        menu().await?;
    }
}
