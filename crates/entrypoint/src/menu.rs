use crate::menu_action::MenuAction;
use crate::menu_action::MenuActionResult;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use eyre::Context;
use eyre::Result;
use strum::VariantArray;
use tracing::info;

pub async fn menu() -> Result<MenuActionResult> {
    // Create a container for the choices we are about to gather
    let mut choices = Vec::new();

    // Flag for showing a warning
    let mut some_unavailable = false;

    // Collect choices
    for action in MenuAction::VARIANTS {
        if action.is_available().await {
            choices.push(action);
        } else {
            some_unavailable = true;
        }
    }

    // show most specific actions first
    choices.reverse();

    // Prompt user for action of choice
    let chosen = PickerTui::new(choices)
        .set_header(if !some_unavailable {
            "Actions"
        } else {
            "Actions (some unavailable items omitted)"
        })
        .pick_many();

    let mut chosen = match chosen {
        Ok(c) => c,
        Err(PickError::Cancelled) => {
            info!("Operation cancelled by user");
            return Ok(MenuActionResult::QuitApplication);
        }
        Err(e) => {
            return Err(e.into());
        }
    };

    // restore execution order
    chosen.reverse();

    for action in &chosen {
        info!("Invoking action \"{action}\"");
        let result = action
            .invoke()
            .await
            .context(format!("invoking action \"{action}\""));
        match result {
            Err(e) => {
                return Err(e.wrap_err("Error calling action handler"));
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
    prompt_line("Press Enter to continue...").await?;
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
