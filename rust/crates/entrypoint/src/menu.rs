use anyhow::Context;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use command::prelude::OutputBehaviour;
use fzf::pick_many;
use fzf::FzfArgs;
use tracing::info;

use crate::action::Action;
use crate::action::ActionResult;

pub async fn menu() -> Result<ActionResult> {
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

    // show most specific actions first
    choices.reverse();

    // Prompt user for action of choice
    let mut chosen = pick_many(FzfArgs {
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
    })?;

    // restore execution order
    chosen.reverse();

    for action in &chosen {
        info!("Invoking action {action}");
        let result = action
            .invoke()
            .await
            .context(format!("invoking action: {action}"))?;
        match result {
            ActionResult::PauseAndContinue if chosen.len() == 1 => {
                // Don't pause when running multiple actions
                CommandBuilder::new(CommandKind::Pause)
                    .use_output_behaviour(OutputBehaviour::Display)
                    .run_raw()
                    .await?;
            }
            ActionResult::QuitApplication => {
                return Ok(result);
            }
            ActionResult::Continue | ActionResult::PauseAndContinue => {}
        }
    }
    Ok(ActionResult::Continue)
}

pub async fn menu_loop() -> Result<()> {
    loop {
        if menu().await? == ActionResult::QuitApplication {
            info!("Goodbye!");
            return Ok(());
        }
    }
}
