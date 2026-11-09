use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_user_input::InlineTerminal;
use cloud_terrastodon_user_input::PickError;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::TerminalGuard;
use cloud_terrastodon_user_input::apply_terminal_control;
use eyre::Result;

/// Keep servicing both halves of the nested terminal handoff. Awaiting the
/// picker alone would deadlock its acquire and release against our inline owner.
pub(super) async fn pick_planning_targets(
    choices: Vec<String>,
    selected: &[String],
    screen: &mut InlineTerminal,
    guard: &mut TerminalGuard,
    cancellation: &CancellationScope,
) -> Result<Option<Vec<String>>> {
    let (picker_cancellation, picker) = cancellation.with_async_child_scope(async |child| {
        PickerTui::new()
            .set_header(
                "Plan targets • Enter: include focused + apply • Tab: toggle • Ctrl-Enter: apply checked • Ctrl-D: clear • Esc: cancel",
            )
            .set_auto_accept(false)
            .set_allow_empty_selection(true)
            .set_include_focused_on_enter(true)
            .set_initial_selected(selected)
            .pick_many(choices, &child)
            .await
    });
    let mut picker = Box::pin(picker);
    let mut first_error = None;
    let mut controls_open = true;
    loop {
        tokio::select! {
            result = &mut picker => {
                if let Some(error) = first_error {
                    if let Err(picker_error) = result
                        && !matches!(picker_error, PickError::Cancelled)
                    {
                        tracing::warn!(?picker_error, "Picker also failed while draining after a terminal error");
                    }
                    return Err(error);
                }
                return match result {
                    Ok(selected) => Ok(Some(selected)),
                    Err(PickError::Cancelled) => Ok(None),
                    Err(error) => Err(error.into()),
                };
            },
            control = guard.next_control(), if controls_open => {
                let result = match control {
                    Some(control) => apply_terminal_control(control, guard, screen),
                    None => {
                        controls_open = false;
                        guard.poison("terminal coordinator owner channel closed");
                        Err(eyre::eyre!("terminal coordinator owner channel closed"))
                    }
                };
                if let Err(error) = result {
                    picker_cancellation.request_cancel("planning target picker terminal failed");
                    if first_error.is_none() {
                        first_error = Some(error);
                    } else {
                        tracing::warn!(?error, "Additional terminal failure while draining the picker");
                    }
                }
            }
        }
    }
}
