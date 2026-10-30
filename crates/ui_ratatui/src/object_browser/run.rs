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
use cloud_terrastodon_user_input::TerminalControl;
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
    let coordinator = match TerminalCoordinator::try_current() {
        Some(coordinator) => coordinator,
        None => TerminalCoordinator::new()?,
    };
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

        let (result, engine_stopped) = tokio::select! {
            result = &mut app_future => (result, false),
            engine_result = &mut engine_future => {
                let error = match engine_result {
                    Ok(_) => eyre::eyre!("explorer engine stopped while the UI was active"),
                    Err(error) => eyre::eyre!("explorer engine failed: {error}"),
                };
                // Let the UI close its render jobs and nested pickers even when
                // the engine has failed. Dropping this future would skip that
                // owned shutdown and could strand terminal ownership.
                cancellation.request_cancel("explorer engine stopped");
                let cleanup = app_future.as_mut().await;
                if let Err(cleanup_error) = cleanup {
                    tracing::warn!(%cleanup_error, "Object browser cleanup after engine failure failed");
                }
                (Err(error), true)
            },
        };
        drop(app_future);
        cancellation.request_cancel("object browser closed");
        // Restore normal console input/output before waiting so cancellation
        // acknowledgments remain visible and Ctrl-C can still force an exit.
        let restore_result = terminal.close().inspect_err(|error| {
            coordinator.poison(format!("Ratatui terminal restoration failed: {error}"));
        });
        if engine_stopped {
            return result.and(restore_result);
        }
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
                engine_result = &mut engine_future => break Err(match engine_result {
                    Ok(_) => eyre::eyre!("explorer engine stopped before pending operations finished"),
                    Err(error) => eyre::eyre!("explorer engine failed during invocation cleanup: {error}"),
                }),
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

    finish_ui_cleanup(
        async { app.close().await.map_err(eyre::Report::from) },
        guard,
        |control, guard| terminal.apply_control(control, guard),
    )
    .await?;
    result
}

/// Finish owned cleanup while nested pickers return terminal ownership. A
/// failed handoff must wake their waiters without dropping the cleanup future.
async fn finish_ui_cleanup(
    closing: impl Future<Output = Result<()>>,
    guard: &mut TerminalGuard,
    mut apply_control: impl FnMut(TerminalControl, &mut TerminalGuard) -> Result<()>,
) -> Result<()> {
    tokio::pin!(closing);
    let mut cleanup_controls = Ok(());
    loop {
        tokio::select! {
            closed = &mut closing => {
                return cleanup_controls.and(closed);
            }
            control = guard.next_control(), if cleanup_controls.is_ok() => {
                cleanup_controls = match control {
                    Some(control) => apply_control(control, guard),
                    None => Err(eyre::eyre!("terminal coordinator owner channel closed during shutdown")),
                }.inspect_err(|error| {
                    // Deferrals do not protect against explicitly dropping
                    // `closing`. Keep it alive, and let waiting pickers fail
                    // their handoff so their cleanup can also finish.
                    guard.poison(format!("terminal handoff during UI cleanup failed: {error}"));
                });
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
    use cloud_terrastodon_command::CancellationToken;
    use std::cell::Cell;
    use std::future::poll_fn;
    use std::io;
    use std::task::Poll;
    use tokio::sync::oneshot;

    #[test]
    fn production_invocation_host_is_context_attaching_tokio_host() {
        let host = TokioInvocationHost::new(attach_invocation_contexts);
        assert_eq!(host.pending_count(), 0);
    }

    #[tokio::test]
    async fn failed_terminal_handoff_waits_for_required_ui_cleanup() {
        let cancellation = CancellationToken::new();
        let requester = cancellation.clone();
        tokio::time::timeout(
            Duration::from_secs(2),
            cancellation.with_async_scope(|scope| async move {
                let coordinator = TerminalCoordinator::new().unwrap();
                let mut guard = coordinator.acquire().await?;
                let (finish, finished) = oneshot::channel();
                let cleanup_entered = Cell::new(false);
                let cleanup_completed = Cell::new(false);
                let handoffs = Cell::new(0);
                let closing = scope.finish_required("finish owned UI cleanup", |_| async {
                    cleanup_entered.set(true);
                    finished.await?;
                    cleanup_completed.set(true);
                    Ok(())
                });
                let mut cleanup = Box::pin(finish_ui_cleanup(closing, &mut guard, |control, _| {
                    assert!(matches!(control, TerminalControl::Suspend { .. }));
                    handoffs.set(handoffs.get() + 1);
                    Err(io::Error::new(io::ErrorKind::BrokenPipe, "terminal suspension failed").into())
                }));
                poll_fn(|context| {
                    assert!(cleanup.as_mut().poll(context).is_pending());
                    Poll::Ready(())
                })
                .await;
                assert!(cleanup_entered.get());
                requester.request_cancel("UI closed");

                // A real nested acquisition requests Suspend. Poisoning on
                // failure must release its waiter while cleanup stays owned.
                tokio::select! {
                    result = &mut cleanup => panic!("cleanup returned before it finished: {result:?}"),
                    acquired = coordinator.acquire() => {
                        let error = acquired.err().expect("the failed handoff rejects acquisition");
                        assert!(error.to_string().contains("terminal suspension failed"));
                    }
                }
                assert_eq!(handoffs.get(), 1);
                assert!(!cleanup_completed.get());
                poll_fn(|context| {
                    assert!(cleanup.as_mut().poll(context).is_pending());
                    Poll::Ready(())
                })
                .await;
                assert_eq!(handoffs.get(), 1, "the failed control channel is no longer polled");

                finish.send(()).unwrap();
                let error = cleanup.await.unwrap_err();
                assert!(cleanup_completed.get());
                assert_eq!(error.downcast_ref::<io::Error>().unwrap().kind(), io::ErrorKind::BrokenPipe);
                assert_eq!(error.to_string(), "terminal suspension failed");
                guard.release().await?;
                Ok::<_, eyre::Report>(())
            }),
        )
        .await
        .expect("terminal failure must not leave cleanup or picker waiters stuck")
        .unwrap();
    }
}
