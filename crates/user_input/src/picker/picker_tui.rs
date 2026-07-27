use super::candidate_message::CandidateMessage;
use super::candidate_sink::CandidateSink;
use super::choice_pool::ChoicePool;
use super::event_handler::EventHandler;
use super::handler_completion::HandlerCompletion;
use super::handler_task::HandlerTask;
use super::picker_event::PickerEvent;
use super::picker_event_state::PickerEventState;
use super::preserved_selection::preserved_selection;
use super::query_debouncer::QueryDebouncer;
use super::query_event::QueryEvent;
use super::return_reason::ReturnReason;
use super::run_outcome::RunOutcome;
use super::should_warn_for_tab::should_warn_for_tab;
use crate::IntoChoices;
use crate::PickError;
use crate::PickResult;
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
use futures::stream::FuturesUnordered;
use nucleo::Nucleo;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use ratatui::Terminal;
use ratatui::buffer::Buffer;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::EnterAlternateScreen;
use ratatui::crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::enable_raw_mode;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use rustc_hash::FxHashSet;
use std::collections::VecDeque;
use std::future::Future;
use std::io::BufWriter;
use std::io::Stderr;
use std::io::stderr;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::Instrument;
use tracing::info_span;
use tracing::trace_span;
use tui_textarea::CursorMove;
use tui_textarea::TextArea;

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

pub struct PickerTui<'a, T> {
    pub default_query: String,
    pub header: Option<String>,
    pub auto_accept: bool,
    terminal_coordinator: Option<TerminalCoordinator>,
    log_buffer: Option<PickerLogBufferHandle>,
    handlers: Vec<EventHandler<'a, T>>,
}

impl<'a, T> Default for PickerTui<'a, T> {
    fn default() -> Self {
        Self {
            default_query: Default::default(),
            header: Default::default(),
            auto_accept: true,
            terminal_coordinator: None,
            log_buffer: None,
            handlers: Vec::new(),
        }
    }
}

