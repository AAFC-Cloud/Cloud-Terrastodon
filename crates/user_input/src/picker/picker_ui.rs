use super::drain_picker_handlers::drain_picker_handlers;
use super::handler_tasks::HandlerTasks;
use super::key_effects::KeyEffects;
use super::picker_candidates::PickerCandidates;
use super::picker_event::PickerEvent;
use super::picker_session::run_picker_session;
use super::picker_terminal::PickerTerminal;
use super::picker_terminal::restore_terminal;
use super::picker_terminal_backend::PickerTerminalBackend;
use super::picker_toast::PickerToast;
use super::preserved_selection::preserved_selection;
use super::query_debouncer::QueryDebouncer;
use super::query_event::QueryEvent;
use crate::PickError;
use crate::PickerLogBufferHandle;
use crate::PickerLogLevel;
use crate::PickerLogRecord;
use crate::TerminalBackend;
use crate::TerminalControl;
use crate::TerminalCoordinator;
use crate::TerminalGuard;
use crate::apply_terminal_control as apply_terminal_control_with_backend;
use crate::scope_picker_log_buffer;
use crate::try_current_picker_log_buffer;
use compact_str::CompactString;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use futures::FutureExt;
use futures::StreamExt;
use nucleo::Nucleo;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use ratatui_textarea::CursorMove;
use ratatui_textarea::TextArea;
use rustc_hash::FxHashSet;
use std::future::Future;
use std::future::pending;
use std::future::poll_fn;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationErrorExt;
use teamy_cancellation::CancellationScope;

#[cfg(feature = "extended_observability")]
macro_rules! extended_trace_span {
    ($name:literal) => {
        tracing::trace_span!($name)
    };
}

#[cfg(not(feature = "extended_observability"))]
macro_rules! extended_trace_span {
    ($name:literal) => {
        tracing::Span::none()
    };
}

/// Own the terminal lifecycle and key-based UI independently of candidate values.
pub(super) struct PickerUi<'a> {
    pub(super) default_query: String,
    pub(super) header: Option<String>,
    pub(super) auto_accept: bool,
    pub(super) allow_empty_selection: bool,
    pub(super) include_focused_on_enter: bool,
    pub(super) initial_focused: Option<CompactString>,
    pub(super) cancellation: Option<Pin<Box<dyn Future<Output = ()> + Send + 'a>>>,
    pub(super) terminal_coordinator: Option<TerminalCoordinator>,
    pub(super) log_buffer: Option<PickerLogBufferHandle>,
}

impl<'a> PickerUi<'a> {
    pub(super) async fn run(
        self,
        candidates: &mut dyn PickerCandidates<'a>,
        many: bool,
        static_empty_is_error: bool,
        cancellation: &CancellationScope,
    ) -> Result<(), PickError<()>> {
        run_picker_session(cancellation, async |owner| {
            self.run_inner(candidates, many, static_empty_is_error, &owner)
                .await
        })
        .await
    }

