use super::app::BrowserMode;
use super::app::ObjectBrowserApp;
use super::excalidraw_render_session::ExcalidrawRenderSession;
use super::object_browser_terminal::ObjectBrowserTerminal;
use crate::object_explorer::ArenaQueryContext;
use crate::object_explorer::ArenaQueryContextFutureExt;
use crate::object_explorer::ExplorerEngine;
#[cfg(test)]
use crate::object_explorer::TokioInvocationHost;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_registry::InvocationFuture;
use cloud_terrastodon_user_input::TerminalBackend;
use cloud_terrastodon_user_input::TerminalCoordinator;
use cloud_terrastodon_user_input::TerminalCoordinatorFutureExt;
use cloud_terrastodon_user_input::TerminalGuard;
use cloud_terrastodon_user_input::TerminalLogBufferFutureExt;
use cloud_terrastodon_user_input::try_current_picker_log_buffer;
use crossterm::event;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

const FRAMES_PER_SECOND: f32 = 60.0;
const COMMAND_CAPACITY: usize = 256;

pub(crate) async fn run_object_browser(
    excalidraw: Option<PathBuf>,
    cancellation: &CancellationScope,
) -> Result<()> {
    let coordinator = TerminalCoordinator::try_current().unwrap_or_default();
    let scoped_coordinator = coordinator.clone();
    coordinator
        .scope(async move { run_with_terminal(scoped_coordinator, excalidraw, cancellation).await })
        .await
}

async fn run_with_terminal(
    coordinator: TerminalCoordinator,
    excalidraw: Option<PathBuf>,
    cancellation: &CancellationScope,
) -> Result<()> {
    tracing::info!("Starting arena-backed object browser");
    let logs = try_current_picker_log_buffer()
        .ok_or_else(|| eyre::eyre!("the object browser requires application log capture"))?;
    let mut guard = coordinator.acquire().await?;
    let mut terminal = match ObjectBrowserTerminal::new(logs) {
        Ok(terminal) => terminal,
        Err(error) => {
            coordinator.poison(format!("Ratatui terminal setup failed: {error}"));
            let release_result = guard.release().await;
            return match release_result {
                Ok(()) => Err(error),
                Err(release_error) => Err(eyre::eyre!(
                    "Ratatui terminal setup failed: {error}; guard release also failed: {release_error}"
                )),
            };
        }
    };

    let app_result = run_engine_and_ui(
        &mut terminal,
        &mut guard,
        &coordinator,
        excalidraw,
        cancellation,
    )
    .await;
    let restore_result = terminal.close().inspect_err(|error| {
        coordinator.poison(format!("Ratatui terminal restoration failed: {error}"));
    });
    let logs_result = if restore_result.is_ok() {
        terminal.flush_after_close()
    } else {
        Ok(())
    };
    let release_result = guard.release().await;
    app_result
        .and(restore_result)
        .and(release_result)
        .and(logs_result)
}

async fn run_engine_and_ui(
    terminal: &mut ObjectBrowserTerminal,
    guard: &mut TerminalGuard,
    coordinator: &TerminalCoordinator,
    excalidraw: Option<PathBuf>,
    cancellation: &CancellationScope,
) -> Result<()> {
    let (_shutdown, run) = cancellation.with_async_child_scope(move |cancellation| async move {
        let app_cancellation = cancellation.clone();
        let (context, inbox) = ArenaQueryContext::channel(COMMAND_CAPACITY);
        let shutdown_handle = context.engine_handle();
        let app_context = context.clone();
        let app_scope_context = app_context.clone();
        let engine_context = context.clone();
        drop(context);

        let engine =
            ExplorerEngine::empty_with_tokio_invocation_host(attach_invocation_contexts, &cancellation);
        let engine_future = engine.run(inbox).with_arena_query_context(engine_context);
        let app_future = async {
            let mut app = ObjectBrowserApp::bootstrap(app_context.clone()).await?;
            app.cancellation = Some(app_cancellation.clone());
            app.excalidraw = Some(ExcalidrawRenderSession::new(excalidraw, &app_cancellation));
            run_event_loop(app, terminal, guard, coordinator, &app_cancellation).await
        }
        .with_arena_query_context(app_scope_context);
        tokio::pin!(engine_future);
        let mut app_future = Box::pin(app_future);

        let result = tokio::select! {
            result = &mut app_future => result,
            _engine = &mut engine_future => return Err(eyre::eyre!("explorer engine stopped while the UI was active")),
        };
        drop(app_future);
        cancellation.request_cancel("object browser closed");
        // Restore normal console input/output before waiting so cancellation
        // acknowledgments remain visible and Ctrl-C can still force an exit.
        let restore_result = terminal.close().inspect_err(|error| {
            coordinator.poison(format!("Ratatui terminal restoration failed: {error}"));
        });
        let draining = async {
            shutdown_handle
                .finish_pending_invocations()
                .await
                .map_err(eyre::Report::from)
        };
        tokio::pin!(draining);
        let mut log_ticks = tokio::time::interval(Duration::from_millis(100));
        log_ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut cleanup_logs = Ok(());
        let mut cleanup_controls = Ok(());
        // Export writers may need engine-side query/session completion while
        // cancellation is postponed. Continue polling the engine until they end.
        let drained = loop {
            tokio::select! {
                drained = &mut draining => break drained,
                _engine = &mut engine_future => break Err(eyre::eyre!("explorer engine stopped before pending operations finished")),
                control = guard.next_control(), if cleanup_controls.is_ok() => {
                    // Invoked requests can own their own pickers. Their exit
                    // needs our resume acknowledgement even after the UI closes.
                    cleanup_controls = match control {
                        Some(control) => terminal.apply_control(control, guard)
                            .and_then(|()| terminal.close()),
                        None => Err(eyre::eyre!("terminal owner channel closed during invocation cleanup")),
                    }.inspect_err(|error| {
                        coordinator.poison(format!("terminal handoff during invocation cleanup failed: {error}"));
                    });
                }
                _ = log_ticks.tick(), if restore_result.is_ok() && cleanup_logs.is_ok() && cleanup_controls.is_ok() => {
                    // The browser still owns the guard, so cleanup events are
                    // captured even though normal console mode is restored.
                    // An output failure must not abandon required cleanup.
                    cleanup_logs = terminal.flush_after_close();
                }
            }
        };
        result.and(restore_result).and(drained).and(cleanup_logs).and(cleanup_controls)
    });
    run.await
}