impl<'a, T> PickerTui<'a, T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn build_text_area(query: &str) -> TextArea<'static> {
        let mut text_area = TextArea::new(vec![query.to_string()]);
        text_area.move_cursor(CursorMove::End);
        text_area.set_block(Block::bordered());
        text_area
    }

    pub fn set_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn set_auto_accept(mut self, auto_accept: bool) -> Self {
        self.auto_accept = auto_accept;
        self
    }

    pub fn set_query(mut self, query: impl Into<String>) -> Self {
        self.default_query = query.into();
        self
    }

    pub fn terminal_coordinator(mut self, coordinator: TerminalCoordinator) -> Self {
        self.terminal_coordinator = Some(coordinator);
        self
    }

    pub fn log_buffer(mut self, log_buffer: PickerLogBufferHandle) -> Self {
        self.log_buffer = Some(log_buffer);
        self
    }

    pub fn add_event_handler<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(Arc<PickerEvent>, CandidateSink<T>) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<()>> + Send + 'a,
    {
        let mut picker = self;
        picker.handlers.push(EventHandler {
            handler: Box::new(move |event, sink| Box::pin(handler(event, sink))),
        });
        picker
    }

    pub async fn pick_one_events(self) -> PickResult<T>
    where
        T: Send + 'a,
    {
        self.run(false, false)
            .await
            .map(|mut items| items.remove(0))
    }

    pub async fn pick_many_events(self) -> PickResult<Vec<T>>
    where
        T: Send + 'a,
    {
        self.run(true, false).await
    }

    pub async fn pick_one(self, choices: impl IntoChoices<T>) -> PickResult<T>
    where
        T: Send + 'a,
    {
        self.with_initial_choices(choices)
            .run(false, true)
            .await
            .map(|mut items| items.remove(0))
    }

    pub async fn pick_many(self, choices: impl IntoChoices<T>) -> PickResult<Vec<T>>
    where
        T: Send + 'a,
    {
        self.with_initial_choices(choices).run(true, true).await
    }

    pub async fn pick_inner(self, many: bool, choices: impl IntoChoices<T>) -> PickResult<Vec<T>>
    where
        T: Send + 'a,
    {
        self.with_initial_choices(choices).run(many, true).await
    }

    pub async fn pick_one_reloadable<F, Fut, C>(self, choice_supplier: F) -> PickResult<T>
    where
        T: Send + 'a,
        F: Fn(bool) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<C>> + Send + 'a,
        C: IntoChoices<T> + 'a,
    {
        self.with_reload_handler(choice_supplier)
            .run(false, false)
            .await
            .map(|mut items| items.remove(0))
    }

    pub async fn pick_many_reloadable<F, Fut, C>(self, choice_supplier: F) -> PickResult<Vec<T>>
    where
        T: Send + 'a,
        F: Fn(bool) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<C>> + Send + 'a,
        C: IntoChoices<T> + 'a,
    {
        self.with_reload_handler(choice_supplier)
            .run(true, false)
            .await
    }

    fn with_initial_choices(self, choices: impl IntoChoices<T>) -> Self
    where
        T: Send + 'a,
    {
        let choices =
            info_span!("picker_prepare_initial_choices").in_scope(|| choices.into_choices());
        let choices = Arc::new(Mutex::new(Some(choices)));
        self.add_event_handler(move |event, sink| {
            let choices = choices.clone();
            async move {
                if matches!(event.as_ref(), PickerEvent::InitialLoad) {
                    if let Some(choices) = choices.lock().expect("choices mutex poisoned").take() {
                        sink.push(choices)?;
                    }
                }
                Ok(())
            }
        })
    }

    fn with_reload_handler<F, Fut, C>(self, choice_supplier: F) -> Self
    where
        T: Send + 'a,
        F: Fn(bool) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<C>> + Send + 'a,
        C: IntoChoices<T> + 'a,
    {
        let choice_supplier = Arc::new(Mutex::new(choice_supplier));
        self.add_event_handler(move |event, sink| {
            let choice_supplier = Arc::clone(&choice_supplier);
            let invalidate = matches!(event.as_ref(), PickerEvent::ReloadRequested(_));
            async move {
                if matches!(
                    event.as_ref(),
                    PickerEvent::InitialLoad | PickerEvent::ReloadRequested(_)
                ) {
                    let future = {
                        let choice_supplier = choice_supplier
                            .lock()
                            .expect("choice supplier mutex poisoned");
                        (choice_supplier)(invalidate)
                    };
                    let choices = future.await?;
                    sink.push(choices)?;
                }
                Ok(())
            }
        })
    }

    async fn run(self, many: bool, static_empty_is_error: bool) -> PickResult<Vec<T>>
    where
        T: Send + 'a,
    {
        match self.run_inner(many, static_empty_is_error).await {
            Ok(RunOutcome::Selected(items)) => Ok(items),
            Ok(RunOutcome::Cancelled) => Err(PickError::Cancelled),
            Ok(RunOutcome::ReloadRequested) => Err(PickError::ReloadRequested),
            Ok(RunOutcome::NoChoices) => Err(PickError::NoChoicesProvided),
            Err(error) => Err(PickError::Eyre(error)),
        }
    }

    #[tracing::instrument(
        name = "picker_session",
        skip_all,
        fields(many = many, static_empty_is_error = static_empty_is_error),
    )]
    async fn run_inner(self, many: bool, static_empty_is_error: bool) -> eyre::Result<RunOutcome<T>>
    where
        T: Send + 'a,
    {
        let explicit_coordinator = self.terminal_coordinator.clone();
        let current_coordinator = TerminalCoordinator::try_current();
        #[cfg(feature = "terminal_coordinator_debug")]
        if let (Some(explicit), Some(current)) = (&explicit_coordinator, &current_coordinator) {
            explicit.debug_assert_matches_current(current)?;
        }
        let coordinator = explicit_coordinator
            .or(current_coordinator)
            .unwrap_or_else(TerminalCoordinator::new);
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
            coordinator.clone(),
            log_buffer.clone(),
            many,
            static_empty_is_error,
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
        coordinator: TerminalCoordinator,
        log_buffer: Option<PickerLogBufferHandle>,
        many: bool,
        static_empty_is_error: bool,
    ) -> eyre::Result<RunOutcome<T>>
    where
        T: Send + 'a,
    {
        let mut guard = coordinator.acquire().await?;
        let mut terminal = None;
        if let Err(error) = (PickerTerminalBackend {
            terminal: &mut terminal,
        })
        .resume()
        {
            coordinator.poison(format!("picker terminal setup failed: {error}"));
            let release_result = guard.release().await;
            return match release_result {
                Ok(()) => Err(error),
                Err(release_error) => Err(eyre::eyre!(
                    "picker terminal setup failed: {error}; guard release also failed: {release_error}"
                )),
            };
        }
        let original_hook = Arc::new(std::panic::take_hook());
        let panic_hook = original_hook.clone();
        std::panic::set_hook(Box::new(move |info| {
            if let Err(error) = restore_terminal() {
                eprintln!("Failed to restore picker terminal during panic: {error}");
            }
            panic_hook(info);
        }));
        // Do not embed the large event-loop future in the coordinator and panic-recovery
        // futures.  Besides reducing the outer future size, this gives the event loop an
        // explicit heap boundary when a picker is nested under another async application.
        let picker_loop = Box::pin(self.run_loop(
            &mut guard,
            &coordinator,
            &mut terminal,
            log_buffer,
            many,
            static_empty_is_error,
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
        let hook_for_restore = original_hook.clone();
        std::panic::set_hook(Box::new(move |info| hook_for_restore(info)));
        match result {
            Err(payload) => {
                std::panic::resume_unwind(payload);
            }
            Ok(result) => match (result, restore_result, release_result) {
                (Ok(outcome), Ok(()), Ok(())) => Ok(outcome),
                (Err(error), _, _) => Err(error),
                (Ok(_), Err(error), _) => Err(error),
                (Ok(_), Ok(()), Err(error)) => Err(error),
            },
        }
    }

    async fn run_loop(
        self,
        guard: &mut TerminalGuard,
        coordinator: &TerminalCoordinator,
        terminal: &mut Option<PickerTerminal>,
        log_buffer: Option<PickerLogBufferHandle>,
        many: bool,
        static_empty_is_error: bool,
    ) -> eyre::Result<RunOutcome<T>>
    where
        T: Send + 'a,
    {
        let (candidate_sender, mut candidate_receiver) =
            tokio::sync::mpsc::unbounded_channel::<CandidateMessage<T>>();
        let mut handler_tasks = FuturesUnordered::<HandlerTask<'a>>::new();
        let mut pending_handlers = 0usize;
        let mut startup_handlers = 0usize;
        let mut picker_state = PickerEventState::default();

        for event in startup_events(&self.default_query) {
            spawn_handlers(
                &self.handlers,
                event,
                true,
                picker_state.generation,
                &candidate_sender,
                &mut handler_tasks,
                &mut pending_handlers,
                &mut startup_handlers,
            );
        }

        let mut nucleo = new_nucleo();
        let mut warned_tab_keys = FxHashSet::<CompactString>::default();
        let mut search_results_keys = Vec::<Arc<CompactString>>::new();
        let mut search_results_heights = Vec::<usize>::new();
        let mut list_state = ListState::default();
        let mut query_text_area = Self::build_text_area(&self.default_query);
        let mut previous_query = None::<String>;
        let mut query_changed = true;
        let mut query_debouncer = QueryDebouncer::default();
        let mut event_stream = EventStream::new();
        let mut ticker = tokio::time::interval(Duration::from_millis(16));
        let mut return_reason = None;
        let mut render_dirty = true;
        let mut pending_tab_warnings = VecDeque::<CompactString>::new();
        let mut log_cursor = usize::MAX;
        if let Some(log_buffer) = &log_buffer {
            log_buffer.records_since(&mut log_cursor);
        }
        let mut toasts = Vec::<PickerToast>::new();

        loop {
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
                        return Err(eyre::eyre!("terminal coordinator owner channel closed"));
                    };
                    apply_terminal_control(control, guard, terminal)?;
                    render_dirty = true;
                }
                input = event_stream.next(), if terminal.is_some() => {
                    keyboard_event_ready = true;
                    match input {
                        Some(Ok(Event::Key(key))) if key.kind == KeyEventKind::Press => {
                            let effects =
                                extended_trace_span!("picker_handle_key").in_scope(|| {
                                    handle_key(
                                        key,
                                        many,
                                        &mut list_state,
                                        &search_results_keys,
                                        &mut picker_state.marked,
                                        &mut query_text_area,
                                        &mut query_changed,
                                        &mut query_debouncer,
                                        &mut return_reason,
                                    )
                                });
                            render_dirty |= effects.render_requested;
                        }
                        Some(Ok(Event::Resize(_, _))) => render_dirty = true,
                        Some(Ok(_)) => {}
                        Some(Err(error)) => return Err(error.into()),
                        None => return_reason = Some(ReturnReason::Cancelled),
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
                                    let effects =
                                        extended_trace_span!("picker_handle_key").in_scope(|| {
                                            handle_key(
                                                key,
                                                many,
                                                &mut list_state,
                                                &search_results_keys,
                                                &mut picker_state.marked,
                                                &mut query_text_area,
                                                &mut query_changed,
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
                Some(message) = candidate_receiver.recv() => {
                    let batch_span = extended_trace_span!("picker_candidate_batch");
                    let (changed, tab_warning_keys) = batch_span.in_scope(|| {
                        process_candidate_message(
                            message,
                            picker_state.generation,
                            &mut picker_state,
                            &mut nucleo,
                            &mut warned_tab_keys,
                        )
                    });
                    pending_tab_warnings.extend(tab_warning_keys);
                    render_dirty |= changed;
                    query_changed |= changed;
                }
                joined = handler_tasks.next(), if pending_handlers > 0 => {
                    match joined {
                        Some(completion) => {
                            handle_handler_completion(
                                completion,
                                &mut pending_handlers,
                                &mut startup_handlers,
                            )?;
                            render_dirty = true;
                        }
                        None => {
                            pending_handlers = 0;
                        }
                    }
                }
                _ = &mut debounce, if query_debouncer.deadline().is_some() => {
                    if let Some(query) = query_debouncer.take_due(Instant::now()) {
                        render_dirty = true;
                        spawn_handlers(
                            &self.handlers,
                            match query {
                                QueryEvent::Cleared => PickerEvent::QueryCleared,
                                QueryEvent::Changed(query) => {
                                    PickerEvent::QueryChanged(Arc::<str>::from(query))
                                }
                            },
                            false,
                            picker_state.generation,
                            &candidate_sender,
                            &mut handler_tasks,
                            &mut pending_handlers,
                            &mut startup_handlers,
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

                    while let Ok(message) = candidate_receiver.try_recv() {
                        progressed = true;
                        let batch_span = extended_trace_span!("picker_candidate_batch");
                        let (changed, warning_keys) = batch_span.in_scope(|| {
                            process_candidate_message(
                                message,
                                picker_state.generation,
                                &mut picker_state,
                                &mut nucleo,
                                &mut warned_tab_keys,
                            )
                        });
                        tab_warning_keys.extend(warning_keys);
                        render_dirty |= changed;
                        query_changed |= changed;
                        if coalesce_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                            break;
                        }
                    }

                    while pending_handlers > 0 {
                        let Some(ready) = handler_tasks.next().now_or_never() else {
                            break;
                        };
                        let Some(completion) = ready else {
                            pending_handlers = 0;
                            break;
                        };
                        progressed = true;
                        handle_handler_completion(
                            completion,
                            &mut pending_handlers,
                            &mut startup_handlers,
                        )?;
                        render_dirty = true;
                        if coalesce_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                            break;
                        }
                    }

                    if query_debouncer
                        .deadline()
                        .is_some_and(|deadline| Instant::now() >= deadline)
                    {
                        if let Some(query) = query_debouncer.take_due(Instant::now()) {
                            progressed = true;
                            spawn_handlers(
                                &self.handlers,
                                match query {
                                    QueryEvent::Cleared => PickerEvent::QueryCleared,
                                    QueryEvent::Changed(query) => {
                                        PickerEvent::QueryChanged(Arc::<str>::from(query))
                                    }
                                },
                                false,
                                picker_state.generation,
                                &candidate_sender,
                                &mut handler_tasks,
                                &mut pending_handlers,
                                &mut startup_handlers,
                            );
                            render_dirty = true;
                        }
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

                pending_tab_warnings.extend(tab_warning_keys);
            }

            while terminal.is_some()
                && return_reason.is_none()
                && let Some(key) = pending_tab_warnings.pop_front()
            {
                wait_for_tab_warning(
                    terminal,
                    &key,
                    guard,
                    &mut candidate_receiver,
                    &mut handler_tasks,
                    &mut pending_handlers,
                    &mut startup_handlers,
                    picker_state.generation,
                    &mut picker_state,
                    &mut nucleo,
                    &mut warned_tab_keys,
                    &mut pending_tab_warnings,
                )
                .instrument(extended_trace_span!("picker_tab_warning"))
                .await?;
                render_dirty = true;
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

            if matches!(return_reason, Some(ReturnReason::ReloadRequested)) {
                handler_tasks = FuturesUnordered::new();
                pending_handlers = 0;
                startup_handlers = 0;
                picker_state.reload();
                nucleo = new_nucleo();
                search_results_keys.clear();
                search_results_heights.clear();
                list_state.select(None);
                previous_query = None;
                query_changed = true;
                query_debouncer.clear();
                return_reason = None;
                render_dirty = true;
                spawn_handlers(
                    &self.handlers,
                    PickerEvent::ReloadRequested(Arc::<str>::from(
                        query_text_area.lines().join("\n"),
                    )),
                    true,
                    picker_state.generation,
                    &candidate_sender,
                    &mut handler_tasks,
                    &mut pending_handlers,
                    &mut startup_handlers,
                );
            } else if return_reason.is_some() {
                break;
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
                        &picker_state.candidates,
                        &mut list_state,
                        &mut search_results_keys,
                        &mut search_results_heights,
                    )
                });
            }

            if self.auto_accept
                && startup_handlers == 0
                && picker_state.candidates.len() == 1
                && !many
            {
                return_reason = Some(ReturnReason::Success);
            }
            if static_empty_is_error
                && startup_handlers == 0
                && pending_handlers == 0
                && candidate_receiver.is_empty()
                && picker_state.candidates.len() == 0
            {
                return Ok(RunOutcome::NoChoices);
            }

            if render_dirty {
                let counts_title = if many {
                    format!(
                        "{} items marked for return of {} items matching query of {} items total",
                        picker_state.marked.len(),
                        search_results_keys.len(),
                        picker_state.candidates.len(),
                    )
                } else {
                    format!(
                        "{} items matching query of {} items total",
                        search_results_keys.len(),
                        picker_state.candidates.len(),
                    )
                };
                let mut query_block = Block::bordered().title(counts_title);
                if let Some(pending_title) = pending_title(pending_handlers) {
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
                                    &picker_state.marked,
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
        }

        drop(handler_tasks);
        let values = match return_reason.expect("picker loop must have a return reason") {
            ReturnReason::Cancelled => return Ok(RunOutcome::Cancelled),
            ReturnReason::ReloadRequested => return Ok(RunOutcome::ReloadRequested),
            ReturnReason::Success => picker_state
                .marked
                .into_iter()
                .filter_map(|key| picker_state.candidates.remove(&key))
                .collect(),
        };
        Ok(RunOutcome::Selected(values))
    }
}

pub(super) fn startup_events(default_query: &str) -> Vec<PickerEvent> {
    let mut events = vec![PickerEvent::InitialLoad];
    if !default_query.is_empty() {
        events.push(PickerEvent::QueryChanged(Arc::<str>::from(default_query)));
    }
    events
}

fn spawn_handlers<'a, T>(
    handlers: &[EventHandler<'a, T>],
    event: PickerEvent,
    is_startup: bool,
    generation: u64,
    sender: &mpsc::UnboundedSender<CandidateMessage<T>>,
    tasks: &mut FuturesUnordered<HandlerTask<'a>>,
    pending_handlers: &mut usize,
    startup_handlers: &mut usize,
) {
    let event_kind = match &event {
        PickerEvent::InitialLoad => "initial_load",
        PickerEvent::QueryChanged(_) => "query_changed",
        PickerEvent::QueryCleared => "query_cleared",
        PickerEvent::ReloadRequested(_) => "reload_requested",
    };
    let handler_span = trace_span!("picker_handler", event = event_kind, startup = is_startup,);
    let event = Arc::new(event);
    for handler in handlers {
        let future = (handler.handler)(
            event.clone(),
            CandidateSink {
                sender: sender.clone(),
                generation,
            },
        );
        let handler_span = handler_span.clone();
        tasks.push(Box::pin(async move {
            let result = future.instrument(handler_span).await;
            HandlerCompletion { is_startup, result }
        }));
        *pending_handlers += 1;
        if is_startup {
            *startup_handlers += 1;
        }
    }
}

pub(super) fn process_candidate_message<T>(
    message: CandidateMessage<T>,
    generation: u64,
    picker_state: &mut PickerEventState<T>,
    nucleo: &mut Nucleo<Arc<CompactString>>,
    warned_tab_keys: &mut FxHashSet<CompactString>,
) -> (bool, Vec<CompactString>) {
    if message.generation != generation {
        return (false, Vec::new());
    }

    let mut tab_warning_keys = Vec::new();
    let changed = extended_trace_span!("picker_inject_candidates").in_scope(|| {
        picker_state.candidates.inject(message.choices, |key| {
            nucleo.injector().push(Arc::new(key.clone()), |x, cols| {
                cols[0] = x.as_ref().as_str().into();
            });
            if should_warn_for_tab(warned_tab_keys, key) {
                tab_warning_keys.push(key.clone());
            }
        })
    });
    (changed, tab_warning_keys)
}

pub(super) fn handle_handler_completion(
    completion: HandlerCompletion,
    pending_handlers: &mut usize,
    startup_handlers: &mut usize,
) -> eyre::Result<()> {
    *pending_handlers = pending_handlers.saturating_sub(1);
    if completion.is_startup {
        *startup_handlers = startup_handlers.saturating_sub(1);
    }
    completion.result
}

fn apply_terminal_control(
    control: TerminalControl,
    guard: &mut TerminalGuard,
    terminal: &mut Option<PickerTerminal>,
) -> eyre::Result<()> {
    let mut backend = PickerTerminalBackend { terminal };
    apply_terminal_control_with_backend(control, guard, &mut backend)
}

struct PickerTerminalBackend<'a> {
    terminal: &'a mut Option<PickerTerminal>,
}

impl TerminalBackend for PickerTerminalBackend<'_> {
    fn is_active(&self) -> bool {
        self.terminal.is_some()
    }

    fn suspend(&mut self) -> eyre::Result<()> {
        if self.terminal.take().is_some() {
            restore_terminal()?;
        }
        Ok(())
    }

    fn resume(&mut self) -> eyre::Result<()> {
        if self.terminal.is_none() {
            *self.terminal = Some(enter_terminal()?);
        }
        Ok(())
    }
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

#[derive(Debug, Default)]
struct KeyEffects {
    render_requested: bool,
}

pub(super) struct PickerToast {
    pub(super) record: PickerLogRecord,
    pub(super) expires_at: Instant,
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

fn handle_key(
    key: KeyEvent,
    many: bool,
    list_state: &mut ListState,
    search_results_keys: &[Arc<CompactString>],
    marked_for_return: &mut FxHashSet<CompactString>,
    query_text_area: &mut TextArea<'static>,
    query_changed: &mut bool,
    query_debouncer: &mut QueryDebouncer,
    return_reason: &mut Option<ReturnReason>,
) -> KeyEffects {
    let mut effects = KeyEffects::default();
    match key.code {
        KeyCode::Esc => *return_reason = Some(ReturnReason::Cancelled),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *return_reason = Some(ReturnReason::Cancelled)
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *return_reason = Some(ReturnReason::ReloadRequested)
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
            if (!many || marked_for_return.is_empty())
                && let Some(selected_index) = list_state.selected()
                && let Some(selected_key) = search_results_keys.get(selected_index)
            {
                marked_for_return.insert(selected_key.as_ref().clone());
            }
            *return_reason = Some(ReturnReason::Success);
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
                let next = selected.saturating_add(10);
                if next < search_results_keys.len() {
                    list_state.select(Some(next));
                    effects.render_requested = Some(selected) != list_state.selected();
                }
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
            effects.render_requested = *query_changed;
        }
        _ => {
            *query_changed = query_text_area.input(key);
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

fn rebuild_results<T>(
    nucleo: &Nucleo<Arc<CompactString>>,
    candidate_pool: &ChoicePool<T>,
    list_state: &mut ListState,
    search_results_keys: &mut Vec<Arc<CompactString>>,
    search_results_heights: &mut Vec<usize>,
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

    let selected_index = preserved_selection(selected_key.as_deref(), search_results_keys);
    list_state.select(selected_index);
    debug_assert_eq!(
        search_results_keys
            .iter()
            .filter(|key| candidate_pool.contains_key(key.as_ref()))
            .count(),
        search_results_keys.len()
    );
}

async fn wait_for_tab_warning<'a, T>(
    terminal: &mut Option<PickerTerminal>,
    key: &CompactString,
    guard: &mut TerminalGuard,
    candidate_receiver: &mut mpsc::UnboundedReceiver<CandidateMessage<T>>,
    handler_tasks: &mut FuturesUnordered<HandlerTask<'a>>,
    pending_handlers: &mut usize,
    startup_handlers: &mut usize,
    generation: u64,
    picker_state: &mut PickerEventState<T>,
    nucleo: &mut Nucleo<Arc<CompactString>>,
    warned_tab_keys: &mut FxHashSet<CompactString>,
    pending_tab_warnings: &mut VecDeque<CompactString>,
) -> eyre::Result<()>
where
    T: Send + 'a,
{
    let mut backend = PickerTerminalBackend { terminal };
    if !backend.is_active() {
        return Ok(());
    }
    backend.suspend().map_err(|error| {
        guard.poison(format!("picker tab-warning suspension failed: {error}"));
        error
    })?;
    drop(backend);
    tracing::warn!(
        key = %key,
        "A picker candidate contains a tab character and may render poorly"
    );
    eprintln!("A picker candidate contains a tab character: {key:?}\nPress Enter to continue...");

    let mut enter = tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
    });
    let mut deferred_control = None;
    loop {
        tokio::select! {
            biased;

            result = &mut enter => {
                result??;
                break;
            }
            control = guard.next_control() => {
                let Some(control) = control else {
                    enter.abort();
                    eyre::bail!("terminal coordinator owner channel closed while showing a tab warning");
                };
                match control {
                    TerminalControl::Suspend { .. } | TerminalControl::Resume { .. } => {
                        // The prompt is using the restored terminal outside the alternate
                        // screen. Do not acknowledge a handoff while stdin is still owned by
                        // this prompt; an inner owner must not initialize its terminal until
                        // the prompt has completed.
                        deferred_control = Some(control);
                    }
                    TerminalControl::Poisoned { message } => {
                        enter.abort();
                        eyre::bail!("terminal coordinator poisoned: {message}");
                    }
                }
            }
            Some(message) = candidate_receiver.recv() => {
                let (changed, warning_keys) = process_candidate_message(
                    message,
                    generation,
                    picker_state,
                    nucleo,
                    warned_tab_keys,
                );
                pending_tab_warnings.extend(warning_keys);
                let _ = changed;
            }
            joined = handler_tasks.next(), if *pending_handlers > 0 => {
                match joined {
                    Some(completion) => {
                        handle_handler_completion(
                            completion,
                            pending_handlers,
                            startup_handlers,
                        )?;
                    }
                    None => *pending_handlers = 0,
                }
            }
        }
    }

    match deferred_control {
        Some(control @ TerminalControl::Suspend { .. }) => {
            // The terminal is already restored, so acknowledge suspension without re-entering
            // the backend. The waiting child may now take ownership safely.
            guard.acknowledge(&control)?;
        }
        Some(control @ TerminalControl::Resume { .. }) => {
            let mut backend = PickerTerminalBackend { terminal };
            backend.resume().map_err(|error| {
                guard.poison(format!("picker tab-warning resume failed: {error}"));
                error
            })?;
            guard.acknowledge(&control)?;
        }
        Some(TerminalControl::Poisoned { .. }) => unreachable!("poisoned controls return above"),
        None => {
            let mut backend = PickerTerminalBackend { terminal };
            backend.resume().map_err(|error| {
                guard.poison(format!("picker tab-warning resume failed: {error}"));
                error
            })?;
        }
    }
    Ok(())
}

type PickerTerminal = Terminal<CrosstermBackend<BufWriter<Stderr>>>;

fn enter_terminal() -> eyre::Result<PickerTerminal> {
    enable_raw_mode()?;
    if let Err(error) = execute!(stderr(), EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(error.into());
    }
    let backend = CrosstermBackend::new(BufWriter::new(stderr()));
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = restore_terminal();
            return Err(error.into());
        }
    };
    if let Err(error) = terminal.clear() {
        let _ = restore_terminal();
        return Err(error.into());
    }
    Ok(terminal)
}

fn restore_terminal() -> eyre::Result<()> {
    let raw_mode = disable_raw_mode();
    let alternate_screen = execute!(stderr(), LeaveAlternateScreen);
    match (raw_mode, alternate_screen) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error.into()),
        (Err(raw_mode), Err(alternate_screen)) => Err(eyre::eyre!(
            "failed to restore picker terminal: raw mode: {raw_mode}; alternate screen: {alternate_screen}"
        )),
    }
}
