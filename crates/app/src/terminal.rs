use cloud_terrastodon_tracing::StructuredLogLevel;
use cloud_terrastodon_tracing::TerminalActivityProbe;
use cloud_terrastodon_tracing::TerminalLogBuffer;
use cloud_terrastodon_user_input::PickerLogBuffer;
use cloud_terrastodon_user_input::PickerLogBufferHandle;
use cloud_terrastodon_user_input::PickerLogLevel;
use cloud_terrastodon_user_input::PickerLogRecord;
use cloud_terrastodon_user_input::PickerLogSpan;
use cloud_terrastodon_user_input::TerminalActivity;
use cloud_terrastodon_user_input::TerminalCoordinator;
use cloud_terrastodon_user_input::TerminalCoordinatorFutureExt;
use cloud_terrastodon_user_input::TerminalLogBufferFutureExt;
use eyre::Result;
use std::future::Future;
use std::sync::Arc;

/// Connects application tracing and invocation task locals to one terminal owner.
///
/// Construction is runtime-independent, so tracing can use the activity probe
/// and log buffer before the application's runtime is created.
pub(crate) struct TerminalSession {
    activity: TerminalActivity,
    log_buffer: TerminalLogBuffer,
}

impl TerminalSession {
    pub(crate) fn new() -> Self {
        Self {
            activity: TerminalActivity::new(),
            log_buffer: TerminalLogBuffer::new(),
        }
    }

    pub(crate) fn log_buffer(&self) -> TerminalLogBuffer {
        self.log_buffer.clone()
    }

    pub(crate) fn activity_probe(&self) -> TerminalActivityProbe {
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
            self.log_buffer.replay_to_stderr();
        }
        Ok(output)
    }
}

struct TracingPickerLogBuffer {
    buffer: TerminalLogBuffer,
}

impl PickerLogBuffer for TracingPickerLogBuffer {
    fn records_since(&self, cursor: &mut usize) -> Vec<PickerLogRecord> {
        self.buffer
            .records_since(cursor)
            .into_iter()
            .map(|record| PickerLogRecord {
                level: match record.level {
                    StructuredLogLevel::Debug | StructuredLogLevel::Trace => PickerLogLevel::Debug,
                    StructuredLogLevel::Info => PickerLogLevel::Info,
                    StructuredLogLevel::Warn => PickerLogLevel::Warn,
                    StructuredLogLevel::Error => PickerLogLevel::Error,
                },
                message: record.message,
                target: record.target,
                timestamp: record.timestamp,
                fields: record
                    .fields
                    .into_iter()
                    .map(|(name, value)| (name.into(), value.into()))
                    .collect(),
                spans: record
                    .spans
                    .into_iter()
                    .map(|span| PickerLogSpan {
                        name: span.name,
                        fields: span
                            .fields
                            .into_iter()
                            .map(|(name, value)| (name.into(), value.into()))
                            .collect(),
                    })
                    .collect(),
                file: record.file,
                line: record.line,
            })
            .collect()
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
        let terminal = TerminalSession::new();
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
                Result::<_>::Ok(counter.get())
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