async fn run_event_loop(
    mut app: ObjectBrowserApp,
    terminal: &mut ObjectBrowserTerminal,
    guard: &mut TerminalGuard,
    coordinator: &TerminalCoordinator,
    cancellation: &CancellationScope,
) -> Result<()> {
    let period = Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND);
    let mut interval = tokio::time::interval(period);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let result = async {
        while !app.should_quit {
            tokio::select! {
                () = cancellation.cancelled() => {
                    break;
                }
                control = guard.next_control() => {
                    let Some(control) = control else {
                        return Err(eyre::eyre!("terminal coordinator owner channel closed"));
                    };
                    terminal.apply_control(control, guard)?;
                }
                _ = interval.tick() => {
                    if let Err(error) = app.tick().await {
                        app.recover_action_error(error)?;
                    }
                    if terminal.is_active() {
                        // Inline rendering queries the cursor position. Do not
                        // leave an EventStream worker holding Crossterm's input
                        // reader, or competing with a nested picker's input.
                        for _ in 0..32 {
                            if app.mode == BrowserMode::NestedPicker || app.should_quit
                                || !event::poll(Duration::ZERO)?
                            {
                                break;
                            }
                            if let Err(error) = app.handle_event(&event::read()?).await {
                                app.recover_action_error(error)?;
                            }
                        }
                        terminal.draw(|frame| app.draw(frame)).inspect_err(|error| {
                            coordinator.poison(format!("Ratatui frame draw failed: {error}"));
                        })?;
                        #[cfg(feature = "extended_observability")]
                        tracing::info!(message = "finished Ratatui frame", tracy.frame_mark = true);
                    }
                }
            }
        }
        Ok::<_, eyre::Report>(())
    }
    .await;

    // A nested picker must return terminal ownership while closing. Continue
    // servicing its handoff even after cancellation or an event-loop error.
    let closing = app.close();
    tokio::pin!(closing);
    loop {
        tokio::select! {
            closed = &mut closing => {
                closed?;
                return result;
            }
            control = guard.next_control() => {
                let control = control.ok_or_else(|| eyre::eyre!("terminal coordinator owner channel closed during shutdown"))?;
                terminal.apply_control(control, guard)?;
            }
        }
    }
}

fn attach_invocation_contexts(future: InvocationFuture) -> InvocationFuture {
    let future = match TerminalCoordinator::try_current() {
        Some(coordinator) => Box::pin(future.with_terminal_coordinator(coordinator)),
        None => future,
    };
    let future = match try_current_picker_log_buffer() {
        Some(log_buffer) => Box::pin(future.with_terminal_log_buffer(log_buffer)),
        None => future,
    };
    match ArenaQueryContext::try_current() {
        Some(context) => Box::pin(future.with_arena_query_context(context)),
        None => future,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_invocation_host_is_context_attaching_tokio_host() {
        let host = TokioInvocationHost::new(attach_invocation_contexts);
        assert_eq!(host.pending_count(), 0);
    }
}
