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
use compact_str::CompactString;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
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
use std::future::Future;
use std::io::BufWriter;
use std::io::Stderr;
use std::io::stderr;
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
    handlers: Vec<EventHandler<'a, T>>,
}

impl<'a, T> Default for PickerTui<'a, T> {
    fn default() -> Self {
        Self {
            default_query: Default::default(),
            header: Default::default(),
            auto_accept: true,
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

    pub fn add_event_handler<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(Arc<PickerEvent>, CandidateSink<T>) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<()>> + Send + 'a,
    {
        self.add_named_event_handler("background work", handler)
    }

    pub fn add_named_event_handler<F, Fut>(mut self, label: impl Into<Arc<str>>, handler: F) -> Self
    where
        F: Fn(Arc<PickerEvent>, CandidateSink<T>) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<()>> + Send + 'a,
    {
        self.handlers.push(EventHandler {
            label: label.into(),
            handler: Box::new(move |event, sink| Box::pin(handler(event, sink))),
        });
        self
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
        self.add_named_event_handler("loading choices", move |event, sink| {
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
        self.add_named_event_handler("loading choices", move |event, sink| {
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
        let mut terminal = enter_terminal()?;
        let original_hook = Arc::new(std::panic::take_hook());
        let panic_hook = original_hook.clone();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            panic_hook(info);
        }));
        let result = self
            .run_loop(&mut terminal, many, static_empty_is_error)
            .await;
        restore_terminal();
        let hook_for_restore = original_hook.clone();
        std::panic::set_hook(Box::new(move |info| hook_for_restore(info)));
        result
    }

    async fn run_loop(
        self,
        terminal: &mut Terminal<CrosstermBackend<BufWriter<Stderr>>>,
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
        let mut pending_work = Vec::<Arc<str>>::new();
        let mut picker_state = PickerEventState::default();

        spawn_handlers(
            &self.handlers,
            PickerEvent::InitialLoad,
            true,
            picker_state.generation,
            &candidate_sender,
            &mut handler_tasks,
            &mut pending_handlers,
            &mut startup_handlers,
            &mut pending_work,
        );
        if !self.default_query.is_empty() {
            spawn_handlers(
                &self.handlers,
                PickerEvent::QueryChanged(Arc::<str>::from(self.default_query.as_str())),
                true,
                picker_state.generation,
                &candidate_sender,
                &mut handler_tasks,
                &mut pending_handlers,
                &mut startup_handlers,
                &mut pending_work,
            );
        }

        let mut nucleo = new_nucleo();
        let mut warned_tab_keys = FxHashSet::<CompactString>::default();
        let mut search_results_keys = Vec::<CompactString>::new();
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

        loop {
            let debounce = query_debouncer
                .deadline()
                .map(|deadline| tokio::time::sleep_until(deadline.into()))
                .unwrap_or_else(|| tokio::time::sleep(Duration::from_secs(86_400)));
            tokio::pin!(debounce);
            tokio::select! {
                // Prefer already-buffered terminal input over continuously-ready background
                // work so navigation keys are handled with the lowest possible queueing delay.
                biased;

                input = event_stream.next() => {
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
                }
                Some(message) = candidate_receiver.recv() => {
                    if message.generation == picker_state.generation {
                        let batch_span = extended_trace_span!("picker_candidate_batch");
                        let (changed, tab_warning_key) = batch_span.in_scope(|| {
                            let mut tab_warning_key = None;
                            let changed = extended_trace_span!("picker_inject_candidates").in_scope(|| {
                                picker_state.candidates.inject(message.choices, |key| {
                                    nucleo.injector().push(key.clone(), |x, cols| {
                                        cols[0] = x.as_str().into();
                                    });
                                    if should_warn_for_tab(&mut warned_tab_keys, key) {
                                        tab_warning_key = Some(key.clone());
                                    }
                                })
                            });
                            (changed, tab_warning_key)
                        });
                        if let Some(key) = tab_warning_key {
                            suspend_for_tab_warning(&key)
                                .instrument(batch_span.clone())
                                .await?;
                            terminal.clear()?;
                        }
                        render_dirty |= changed;
                        query_changed |= changed;
                    }
                }
                joined = handler_tasks.next(), if pending_handlers > 0 => {
                    match joined {
                        Some(completion) => {
                            pending_handlers -= 1;
                            remove_pending_work(&mut pending_work, &completion.label);
                            render_dirty = true;
                            if completion.is_startup {
                                startup_handlers -= 1;
                            }
                            if let Err(error) = completion.result {
                                return Err(error);
                            }
                        }
                        None => {
                            pending_handlers = 0;
                            pending_work.clear();
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
                            &mut pending_work,
                        );
                    }
                }
                _ = ticker.tick() => {}
            }

            if matches!(return_reason, Some(ReturnReason::ReloadRequested)) {
                handler_tasks = FuturesUnordered::new();
                pending_handlers = 0;
                startup_handlers = 0;
                pending_work.clear();
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
                    &mut pending_work,
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
                if let Some(pending_title) = pending_title(&pending_work) {
                    query_block =
                        query_block.title_bottom(Line::from(pending_title).right_aligned());
                }
                query_text_area.set_block(query_block);

                extended_trace_span!("picker_render").in_scope(|| {
                    terminal.draw(|frame| {
                        let area = frame.area();
                        let buf = frame.buffer_mut();
                        let [list_area, searchbox_area] =
                            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)])
                                .areas(area);
                        if search_results_keys.is_empty() {
                            let empty_message = if query_text_area.is_empty() {
                                "No choices yet, try typing to search"
                            } else {
                                "No results"
                            };
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
                    })
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

fn spawn_handlers<'a, T>(
    handlers: &[EventHandler<'a, T>],
    event: PickerEvent,
    is_startup: bool,
    generation: u64,
    sender: &mpsc::UnboundedSender<CandidateMessage<T>>,
    tasks: &mut FuturesUnordered<HandlerTask<'a>>,
    pending_handlers: &mut usize,
    startup_handlers: &mut usize,
    pending_work: &mut Vec<Arc<str>>,
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
        let label = handler.label.clone();
        pending_work.push(label.clone());
        let handler_span = handler_span.clone();
        tasks.push(Box::pin(async move {
            let result = future.instrument(handler_span).await;
            HandlerCompletion {
                label,
                is_startup,
                result,
            }
        }));
        *pending_handlers += 1;
        if is_startup {
            *startup_handlers += 1;
        }
    }
}

fn new_nucleo() -> Nucleo<CompactString> {
    Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1)
}

fn list_block(header: Option<&str>) -> Block<'static> {
    let mut block = Block::bordered();
    if let Some(header) = header {
        block = block.title(header.to_owned());
    }
    block
}

fn pending_title(pending_work: &[Arc<str>]) -> Option<String> {
    if pending_work.is_empty() {
        return None;
    }

    let mut title = String::from("loading: ");
    for (index, label) in pending_work.iter().enumerate() {
        if index > 0 {
            title.push_str(", ");
        }
        title.push_str(label);
    }
    Some(title)
}

fn remove_pending_work(pending_work: &mut Vec<Arc<str>>, label: &Arc<str>) {
    if let Some(index) = pending_work.iter().position(|pending| pending == label) {
        pending_work.remove(index);
    }
}

#[derive(Debug, Default)]
struct KeyEffects {
    render_requested: bool,
}

fn handle_key(
    key: KeyEvent,
    many: bool,
    list_state: &mut ListState,
    search_results_keys: &[CompactString],
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
                if !marked_for_return.insert(selected_item.clone()) {
                    marked_for_return.remove(selected_item);
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
                marked_for_return.insert(selected_key.clone());
            }
            *return_reason = Some(ReturnReason::Success);
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            marked_for_return.extend(search_results_keys.iter().cloned());
            effects.render_requested = true;
        }
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *marked_for_return = search_results_keys
                .iter()
                .filter(|key| !marked_for_return.contains(*key))
                .cloned()
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

fn row_style(marked: bool, cursor: bool) -> Style {
    match (marked, cursor) {
        (true, true) => Style::new().bg(Color::Magenta),
        (true, false) => Style::new().bg(Color::DarkGray),
        (false, true) => Style::new().bg(Color::Blue),
        (false, false) => Style::default(),
    }
}

fn render_picker_list(
    buf: &mut Buffer,
    area: Rect,
    header: Option<&str>,
    many: bool,
    search_results_keys: &[CompactString],
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
        let marked = many && marked_for_return.contains(&search_results_keys[index]);
        let item_style = row_style(marked, selected_index == Some(index));
        let dot_style = item_style.fg(Color::Red);
        for row_y in row_area.top()..row_area.bottom() {
            for x in row_area.left()..row_area.right() {
                buf[(x, row_y)].reset();
                buf[(x, row_y)].set_style(item_style);
            }
        }

        for (line_index, line) in search_results_keys[index].lines().enumerate() {
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
    nucleo: &Nucleo<CompactString>,
    candidate_pool: &ChoicePool<T>,
    list_state: &mut ListState,
    search_results_keys: &mut Vec<CompactString>,
    search_results_heights: &mut Vec<usize>,
) {
    let selected_key = list_state
        .selected()
        .and_then(|index| search_results_keys.get(index))
        .cloned();
    search_results_keys.clear();
    search_results_heights.clear();
    for item in nucleo.snapshot().matched_items(..) {
        let key = item.data.clone();
        let height = key.lines().count().max(1);
        search_results_keys.push(key);
        search_results_heights.push(height);
    }

    let selected_index = preserved_selection(selected_key.as_ref(), search_results_keys);
    list_state.select(selected_index);
    debug_assert_eq!(
        search_results_keys
            .iter()
            .filter(|key| candidate_pool.contains_key(*key))
            .count(),
        search_results_keys.len()
    );
}

async fn suspend_for_tab_warning(key: &CompactString) -> eyre::Result<()> {
    restore_terminal();
    tracing::warn!(
        key = %key,
        "A picker candidate contains a tab character and may render poorly"
    );
    eprintln!("A picker candidate contains a tab character: {key:?}\nPress Enter to continue...");
    tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
    })
    .await??;
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;
    Ok(())
}

type PickerTerminal = Terminal<CrosstermBackend<BufWriter<Stderr>>>;

fn enter_terminal() -> eyre::Result<PickerTerminal> {
    enable_raw_mode()?;
    execute!(stderr(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(BufWriter::new(stderr()));
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal() {
    if let Err(error) = disable_raw_mode() {
        eprintln!("Failed to disable raw mode: {error}");
    }
    if let Err(error) = execute!(stderr(), LeaveAlternateScreen) {
        eprintln!("Failed to leave alternate screen: {error}");
    }
}