    fn build_text_area(query: &str) -> TextArea<'static> {
        let mut text_area = TextArea::new(vec![query.to_string()]);
        text_area.move_cursor(CursorMove::End);
        text_area.set_block(Block::bordered());
        text_area
    }

    fn apply_initial_focus(
        &mut self,
        list_state: &mut ListState,
        search_results_keys: &[Arc<CompactString>],
    ) {
        if let Some(key) = &self.initial_focused
            && let Some(index) = search_results_keys
                .iter()
                .position(|candidate| **candidate == *key)
        {
            list_state.select(Some(index));
            self.initial_focused = None;
        }
    }

    fn cancel_initial_focus(&mut self) {
        self.initial_focused = None;
    }

    #[tracing::instrument(
        name = "picker_session",
        skip_all,
        fields(many = many, static_empty_is_error = static_empty_is_error),
    )]
    async fn run_inner(
        self,
        candidates: &mut dyn PickerCandidates<'a>,
        many: bool,
        static_empty_is_error: bool,
        cancellation: &CancellationScope,
    ) -> Result<(), PickError<()>> {
        let explicit_coordinator = self.terminal_coordinator.clone();
        let current_coordinator = TerminalCoordinator::try_current();
        #[cfg(feature = "terminal_coordinator_debug")]
        if let (Some(explicit), Some(current)) = (&explicit_coordinator, &current_coordinator) {
            explicit.debug_assert_matches_current(current)?;
        }
        let coordinator = match explicit_coordinator.or(current_coordinator) {
            Some(coordinator) => coordinator,
            None => TerminalCoordinator::new()?,
        };
        #[cfg(feature = "terminal_coordinator_debug")]
        coordinator.debug_assert_matches_registered_application_root()?;
        let log_buffer = self
            .log_buffer
            .clone()
            .or_else(try_current_picker_log_buffer);
        // Keep the large picker state machine behind a heap boundary.  This is especially
        // important when a picker is awaited from a registry/CLI future: the picker owns the
        // terminal loop, event stream, matcher, and handler collection, and embedding all of
        // that in the outer async state machine makes the synchronous poll stack unnecessarily
        // deep.
        let picker_run = Box::pin(self.run_inner_with_coordinator(
            candidates,
            coordinator.clone(),
            log_buffer.clone(),
            many,
            static_empty_is_error,
            cancellation,
        ));
        let scoped_run = coordinator.scope(picker_run);
        if let Some(log_buffer) = log_buffer {
            scope_picker_log_buffer(log_buffer, scoped_run).await
        } else {
            scoped_run.await
        }
    }

    async fn run_inner_with_coordinator(
        self,
        candidates: &mut dyn PickerCandidates<'a>,
        coordinator: TerminalCoordinator,
        log_buffer: Option<PickerLogBufferHandle>,
        many: bool,
        static_empty_is_error: bool,
        cancellation: &CancellationScope,
    ) -> Result<(), PickError<()>> {
        // No terminal cleanup is owed until acquisition succeeds. Cancellation
        // can withdraw a queued handoff instead of opening a cancelled picker.
        let mut guard = cancellation
            .cancellable_scope(async |_scope| coordinator.acquire().await)
            .await?;
        let mut terminal = None;
        if let Err(error) = (PickerTerminalBackend {
            terminal: &mut terminal,
        })
        .resume()
        {
            coordinator.poison(format!("picker terminal setup failed: {error}"));
            let release_result = guard.release().await;
            return match release_result {
                Ok(()) => Err(error.into()),
                Err(release_error) => Err(eyre::eyre!(
                    "picker terminal setup failed: {error}; guard release also failed: {release_error}"
                )
                .into()),
            };
        }
        // Do not embed the large event-loop future in the coordinator and panic-recovery
        // futures.  Besides reducing the outer future size, this gives the event loop an
        // explicit heap boundary when a picker is nested under another async application.
        let picker_loop = Box::pin(self.run_loop(
            candidates,
            &mut guard,
            &coordinator,
            &mut terminal,
            log_buffer,
            many,
            static_empty_is_error,
            cancellation,
        ));
        let result = AssertUnwindSafe(picker_loop).catch_unwind().await;
        let restore_result = if terminal.take().is_some() {
            restore_terminal().map_err(|error| {
                coordinator.poison(format!("picker terminal restoration failed: {error}"));
                error
            })
        } else {
            Ok(())
        };
        if result.is_err() {
            coordinator.poison("picker owner panicked while holding a terminal frame");
        }
        let release_result = guard.release().await;
        match result {
            Err(payload) => {
                std::panic::resume_unwind(payload);
            }
            Ok(result) => match (result, restore_result, release_result) {
                (result @ Err(_), _, _) => result,
                (Ok(_), Err(error), _) | (Ok(_), Ok(()), Err(error)) => {
                    // Successful choices were previously consumed by the loop;
                    // terminal cleanup failures do not return partial choices.
                    candidates.marked_mut().clear();
                    Err(error.into())
                }
                (Ok(outcome), Ok(()), Ok(())) => Ok(outcome),
            },
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the picker loop uses its terminal owner, presentation state, and cancellation scope directly"
    )]
    async fn run_loop(
        mut self,
        candidates: &mut dyn PickerCandidates<'a>,
        guard: &mut TerminalGuard,
        coordinator: &TerminalCoordinator,
        terminal: &mut Option<PickerTerminal>,
        log_buffer: Option<PickerLogBufferHandle>,
        many: bool,
        static_empty_is_error: bool,
        owner: &CancellationScope,
    ) -> Result<(), PickError<()>> {
        let mut cancellation = self
            .cancellation
            .take()
            .unwrap_or_else(|| Box::pin(pending()));
        let mut handler_tasks = HandlerTasks::new(owner.clone());

        for event in startup_events(&self.default_query) {
            candidates.spawn_handlers(&mut handler_tasks, event, true);
        }

        let mut nucleo = new_nucleo();
        let mut warned_tab_keys = FxHashSet::<CompactString>::default();
        let mut search_results_keys = Vec::<Arc<CompactString>>::new();
        let mut search_results_heights = Vec::<usize>::new();
        let mut list_state = ListState::default();
        let mut query_text_area = Self::build_text_area(&self.default_query);
        let mut previous_query = None::<String>;
        let mut query_changed = true;
        let mut selection_needs_reset = false;
        let mut query_debouncer = QueryDebouncer::default();
        let mut event_stream = EventStream::new();
        let mut ticker = tokio::time::interval(Duration::from_millis(16));
        let mut return_reason = None::<Result<(), PickError<()>>>;
        let mut render_dirty = true;
        let mut log_cursor = usize::MAX;
        if let Some(log_buffer) = &log_buffer {
            log_buffer.records_since(&mut log_cursor);
        }
        let mut toasts = Vec::<PickerToast>::new();

        // Every UI exit (including input, rendering, and coordinator failures)
        // reaches the same drain. Catch UI panics while the tasks are still owned.
        let outcome = AssertUnwindSafe(async {
        let return_reason = loop {
            let debounce = query_debouncer
                .deadline()
                .map(|deadline| tokio::time::sleep_until(deadline.into()))
                .unwrap_or_else(|| tokio::time::sleep(Duration::from_secs(86_400)));
            tokio::pin!(debounce);
            let mut keyboard_event_ready = false;
            tokio::select! {
                // Prefer already-buffered terminal input over continuously-ready background
                // work so navigation keys are handled with the lowest possible queueing delay.
                biased;

                control = guard.next_control() => {
                    let Some(control) = control else {
                        return Err(
                            eyre::eyre!("terminal coordinator owner channel closed").into()
                        );
                    };
                    apply_terminal_control(control, guard, terminal)?;
                    render_dirty = true;
                }
                () = &mut cancellation => {
                    return_reason = Some(Err(PickError::Cancelled));
                }
                () = owner.cancelled() => {
                    // Intent closes the UI even while a handler is protected.
                    // Preserve the execution cause; the common drain below
                    // still finishes every admitted handler before returning it.
                    return_reason = Some(Err(CancellationError::new(
                        owner.cancellation_reason().unwrap_or_else(|| "Picker execution cancelled".to_owned()),
                    ).into()));
                }
                input = event_stream.next(), if terminal.is_some() => {
                    keyboard_event_ready = true;
                    match input {
                        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                            self.cancel_initial_focus();
                            let effects =
                                extended_trace_span!("picker_handle_key").in_scope(|| {
                                    handle_key(
                                        key,
                                        many,
                                        self.allow_empty_selection,
                                        self.include_focused_on_enter,
                                        &mut list_state,
                                        &search_results_keys,
                                        candidates.marked_mut(),
                                        &mut query_text_area,
                                        &mut query_changed,
                                        &mut selection_needs_reset,
                                        &mut query_debouncer,
                                        &mut return_reason,
                                    )
                                });
                            render_dirty |= effects.render_requested;
                        }
                        Some(Ok(Event::Resize(_, _))) => render_dirty = true,
                        Some(Ok(_)) => {}
                        Some(Err(error)) => return Err(error.into()),
                        None => return_reason = Some(Err(PickError::Cancelled)),
                    }
                    if return_reason.is_none() {
                        let deadline = Instant::now() + Duration::from_millis(2);
                        while Instant::now() < deadline {
                            let Some(next_input) =
                                event_stream.next().now_or_never().flatten()
                            else {
                                break;
                            };
                            match next_input {
                                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                                    self.cancel_initial_focus();
                                    let effects =
                                        extended_trace_span!("picker_handle_key").in_scope(|| {
                                            handle_key(
                                                key,
                                                many,
                                                self.allow_empty_selection,
                                                self.include_focused_on_enter,
                                                &mut list_state,
                                                &search_results_keys,
                                                candidates.marked_mut(),
                                                &mut query_text_area,
                                                &mut query_changed,
                                                &mut selection_needs_reset,
                                                &mut query_debouncer,
                                                &mut return_reason,
                                            )
                                        });
                                    render_dirty |= effects.render_requested;
                                    if return_reason.is_some() {
                                        break;
                                    }
                                }
                                Ok(Event::Resize(_, _)) => render_dirty = true,
                                Ok(_) => {}
                                Err(error) => return Err(error.into()),
                            }
                        }
                    }
                }
                Some((changed, tab_warning_keys)) = poll_fn(|cx| candidates.poll_update(cx, &mut nucleo, &mut warned_tab_keys)) => {
                    show_tab_warnings(tab_warning_keys, &mut toasts);
                    render_dirty |= changed;
                    query_changed |= changed;
                }
                joined = handler_tasks.next(), if !handler_tasks.is_empty() => {
                    if let Some(Err(error)) = joined {
                        return_reason = Some(Err(error.into()));
                    }
                    render_dirty = true;
                }
                _ = &mut debounce, if query_debouncer.deadline().is_some() => {
                    if let Some(query) = query_debouncer.take_due(Instant::now()) {
                        render_dirty = true;
                        candidates.spawn_handlers(
                            &mut handler_tasks,
                            match query {
                                QueryEvent::Cleared => PickerEvent::QueryCleared,
                                QueryEvent::Changed(query) => {
                                    PickerEvent::QueryChanged(Arc::<str>::from(query))
                                }
                            },
                            false,
                        );
                    }
                }
                _ = ticker.tick() => {}
            }

            // A coordinator handoff has priority over any other work that became ready during
            // the select. It must be acknowledged before the owner can safely draw again.
            while let Some(control) = guard.try_next_control()? {
                apply_terminal_control(control, guard, terminal)?;
                render_dirty = true;
            }

            // Once input is observed, consolidate the ready work into the same frame. The
            // deadline prevents a continuously-producing backend from starving another key.
            if return_reason.is_none() {
                let coalesce_deadline =
                    keyboard_event_ready.then(|| Instant::now() + Duration::from_millis(2));
                let mut tab_warning_keys = Vec::new();
                loop {
                    let mut progressed = false;

                    while let Some((changed, warning_keys)) = candidates.try_update(&mut nucleo, &mut warned_tab_keys) {
                        progressed = true;
                        tab_warning_keys.extend(warning_keys);
                        render_dirty |= changed;
                        query_changed |= changed;
                        if coalesce_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                            break;
                        }
                    }

                    while !handler_tasks.is_empty() {
                        let Some(ready) = handler_tasks.next().now_or_never() else {
                            break;
                        };
                        let Some(completion) = ready else {
                            break;
                        };
                        progressed = true;
                        if let Err(error) = completion {
                            return_reason.get_or_insert(Err(error.into()));
                        }
                        render_dirty = true;
                        if coalesce_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                            break;
                        }
                    }

                    if query_debouncer
                        .deadline()
                        .is_some_and(|deadline| Instant::now() >= deadline)
                        && let Some(query) = query_debouncer.take_due(Instant::now())
                    {
                        progressed = true;
                        candidates.spawn_handlers(
                            &mut handler_tasks,
                            match query {
                                QueryEvent::Cleared => PickerEvent::QueryCleared,
                                QueryEvent::Changed(query) => {
                                    PickerEvent::QueryChanged(Arc::<str>::from(query))
                                }
                            },
                            false,
                        );
                        render_dirty = true;
                    }

                    while let Some(control) = guard.try_next_control()? {
                        progressed = true;
                        apply_terminal_control(control, guard, terminal)?;
                        render_dirty = true;
                    }

                    if coalesce_deadline.is_some_and(|deadline| Instant::now() >= deadline)
                        || !progressed
                    {
                        break;
                    }
                }

                show_tab_warnings(tab_warning_keys, &mut toasts);
            }

            if let Some(log_buffer) = &log_buffer {
                for record in log_buffer.records_since(&mut log_cursor) {
                    if should_show_as_toast(record.level) {
                        toasts.push(PickerToast {
                            record,
                            expires_at: Instant::now() + Duration::from_secs(4),
                        });
                        render_dirty = true;
                    }
                }
            }
            let now = Instant::now();
            render_dirty |= advance_toasts(&mut toasts, now);

            if matches!(return_reason, Some(Err(PickError::ReloadRequested))) {
                handler_tasks.restart();
                candidates.reload();
                debug_assert_eq!(handler_tasks.generation(), candidates.generation());
                nucleo = new_nucleo();
                search_results_keys.clear();
                search_results_heights.clear();
                list_state.select(None);
                previous_query = None;
                query_changed = true;
                selection_needs_reset = true;
                query_debouncer.clear();
                return_reason = None;
                render_dirty = true;
                candidates.spawn_handlers(
                    &mut handler_tasks,
                    PickerEvent::ReloadRequested(Arc::<str>::from(
                        query_text_area.lines().join("\n"),
                    )),
                    true,
                );
            } else if let Some(return_reason) = return_reason.take() {
                break return_reason;
            }

            if query_changed {
                let new_query = extended_trace_span!("picker_reparse_query").in_scope(|| {
                    let new_query = query_text_area.lines().join("\n");
                    nucleo.pattern.reparse(
                        0,
                        &new_query,
                        CaseMatching::Smart,
                        Normalization::Smart,
                        previous_query
                            .as_deref()
                            .is_some_and(|previous| new_query.starts_with(previous)),
                    );
                    new_query
                });
                previous_query = Some(new_query);
                query_changed = false;
            }

            let status = extended_trace_span!("picker_nucleo_tick").in_scope(|| nucleo.tick(10));
            if status.changed {
                render_dirty = true;
                extended_trace_span!("picker_rebuild_results").in_scope(|| {
                    rebuild_results(
                        &nucleo,
                        candidates,
                        &mut list_state,
                        &mut search_results_keys,
                        &mut search_results_heights,
                        selection_needs_reset,
                    )
                });
                selection_needs_reset = false;
                self.apply_initial_focus(&mut list_state, &search_results_keys);
            }

            if self.auto_accept
                && handler_tasks.startup() == 0
                && candidates.len() == 1
                && !many
            {
                return_reason = Some(Ok(()));
            }
            if static_empty_is_error
                && handler_tasks.startup() == 0
                && handler_tasks.pending() == 0
                && candidates.channel_is_empty()
                && candidates.len() == 0
            {
                return Err(PickError::NoChoicesProvided);
            }

            if render_dirty {
                let counts_title = if many {
                    format!(
                        "{} items marked for return of {} items matching query of {} items total",
                        candidates.marked().len(),
                        search_results_keys.len(),
                        candidates.len(),
                    )
                } else {
                    format!(
                        "{} items matching query of {} items total",
                        search_results_keys.len(),
                        candidates.len(),
                    )
                };
                let mut query_block = Block::bordered().title(counts_title);
                if let Some(pending_title) = pending_title(handler_tasks.pending()) {
                    query_block =
                        query_block.title_bottom(Line::from(pending_title).left_aligned());
                }
                query_text_area.set_block(query_block);

                extended_trace_span!("picker_render").in_scope(|| -> eyre::Result<()> {
                    let Some(terminal) = terminal.as_mut() else {
                        return Ok(());
                    };
                    terminal
                        .draw(|frame| {
                            let area = frame.area();
                            let buf = frame.buffer_mut();
                            let [list_area, searchbox_area] =
                                Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
                                    .areas(area);
                            if search_results_keys.is_empty() {
                                let empty_message =
                                    empty_picker_message(query_text_area.is_empty());
                                Paragraph::new(empty_message)
                                    .block(list_block(self.header.as_deref()))
                                    .render(list_area, buf);
                            } else {
                                render_picker_list(
                                    buf,
                                    list_area,
                                    self.header.as_deref(),
                                    many,
                                    &search_results_keys,
                                    &search_results_heights,
                                    candidates.marked(),
                                    &mut list_state,
                                );
                            }
                            if query_text_area.is_empty() {
                                Paragraph::new("Type to search".gray())
                                    .block(
                                        query_text_area
                                            .block()
                                            .cloned()
                                            .unwrap_or_else(Block::bordered),
                                    )
                                    .render(searchbox_area, buf);
                            } else {
                                query_text_area.render(searchbox_area, buf);
                            }
                            render_toasts(buf, area, &toasts);
                        })
                        .map(|_| ())
                        .map_err(|error| {
                            coordinator.poison(format!("picker frame draw failed: {error}"));
                            eyre::eyre!(error)
                        })?;
                    Ok(())
                })?;
                #[cfg(feature = "extended_observability")]
                tracing::info!(message = "finished picker frame", tracy.frame_mark = true);
                render_dirty = false;
            }
        };

        while candidates.try_update(&mut nucleo, &mut warned_tab_keys).is_some() {}

        return_reason
        }).catch_unwind().await;

        // Even a suspended parent must service handoffs: a nested picker cannot
        // finish releasing its frame until its parent acknowledges the resume.
        let drain_result = drain_picker_handlers(
            &mut handler_tasks,
            guard,
            &mut PickerTerminalBackend { terminal },
        )
        .await;
        let return_reason = match outcome {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        if let Err(error) = drain_result {
            let cancelled = match &return_reason {
                Err(PickError::Cancelled) => true,
                Err(PickError::Eyre(error, _)) => error.is_cancellation(),
                _ => false,
            };
            if return_reason.is_ok() || cancelled {
                return Err(error.into());
            }
            tracing::warn!(%error, "picker handler also failed during shutdown");
        }
        return_reason
    }
}

