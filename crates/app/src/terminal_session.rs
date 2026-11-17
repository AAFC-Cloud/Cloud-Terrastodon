use crate::tracing_picker_log_buffer::TracingPickerLogBuffer;
use cloud_terrastodon_user_input::PickerLogBufferHandle;
use cloud_terrastodon_user_input::TerminalActivity;
use cloud_terrastodon_user_input::TerminalCoordinator;
use cloud_terrastodon_user_input::TerminalCoordinatorFutureExt;
use cloud_terrastodon_user_input::TerminalLogBufferFutureExt;
use eyre::Result;
use std::future::Future;
use std::sync::Arc;
use teamy_tracing::capture::LogBuffer;

/// Connects application tracing and invocation task locals to one terminal owner.
///
/// Construction is runtime-independent, so tracing can use the activity probe
/// and log buffer before the application's runtime is created.
pub(crate) struct TerminalSession {
    activity: TerminalActivity,
    log_buffer: LogBuffer,
    capture: Option<LogBuffer>,
}

impl TerminalSession {
    pub(crate) fn new(log_buffer: LogBuffer, capture: Option<LogBuffer>) -> Self {
        Self {
            activity: TerminalActivity::new(),
            log_buffer,
            capture,
        }
    }

    pub(crate) fn log_buffer(&self) -> LogBuffer {
        self.log_buffer.clone()
    }

    pub(crate) fn activity_probe(&self) -> Arc<dyn Fn() -> bool + Send + Sync> {
        let activity = self.activity.clone();
        Arc::new(move || activity.is_active())
    }

    /// Runs an invocation on the caller-owned runtime with terminal task locals.
    ///
    /// The outer result reports terminal setup failure; the invocation's output
    /// is returned unchanged. No `Send` or `'static` bound is needed because the
    /// invocation is driven directly by `block_on`, not spawned.
    pub(crate) fn block_on<F: Future>(
        &self,
        runtime: &tokio::runtime::Runtime,
        invocation: F,
        replay_logs: bool,
    ) -> Result<F::Output> {
        // Coordinator construction spawns its actor. Enter the runtime now;
        // the later block_on would otherwise be too late for synchronous setup.
        let coordinator = {
            let _runtime_guard = runtime.enter();
            TerminalCoordinator::try_new_with_activity(self.activity.clone())?
        };
        #[cfg(feature = "terminal_coordinator_debug")]
        let _debug_application_root = coordinator.debug_register_as_application_root()?;
        let picker_log_buffer: PickerLogBufferHandle = Arc::new(TracingPickerLogBuffer {
            buffer: self.log_buffer.clone(),
            capture: self.capture.clone(),
        });

        // Keep potentially large CLI dispatch futures off the main-thread stack.
        let invocation = Box::pin(invocation.with_terminal_coordinator(coordinator.clone()));
        let output = runtime.block_on(invocation.with_terminal_log_buffer(picker_log_buffer));
        if let Some(payload) = coordinator.take_actor_panic() {
            // Infrastructure invariant violations must remain process-level
            // failures, not be hidden inside a value-level invocation result.
            std::panic::resume_unwind(payload);
        }
        if replay_logs {
            // Exit replay remains best effort: an unavailable stderr must not
            // replace the invocation's original result with a secondary error.
            // Reporting that failure to the same destination would fail again.
            let _ = self.log_buffer.replay_to(std::io::stderr());
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_user_input::try_current_picker_log_buffer;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn scopes_terminal_state_for_a_borrowed_non_send_invocation() -> Result<()> {
        let terminal = TerminalSession::new(LogBuffer::new(), None);
        let active = terminal.activity_probe();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let counter = Rc::new(Cell::new(0));
        assert!(!active());
        assert!(TerminalCoordinator::try_current().is_none());
        assert!(try_current_picker_log_buffer().is_none());

        let output = terminal.block_on(
            &runtime,
            async {
                let counter = Rc::clone(&counter);
                let coordinator = TerminalCoordinator::try_current()
                    .expect("invocation should inherit the terminal coordinator");
                let logs = try_current_picker_log_buffer()
                    .expect("invocation should inherit the picker log buffer");
                assert!(logs.records_since(&mut 0).is_empty());
                let guard = coordinator.acquire().await?;
                assert!(active());
                tokio::task::yield_now().await;
                counter.set(counter.get() + 1);
                guard.release().await?;
                assert!(!active());
                eyre::Ok(counter.get())
            },
            false,
        )??;

        assert_eq!(output, 1);
        assert_eq!(counter.get(), 1);
        assert!(!active());
        assert!(TerminalCoordinator::try_current().is_none());
        assert!(try_current_picker_log_buffer().is_none());
        Ok(())
    }
}
