use crate::action::Action;
use crate::action::ActionResult;
use crate::read_line::read_line;
use anyhow::Context;
use anyhow::Result;
use fzf::pick_many;
use fzf::FzfArgs;
use std::io::Write;
use tracing::error;
use tracing::info;

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
            .context(format!("invoking action \"{action}\""));
        match result {
            Err(e) => {
                error!("Error calling action handler: {:?}", e);
                press_enter_to_continue().await?;
            }
            Ok(ActionResult::PauseAndContinue) if chosen.len() == 1 => {
                // Only pause when running a single action
                press_enter_to_continue().await?;
            }
            Ok(ActionResult::QuitApplication) => {
                return Ok(ActionResult::QuitApplication);
            }
            Ok(ActionResult::Continue) | Ok(ActionResult::PauseAndContinue) => {}
        }
    }
    Ok(ActionResult::Continue)
}

pub async fn press_enter_to_continue() -> Result<()> {
    print!("Press Enter to continue...");
    std::io::stdout().flush()?;
    read_line().await?;
    Ok(())
}

pub async fn menu_loop() -> Result<()> {
    loop {
        if menu().await? == ActionResult::QuitApplication {
            info!("Goodbye!");
            return Ok(());
        }
    }
}