pub(super) fn startup_events(default_query: &str) -> Vec<PickerEvent> {
    let mut events = vec![PickerEvent::InitialLoad];
    if !default_query.is_empty() {
        events.push(PickerEvent::QueryChanged(Arc::<str>::from(default_query)));
    }
    events
}

// Keep warnings in the picker. A blocking stdin prompt would outlive cancellation
// and prevent a nested terminal owner from completing its handoff.
fn show_tab_warnings(keys: Vec<CompactString>, toasts: &mut Vec<PickerToast>) {
    for key in keys {
        tracing::warn!(%key, "A picker candidate contains a tab character and may render poorly");
        toasts.push(PickerToast {
            record: PickerLogRecord {
                timestamp: std::time::SystemTime::now(),
                level: PickerLogLevel::Warn,
                message: Arc::from(format!("Candidate contains a tab: {key:?}")),
                target: Arc::from(module_path!()),
                fields: Vec::new(),
                spans: Vec::new(),
                file: Some(Arc::from(file!())),
                line: Some(line!()),
            },
            expires_at: Instant::now() + Duration::from_secs(4),
        });
    }
}

fn apply_terminal_control(
    control: TerminalControl,
    guard: &mut TerminalGuard,
    terminal: &mut Option<PickerTerminal>,
) -> eyre::Result<()> {
    let mut backend = PickerTerminalBackend { terminal };
    apply_terminal_control_with_backend(control, guard, &mut backend)
}

