use super::handler_tasks::HandlerTasks;
use crate::TerminalBackend;
use crate::TerminalGuard;
use crate::apply_terminal_control;

/// Cancel optional handlers while retaining required work and terminal handoffs.
/// The picker session owns protection through this drain and its guard release.
pub(super) async fn drain_picker_handlers(
    handlers: &mut HandlerTasks<'_>,
    guard: &mut TerminalGuard,
    backend: &mut impl TerminalBackend,
) -> eyre::Result<()> {
    handlers.cancel("picker is closing");
    let mut controls_open = !guard.is_poisoned();
    let mut failure = (!controls_open)
        .then(|| eyre::eyre!("terminal coordinator was poisoned before draining picker handlers"));
    // A nested owner can also have been spawned outside an event handler. Its
    // frame must resume this backend before the parent releases its own guard.
    while !handlers.is_empty() || (controls_open && !backend.is_active()) {
        let result = tokio::select! {
            control = guard.next_control(), if controls_open => {
                match control {
                    Some(control) => {
                        let result = apply_terminal_control(control, guard, backend);
                        if result.is_err() {
                            // A poisoned/failed backend cannot resume; still
                            // retain the jobs and observe all their outcomes.
                            controls_open = false;
                        }
                        result
                    }
                    None => {
                        controls_open = false;
                        Err(eyre::eyre!("terminal coordinator owner channel closed while draining picker handlers"))
                    }
                }
            }
            completion = handlers.next(), if !handlers.is_empty() => completion.unwrap_or(Ok(())),
        };
        if let Err(error) = result {
            if failure.is_none() {
                failure = Some(error);
            } else {
                tracing::warn!(%error, "additional picker shutdown failure");
            }
        }
    }
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}
