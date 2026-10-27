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
    let mut picker = Box::pin(
        PickerTui::new()
            .set_header(
                "Plan targets • Enter: include focused + apply • Tab: toggle • Ctrl-Enter: apply checked • Ctrl-D: clear • Esc: cancel",
            )
            .set_auto_accept(false)
            .set_allow_empty_selection(true)
            .set_include_focused_on_enter(true)
            .set_initial_selected(selected)
            .cancel_on(cancellation.cancelled())
            .pick_many(choices),
    );
    loop {
        tokio::select! {
            result = &mut picker => return match result {
                Ok(selected) => Ok(Some(selected)),
                Err(PickError::Cancelled) => Ok(None),
                Err(error) => Err(error.into()),
            },
            control = guard.next_control() => {
                let control = control.ok_or_else(|| eyre::eyre!("terminal coordinator owner channel closed"))?;
                apply_terminal_control(control, guard, screen)?;
            }
        }
    }
}