pub(super) fn new_nucleo() -> Nucleo<Arc<CompactString>> {
    Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1)
}

fn list_block(header: Option<&str>) -> Block<'static> {
    let mut block = Block::bordered();
    if let Some(header) = header {
        block = block.title(header.to_owned());
    }
    block
}

pub(super) const fn empty_picker_message(query_is_empty: bool) -> &'static str {
    if query_is_empty {
        "No choices yet, try typing to search"
    } else {
        "No results"
    }
}

fn pending_title(pending_handlers: usize) -> Option<String> {
    if pending_handlers == 0 {
        return None;
    }

    Some(format!("loading {pending_handlers} background tasks"))
}

pub(super) fn should_show_as_toast(level: PickerLogLevel) -> bool {
    matches!(
        level,
        PickerLogLevel::Info | PickerLogLevel::Warn | PickerLogLevel::Error
    )
}

pub(super) fn advance_toasts(toasts: &mut Vec<PickerToast>, now: Instant) -> bool {
    let count = toasts.len();
    toasts.retain(|toast| toast.expires_at > now);
    count != toasts.len()
        || toasts.iter().any(|toast| {
            toast.expires_at.saturating_duration_since(now) < Duration::from_millis(700)
        })
}

pub(super) fn render_toasts(buf: &mut Buffer, area: Rect, toasts: &[PickerToast]) {
    let max_width = toasts
        .iter()
        .rev()
        .take(area.height as usize)
        .map(|toast| toast.record.message.chars().count().saturating_add(4))
        .max()
        .unwrap_or_default()
        .min(area.width as usize)
        .max(1) as u16;
    let mut bottom = area.bottom();
    for toast in toasts.iter().rev() {
        if bottom <= area.top() {
            break;
        }
        let message_width = toast.record.message.chars().count().saturating_add(4);
        let left_padding = max_width.saturating_sub(message_width as u16) as usize;
        let message = format!("{}{}", " ".repeat(left_padding), toast.record.message);
        let width = max_width;
        let height = 1u16;
        let y = bottom.saturating_sub(height);
        let x = area.right().saturating_sub(width);
        let toast_area = Rect::new(x, y, width, height);
        let fading =
            toast.expires_at.saturating_duration_since(Instant::now()) < Duration::from_millis(700);
        let style = match (toast.record.level, fading) {
            (PickerLogLevel::Info, false) => Style::new().fg(Color::White).bg(Color::Blue),
            (PickerLogLevel::Warn, false) => Style::new().fg(Color::Black).bg(Color::Yellow),
            (PickerLogLevel::Error, false) => Style::new().fg(Color::White).bg(Color::Red),
            (_, true) => Style::new().fg(Color::DarkGray).bg(Color::Black),
            (PickerLogLevel::Debug, _) => Style::default(),
        };
        for x in toast_area.left()..toast_area.right() {
            buf[(x, y)].set_symbol(" ").set_style(style);
        }
        Paragraph::new(message).style(style).render(toast_area, buf);
        bottom = y;
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "key handling mutates the picker state components owned by the run loop"
)]
pub(super) fn handle_key(
    key: KeyEvent,
    many: bool,
    allow_empty_selection: bool,
    include_focused_on_enter: bool,
    list_state: &mut ListState,
    search_results_keys: &[Arc<CompactString>],
    marked_for_return: &mut FxHashSet<CompactString>,
    query_text_area: &mut TextArea<'static>,
    query_changed: &mut bool,
    selection_needs_reset: &mut bool,
    query_debouncer: &mut QueryDebouncer,
    return_reason: &mut Option<Result<(), PickError<()>>>,
) -> KeyEffects {
    let mut effects = KeyEffects::default();
    match key.code {
        KeyCode::Esc => *return_reason = Some(Err(PickError::Cancelled)),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *return_reason = Some(Err(PickError::Cancelled))
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *return_reason = Some(Err(PickError::ReloadRequested))
        }
        KeyCode::Up => {
            let previous = list_state.selected();
            list_state.select_previous();
            effects.render_requested = previous != list_state.selected();
        }
        KeyCode::Down => {
            let previous = list_state.selected();
            list_state.select_next();
            effects.render_requested = previous != list_state.selected();
        }
        KeyCode::Tab => {
            if many
                && let Some(selected_item) = list_state
                    .selected()
                    .and_then(|index| search_results_keys.get(index))
            {
                if !marked_for_return.insert(selected_item.as_ref().clone()) {
                    marked_for_return.remove(selected_item.as_ref());
                }
                effects.render_requested = true;
                list_state.select_next();
            }
        }
        KeyCode::Enter => {
            if (!many
                || (!allow_empty_selection && marked_for_return.is_empty())
                || (include_focused_on_enter && !key.modifiers.contains(KeyModifiers::CONTROL)))
                && let Some(selected_index) = list_state.selected()
                && let Some(selected_key) = search_results_keys.get(selected_index)
            {
                marked_for_return.insert(selected_key.as_ref().clone());
            }
            *return_reason = Some(Ok(()));
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            marked_for_return.extend(search_results_keys.iter().map(|key| key.as_ref().clone()));
            effects.render_requested = true;
        }
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *marked_for_return = search_results_keys
                .iter()
                .filter(|key| !marked_for_return.contains(key.as_ref()))
                .map(|key| key.as_ref().clone())
                .collect();
            effects.render_requested = true;
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            marked_for_return.clear();
            effects.render_requested = true;
        }
        KeyCode::PageUp => {
            if let Some(selected) = list_state.selected() {
                list_state.select(Some(selected.saturating_sub(10)));
                effects.render_requested = Some(selected) != list_state.selected();
            }
        }
        KeyCode::PageDown => {
            if let Some(selected) = list_state.selected() {
                let next = selected
                    .saturating_add(10)
                    .min(search_results_keys.len().saturating_sub(1));
                list_state.select(Some(next));
                effects.render_requested = Some(selected) != list_state.selected();
            }
        }
        KeyCode::Home => {
            let previous = list_state.selected();
            list_state.select(Some(0));
            effects.render_requested = previous != list_state.selected();
        }
        KeyCode::End => {
            let previous = list_state.selected();
            list_state.select(Some(search_results_keys.len().saturating_sub(1)));
            effects.render_requested = previous != list_state.selected();
        }
        KeyCode::BackTab if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *query_changed = query_text_area.delete_word();
            if *query_changed {
                *selection_needs_reset = true;
                list_state.select(Some(0));
                *list_state.offset_mut() = 0;
            }
            effects.render_requested = *query_changed;
        }
        _ => {
            *query_changed = query_text_area.input(key);
            if *query_changed {
                *selection_needs_reset = true;
                list_state.select(Some(0));
                *list_state.offset_mut() = 0;
            }
            effects.render_requested = *query_changed;
        }
    }

    if *query_changed {
        query_debouncer.schedule(query_text_area.lines().join("\n"), Instant::now());
    }

    effects
}

