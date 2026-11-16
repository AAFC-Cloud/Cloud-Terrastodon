use super::candidate_sink::CandidateSink;
use super::event_handler::EventHandler;
use super::picker_candidate_store::PickerCandidateStore;
use super::picker_event::PickerEvent;
use super::picker_ui::PickerUi;
use crate::IntoChoices;
use crate::PickManyResult;
use crate::PickResult;
use crate::PickerLogBufferHandle;
use crate::TerminalCoordinator;
use compact_str::CompactString;
use rustc_hash::FxHashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use teamy_cancellation::CancellationScope;
use tracing::info_span;

pub struct PickerTui<'a, T> {
    pub default_query: String,
    pub header: Option<String>,
    pub auto_accept: bool,
    allow_empty_selection: bool,
    include_focused_on_enter: bool,
    initial_marked: FxHashSet<CompactString>,
    initial_focused: Option<CompactString>,
    cancellation: Option<Pin<Box<dyn Future<Output = ()> + Send + 'a>>>,
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
            allow_empty_selection: false,
            include_focused_on_enter: false,
            initial_marked: FxHashSet::default(),
            initial_focused: None,
            cancellation: None,
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

    pub fn set_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn set_auto_accept(mut self, auto_accept: bool) -> Self {
        self.auto_accept = auto_accept;
        self
    }

    /// Let Enter accept an empty marked set in a multi-select picker.
    ///
    /// This supports editing an existing selection, including removing its last
    /// item. The default still accepts the highlighted row when none are marked;
    /// single-select pickers always retain that behavior. Disable automatic
    /// acceptance separately when the user must review a one-item choice list.
    pub fn set_allow_empty_selection(mut self, allow_empty_selection: bool) -> Self {
        self.allow_empty_selection = allow_empty_selection;
        self
    }

    /// Let plain Enter include the focused row in a multi-selection before
    /// accepting it. Ctrl-Enter applies only the marked set, including an empty
    /// set when allowed. This makes typing a target and pressing Enter an add
    /// operation while retaining an explicit way to apply removals.
    pub fn set_include_focused_on_enter(mut self, include: bool) -> Self {
        self.include_focused_on_enter = include;
        self
    }

    /// Cancel when the supplied notification completes, returning through the
    /// normal terminal restoration and guard-release path.
    ///
    /// The caller must continue polling this picker until completion. A nested
    /// terminal owner is allowed to finish before cancellation releases this
    /// picker's frame; cancellation of that owner's work remains its caller's
    /// responsibility.
    pub fn cancel_on(mut self, notification: impl Future<Output = ()> + Send + 'a) -> Self {
        self.cancellation = Some(Box::pin(notification));
        self
    }

    pub fn set_query(mut self, query: impl Into<String>) -> Self {
        self.default_query = query.into();
        self
    }

    /// Mark choices before a multi-select picker is first rendered.
    ///
    /// Keys use the same identity as [`crate::Choice::key`]. This lets an
    /// editor reopen a picker with its persisted selections without coupling
    /// the picker to the selected value type.
    pub fn set_initial_selected<I, S>(mut self, keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.initial_marked = keys
            .into_iter()
            .map(|key| CompactString::from(key.as_ref()))
            .collect();
        self
    }

    /// Focus one choice when its key first appears in the matched results.
    ///
    /// Unlike marked multi-selection, this controls the highlighted row. The
    /// preference waits for asynchronously loaded choices, then applies once.
    /// Keyboard input cancels an unapplied preference so late results never
    /// override the user's navigation or search.
    pub fn set_initial_focused(mut self, key: impl AsRef<str>) -> Self {
        self.initial_focused = Some(CompactString::from(key.as_ref()));
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

    /// Register event work owned by this picker generation.
    ///
    /// Use the supplied child scope for requests. Reload and shutdown request
    /// cancellation and keep polling admitted handlers until their required
    /// completion finishes; results from retired generations are ignored.
    pub fn add_event_handler<F, Fut>(self, handler: F) -> Self
    where
        F: Fn(CancellationScope, Arc<PickerEvent>, CandidateSink<T>) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<()>> + Send + 'a,
    {
        let mut picker = self;
        picker.handlers.push(EventHandler::new(handler));
        picker
    }

    pub async fn pick_one_events(self, cancellation: &CancellationScope) -> PickResult<T>
    where
        T: Send + 'a,
    {
        self.run(false, false, cancellation)
            .await
            .map(|mut items| items.remove(0))
    }

    pub async fn pick_many_events(self, cancellation: &CancellationScope) -> PickManyResult<T>
    where
        T: Send + 'a,
    {
        self.run(true, false, cancellation).await
    }

    pub async fn pick_one(
        self,
        choices: impl IntoChoices<T>,
        cancellation: &CancellationScope,
    ) -> PickResult<T>
    where
        T: Send + 'a,
    {
        self.with_initial_choices(choices)
            .run(false, true, cancellation)
            .await
            .map(|mut items| items.remove(0))
    }

    pub async fn pick_many(
        self,
        choices: impl IntoChoices<T>,
        cancellation: &CancellationScope,
    ) -> PickManyResult<T>
    where
        T: Send + 'a,
    {
        self.with_initial_choices(choices)
            .run(true, true, cancellation)
            .await
    }

    pub async fn pick_inner(
        self,
        many: bool,
        choices: impl IntoChoices<T>,
        cancellation: &CancellationScope,
    ) -> PickManyResult<T>
    where
        T: Send + 'a,
    {
        self.with_initial_choices(choices)
            .run(many, true, cancellation)
            .await
    }

    pub async fn pick_one_reloadable<F, Fut, C>(
        self,
        cancellation: &CancellationScope,
        choice_supplier: F,
    ) -> PickResult<T>
    where
        T: Send + 'a,
        F: Fn(CancellationScope, bool) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<C>> + Send + 'a,
        C: IntoChoices<T> + 'a,
    {
        self.with_reload_handler(choice_supplier)
            .run(false, false, cancellation)
            .await
            .map(|mut items| items.remove(0))
    }

    pub async fn pick_many_reloadable<F, Fut, C>(
        self,
        cancellation: &CancellationScope,
        choice_supplier: F,
    ) -> PickManyResult<T>
    where
        T: Send + 'a,
        F: Fn(CancellationScope, bool) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<C>> + Send + 'a,
        C: IntoChoices<T> + 'a,
    {
        self.with_reload_handler(choice_supplier)
            .run(true, false, cancellation)
            .await
    }

    fn with_initial_choices(self, choices: impl IntoChoices<T>) -> Self
    where
        T: Send + 'a,
    {
        let choices =
            info_span!("picker_prepare_initial_choices").in_scope(|| choices.into_choices());
        let choices = Arc::new(Mutex::new(Some(choices)));
        self.add_event_handler(move |_child, event, sink| {
            let choices = choices.clone();
            async move {
                if matches!(event.as_ref(), PickerEvent::InitialLoad)
                    && let Some(choices) = choices
                        .lock()
                        .map_err(|_| eyre::eyre!("choices mutex poisoned"))?
                        .take()
                {
                    sink.push(choices)?;
                }
                Ok(())
            }
        })
    }

    fn with_reload_handler<F, Fut, C>(self, choice_supplier: F) -> Self
    where
        T: Send + 'a,
        F: Fn(CancellationScope, bool) -> Fut + Send + 'a,
        Fut: Future<Output = eyre::Result<C>> + Send + 'a,
        C: IntoChoices<T> + 'a,
    {
        let choice_supplier = Arc::new(Mutex::new(choice_supplier));
        self.add_event_handler(move |child, event, sink| {
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
                            .map_err(|_| eyre::eyre!("choice supplier mutex poisoned"))?;
                        (choice_supplier)(child, invalidate)
                    };
                    let choices = future.await?;
                    sink.push(choices)?;
                }
                Ok(())
            }
        })
    }

    async fn run(
        self,
        many: bool,
        static_empty_is_error: bool,
        cancellation: &CancellationScope,
    ) -> PickManyResult<T>
    where
        T: Send + 'a,
    {
        let mut candidates = PickerCandidateStore::new(self.handlers, self.initial_marked);
        let ui = PickerUi {
            default_query: self.default_query,
            header: self.header,
            auto_accept: self.auto_accept,
            allow_empty_selection: self.allow_empty_selection,
            include_focused_on_enter: self.include_focused_on_enter,
            initial_focused: self.initial_focused,
            cancellation: self.cancellation,
            terminal_coordinator: self.terminal_coordinator,
            log_buffer: self.log_buffer,
        };
        let outcome = ui
            .run(&mut candidates, many, static_empty_is_error, cancellation)
            .await;
        candidates.finish(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::PickerTui;
    use compact_str::CompactString;
    use rustc_hash::FxHashSet;

    #[test]
    fn picker_accepts_initial_multi_selection_keys() {
        let picker = PickerTui::<String>::new().set_initial_selected(["first", "third", "first"]);

        assert_eq!(
            picker.initial_marked,
            FxHashSet::from_iter([CompactString::from("first"), CompactString::from("third")])
        );
    }
}
