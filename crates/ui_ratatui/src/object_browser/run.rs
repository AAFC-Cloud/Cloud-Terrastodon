use std::time::Duration;

use cloud_terrastodon_registry::InvocationFuture;
use cloud_terrastodon_user_input::{
    TerminalBackend, TerminalCoordinator, TerminalCoordinatorFutureExt, TerminalGuard,
    TerminalLogBufferFutureExt, apply_terminal_control, try_current_picker_log_buffer,
};
use crossterm::event::EventStream;
use eyre::Result;
use futures::StreamExt;
use ratatui::DefaultTerminal;

use super::app::ObjectBrowserApp;
use super::terminal::{RatatuiTerminalBackend, restore_terminal};
#[cfg(test)]
use crate::object_explorer::TokioInvocationHost;
use crate::object_explorer::{ArenaQueryContext, ArenaQueryContextFutureExt, ExplorerEngine};

const FRAMES_PER_SECOND: f32 = 60.0;
const COMMAND_CAPACITY: usize = 256;

pub(crate) async fn run_object_browser() -> Result<()> {
    let coordinator = TerminalCoordinator::try_current().unwrap_or_else(TerminalCoordinator::new);
    let scoped_coordinator = coordinator.clone();
    coordinator
        .scope(async move { run_with_terminal(scoped_coordinator).await })
        .await
}

async fn run_with_terminal(coordinator: TerminalCoordinator) -> Result<()> {
    tracing::info!("Starting arena-backed object browser");
    let mut guard = coordinator.acquire().await?;
    let mut terminal = None;
    if let Err(error) = (RatatuiTerminalBackend {
        terminal: &mut terminal,
        coordinator: &coordinator,
    })
    .resume()
    {
        let release_result = guard.release().await;
        return match release_result {
            Ok(()) => Err(error),
            Err(release_error) => Err(eyre::eyre!(
                "Ratatui terminal setup failed: {error}; guard release also failed: {release_error}"
            )),
        };
    }

    let app_result = run_engine_and_ui(&mut terminal, &mut guard, &coordinator).await;
    let restore_result = if terminal.take().is_some() {
        restore_terminal(&coordinator)
    } else {
        Ok(())
    };
    let release_result = guard.release().await;
    match (app_result, restore_result, release_result) {
        (Ok(()), Ok(()), Ok(())) => Ok(()),
        (Err(error), _, _) => Err(error),
        (Ok(()), Err(error), _) => Err(error),
        (Ok(()), Ok(()), Err(error)) => Err(error),
    }
}

async fn run_engine_and_ui(
    terminal: &mut Option<DefaultTerminal>,
    guard: &mut TerminalGuard,
    coordinator: &TerminalCoordinator,
) -> Result<()> {
    let (context, inbox) = ArenaQueryContext::channel(COMMAND_CAPACITY);
    let app_context = context.clone();
    let app_scope_context = app_context.clone();
    let engine_context = context.clone();
    drop(context);

    let engine = ExplorerEngine::empty_with_tokio_invocation_host(attach_invocation_contexts);
    let engine_future = engine.run(inbox).with_arena_query_context(engine_context);
    let app_future = async move {
        let app = ObjectBrowserApp::bootstrap(app_context.clone()).await?;
        run_event_loop(app, terminal, guard, coordinator).await
    }
    .with_arena_query_context(app_scope_context);
    tokio::pin!(engine_future);
    tokio::pin!(app_future);

    tokio::select! {
        result = &mut app_future => result,
        _engine = &mut engine_future => Err(eyre::eyre!("explorer engine stopped while the UI was active")),
    }
}

async fn run_event_loop(
    mut app: ObjectBrowserApp,
    terminal: &mut Option<DefaultTerminal>,
    guard: &mut TerminalGuard,
    coordinator: &TerminalCoordinator,
) -> Result<()> {
    let period = Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND);
    let mut interval = tokio::time::interval(period);
    let mut events = EventStream::new();

    while !app.should_quit {
        tokio::select! {
            control = guard.next_control() => {
                let Some(control) = control else {
                    return Err(eyre::eyre!("terminal coordinator owner channel closed"));
                };
                let mut backend = RatatuiTerminalBackend { terminal, coordinator };
                apply_terminal_control(control, guard, &mut backend)?;
            }
            _ = interval.tick() => {
                app.tick().await?;
                if let Some(terminal) = terminal.as_mut() {
                    terminal.draw(|frame| app.draw(frame)).map_err(|error| {
                        coordinator.poison(format!("Ratatui frame draw failed: {error}"));
                        error
                    })?;
                    #[cfg(feature = "extended_observability")]
                    tracing::info!(message = "finished Ratatui frame", tracy.frame_mark = true);
                }
            }
            Some(Ok(event)) = events.next(), if terminal.is_some() => {
                app.handle_event(&event).await?;
            }
        }
    }

    app.close().await?;
    Ok(())
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
