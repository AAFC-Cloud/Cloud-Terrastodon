use crate::menu_action::MenuAction;
use crate::menu_action::MenuActionResult;
use crate::read_line::read_line;
use anyhow::Context;
use anyhow::Result;
use cloud_terrastodon_core_fzf::pick_many;
use cloud_terrastodon_core_fzf::FzfArgs;
use std::io::Write;
use tracing::error;
use tracing::info;

pub async fn menu() -> Result<MenuActionResult> {
    // Create a container for the choices we are about to gather
    let mut choices = Vec::new();

    // Flag for showing a warning
    let mut some_unavailable = false;

    // Collect choices
    for action in MenuAction::variants() {
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
            Ok(MenuActionResult::PauseAndContinue) if chosen.len() == 1 => {
                // Only pause when running a single action
                press_enter_to_continue().await?;
            }
            Ok(MenuActionResult::QuitApplication) => {
                return Ok(MenuActionResult::QuitApplication);
            }
            Ok(MenuActionResult::Continue) | Ok(MenuActionResult::PauseAndContinue) => {}
        }
    }
    Ok(MenuActionResult::Continue)
}

pub async fn press_enter_to_continue() -> Result<()> {
    print!("Press Enter to continue...");
    std::io::stdout().flush()?;
    read_line().await?;
    Ok(())
}

pub async fn menu_loop() -> Result<()> {
    loop {
        if menu().await? == MenuActionResult::QuitApplication {
            info!("Goodbye!");
            return Ok(());
        }
    }
}