pub(super) fn row_style(marked: bool, cursor: bool) -> Style {
    match (marked, cursor) {
        (true, true) => Style::new().bg(Color::Magenta),
        (true, false) => Style::new().bg(Color::DarkGray),
        (false, true) => Style::new().bg(Color::Blue),
        (false, false) => Style::default(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "rendering receives separate terminal layout and picker selection state"
)]
pub(super) fn render_picker_list(
    buf: &mut Buffer,
    area: Rect,
    header: Option<&str>,
    many: bool,
    search_results_keys: &[Arc<CompactString>],
    search_results_heights: &[usize],
    marked_for_return: &FxHashSet<CompactString>,
    list_state: &mut ListState,
) {
    let block = list_block(header);
    let list_area = block.inner(area);
    block.render(area, buf);

    if list_area.is_empty() || search_results_keys.is_empty() {
        list_state.select(None);
        return;
    }

    let last_index = search_results_keys.len() - 1;
    let selected_index = match list_state.selected() {
        Some(index) if index <= last_index => Some(index),
        Some(_) => {
            list_state.select(Some(last_index));
            Some(last_index)
        }
        None => None,
    };
    let max_height = list_area.height as usize;
    let mut first_visible_index = list_state.offset().min(last_index);

    if let Some(selected_index) = selected_index {
        if selected_index < first_visible_index {
            first_visible_index = selected_index;
        } else {
            let mut visible_height = 0;
            let mut visible_end = first_visible_index;
            while visible_end <= last_index {
                let row_height = search_results_heights[visible_end].max(1);
                if visible_height + row_height > max_height && visible_end > first_visible_index {
                    break;
                }
                visible_height += row_height;
                visible_end += 1;
                if visible_height >= max_height {
                    break;
                }
            }
            if selected_index >= visible_end {
                let mut selected_window_start = selected_index;
                let mut selected_window_height = search_results_heights[selected_index].max(1);
                while selected_window_start > 0 {
                    let previous_height = search_results_heights[selected_window_start - 1].max(1);
                    if selected_window_height.saturating_add(previous_height) > max_height {
                        break;
                    }
                    selected_window_height += previous_height;
                    selected_window_start -= 1;
                }
                first_visible_index = selected_window_start;
            }
        }
    }
    *list_state.offset_mut() = first_visible_index;

    let mut y = list_area.top();

    for index in first_visible_index..search_results_keys.len() {
        if y >= list_area.bottom() {
            break;
        }

        let height = search_results_heights[index].max(1) as u16;
        let row_height = height.min(list_area.bottom().saturating_sub(y));
        let row_area = Rect::new(list_area.left(), y, list_area.width, row_height);
        let key = search_results_keys[index].as_ref();
        let marked = many && marked_for_return.contains(key);
        let item_style = row_style(marked, selected_index == Some(index));
        let dot_style = item_style.fg(Color::Red);
        for row_y in row_area.top()..row_area.bottom() {
            for x in row_area.left()..row_area.right() {
                buf[(x, row_y)].reset();
                buf[(x, row_y)].set_style(item_style);
            }
        }

        for (line_index, line) in key.lines().enumerate() {
            let line_y = y + line_index as u16;
            if line_y >= list_area.bottom() {
                break;
            }

            let (prefix, prefix_style, key_x) = if marked {
                if line_index == 0 {
                    ("  • ", dot_style, list_area.left().saturating_add(4))
                } else {
                    ("    ", item_style, list_area.left().saturating_add(4))
                }
            } else {
                ("", item_style, list_area.left())
            };
            let prefix_width = if marked { 4 } else { 0 };
            buf.set_stringn(
                list_area.left(),
                line_y,
                prefix,
                (list_area.width as usize).min(prefix_width),
                prefix_style,
            );
            buf.set_stringn(
                key_x,
                line_y,
                line,
                (list_area.width as usize).saturating_sub(prefix_width),
                item_style,
            );
        }

        y = y.saturating_add(height);
    }
}

fn rebuild_results(
    nucleo: &Nucleo<Arc<CompactString>>,
    candidate_pool: &dyn PickerCandidates<'_>,
    list_state: &mut ListState,
    search_results_keys: &mut Vec<Arc<CompactString>>,
    search_results_heights: &mut Vec<usize>,
    selection_needs_reset: bool,
) {
    let selected_key = list_state
        .selected()
        .and_then(|index| search_results_keys.get(index))
        .cloned();
    search_results_keys.clear();
    search_results_heights.clear();
    for item in nucleo.snapshot().matched_items(..) {
        let key = Arc::clone(item.data);
        let height = key.lines().count().max(1);
        search_results_keys.push(key);
        search_results_heights.push(height);
    }

    let selected_index = if selection_needs_reset {
        (!search_results_keys.is_empty()).then_some(0)
    } else {
        preserved_selection(selected_key.as_deref(), search_results_keys)
    };
    list_state.select(selected_index);
    if selection_needs_reset {
        *list_state.offset_mut() = 0;
    }
    debug_assert_eq!(
        search_results_keys
            .iter()
            .filter(|key| candidate_pool.contains_key(key.as_str()))
            .count(),
        search_results_keys.len()
    );
}

#[cfg(test)]
mod tests {
    use super::PickerUi;
    use compact_str::CompactString;
    use ratatui::widgets::ListState;
    use std::sync::Arc;

    #[test]
    fn initial_focus_waits_for_its_candidate_then_leaves_later_navigation_alone() {
        let mut picker = PickerUi {
            default_query: String::new(),
            header: None,
            auto_accept: true,
            allow_empty_selection: false,
            include_focused_on_enter: false,
            initial_focused: Some(CompactString::from("None")),
            cancellation: None,
            terminal_coordinator: None,
            log_buffer: None,
        };
        let mut list_state = ListState::default();
        picker.apply_initial_focus(&mut list_state, &[]);
        assert_eq!(list_state.selected(), None);
        let partial = vec![Arc::new(CompactString::from("Unresolved"))];
        list_state.select(Some(0));
        picker.apply_initial_focus(&mut list_state, &partial);
        assert_eq!(list_state.selected(), Some(0));

        let choices = vec![
            Arc::new(CompactString::from("Unresolved")),
            Arc::new(CompactString::from("None")),
            Arc::new(CompactString::from("Resolved")),
        ];
        picker.apply_initial_focus(&mut list_state, &choices);
        assert_eq!(list_state.selected(), Some(1));
        list_state.select(Some(2));
        picker.apply_initial_focus(&mut list_state, &choices);
        assert_eq!(
            list_state.selected(),
            Some(2),
            "a later result update must not restore initial focus"
        );
    }

    #[test]
    fn user_input_cancels_initial_focus_before_delayed_results_arrive() {
        let mut picker = PickerUi {
            default_query: String::new(),
            header: None,
            auto_accept: true,
            allow_empty_selection: false,
            include_focused_on_enter: false,
            initial_focused: Some(CompactString::from("None")),
            cancellation: None,
            terminal_coordinator: None,
            log_buffer: None,
        };
        let mut list_state = ListState::default();
        picker.apply_initial_focus(&mut list_state, &[]);
        picker.cancel_initial_focus();
        list_state.select(Some(0));
        picker.apply_initial_focus(
            &mut list_state,
            &[
                Arc::new(CompactString::from("Unresolved")),
                Arc::new(CompactString::from("None")),
            ],
        );
        assert_eq!(list_state.selected(), Some(0));
    }
}
