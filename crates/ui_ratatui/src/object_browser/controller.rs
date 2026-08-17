use crate::object_explorer::ArbitraryInvocationStart;
use crate::object_explorer::ArenaBrowseSession;
use crate::object_explorer::ArenaQueryContext;
use crate::object_explorer::ArenaQueryContextError;
use crate::object_explorer::Breadcrumb;
use crate::object_explorer::BreadcrumbContextSnapshot;
use crate::object_explorer::Breadcrumbs;
use crate::object_explorer::BuilderTransition;
use crate::object_explorer::CardAddress;
use crate::object_explorer::CardNavigation;
use crate::object_explorer::CardWindow;
use crate::object_explorer::CardWindowBudget;
use crate::object_explorer::ExplorerHandle;
use crate::object_explorer::ExplorerHandleError;
use crate::object_explorer::FieldBindingPacket;
use crate::object_explorer::FieldCandidateAction;
use crate::object_explorer::FieldCandidateActions;
use crate::object_explorer::InvocationEvent;
use crate::object_explorer::InvocationMode;
use crate::object_explorer::InvocationStart;
use crate::object_explorer::OpenTabs;
use crate::object_explorer::OwnedValuePacket;
use crate::object_explorer::ProductionBatch;
use crate::object_explorer::ProductionStrategy;
use crate::object_explorer::QueryProgressState;
use crate::object_explorer::RootSnapshot;
use crate::object_explorer::SlotId;
use crate::object_explorer::Tab;
use crate::object_explorer::TabHeaderSnapshot;
use crate::object_explorer::TabUiState;
use crate::object_explorer::TabUiStates;
use crate::object_explorer::TabUpdate;
use crate::object_explorer::ValueAddress;
use crate::object_explorer::ValueCandidateWindow;
use crate::object_explorer::ValueCandidateWindowBudget;
use cloud_terrastodon_registry::Function;
use facet::Facet;
use facet::Shape;
use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ObjectBrowserControllerError {
    Engine(String),
    NoActiveTab,
}

impl fmt::Display for ObjectBrowserControllerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Engine(message) => formatter.write_str(message),
            Self::NoActiveTab => formatter.write_str("the object browser has no active tab"),
        }
    }
}

impl Error for ObjectBrowserControllerError {}

impl From<ArenaQueryContextError> for ObjectBrowserControllerError {
    fn from(value: ArenaQueryContextError) -> Self {
        Self::Engine(value.to_string())
    }
}

impl From<ExplorerHandleError> for ObjectBrowserControllerError {
    fn from(value: ExplorerHandleError) -> Self {
        Self::Engine(value.to_string())
    }
}

/// Ratatui-facing state over the single-owner ExplorerEngine.
///
/// The controller retains only command capabilities, stable identities, and
/// bounded snapshots. Arena values remain exclusively inside the engine.
pub(crate) struct ObjectBrowserController {
    context: ArenaQueryContext,
    handle: ExplorerHandle,
    browse: ArenaBrowseSession,
    open_tabs: OpenTabs,
    tab_states: TabUiStates,
    active_tab_header: TabHeaderSnapshot,
    window: Option<CardWindow>,
    scanning: bool,
    reanchor_on_next_window: bool,
}

impl ObjectBrowserController {
    pub(crate) async fn bootstrap(
        context: ArenaQueryContext,
    ) -> Result<Self, ObjectBrowserControllerError> {
        let handle = context.engine_handle();
        let breadcrumbs = Breadcrumbs::default();
        let initial_tab = Tab::new("unnamed", breadcrumbs.clone());
        let tab = handle
            .insert_ready(OwnedValuePacket::new(initial_tab.clone()))
            .await?;
        let browse = context.open_browse(breadcrumbs).await?;
        let mut open_tabs = OpenTabs::default();
        open_tabs.open(tab);
        let mut tab_states = TabUiStates::default();
        let tab_address = CardAddress::Value(ValueAddress::root(tab));
        let state = tab_states.for_tab_mut(tab);
        state.select(tab_address.clone());
        state.set_viewport_anchor(tab_address);

        Ok(Self {
            context,
            handle,
            browse,
            open_tabs,
            tab_states,
            active_tab_header: TabHeaderSnapshot::observe(tab, &initial_tab, 12),
            window: None,
            scanning: false,
            reanchor_on_next_window: false,
        })
    }

    pub(crate) fn active_tab(&self) -> Option<SlotId> {
        self.open_tabs.active()
    }

    pub(crate) const fn active_tab_header(&self) -> &TabHeaderSnapshot {
        &self.active_tab_header
    }

    pub(crate) fn active_tab_ordinal(&self) -> Option<(usize, usize)> {
        self.open_tabs.active_ordinal()
    }

    pub(crate) fn active_state(&self) -> Option<&TabUiState> {
        self.active_tab()
            .and_then(|tab| self.tab_states.for_tab(tab))
    }

    pub(crate) fn active_state_mut(&mut self) -> Option<&mut TabUiState> {
        let tab = self.active_tab()?;
        Some(self.tab_states.for_tab_mut(tab))
    }

    pub(crate) fn focus_row(
        &mut self,
        row: Option<crate::object_explorer::CardRowKey>,
    ) -> Result<(), ObjectBrowserControllerError> {
        self.active_state_mut()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?
            .focus_row(row);
        Ok(())
    }

    pub(crate) fn window(&self) -> Option<&CardWindow> {
        self.window.as_ref()
    }

    pub(crate) const fn is_scanning(&self) -> bool {
        self.scanning
    }

    pub(crate) fn select(
        &mut self,
        selection: CardAddress,
    ) -> Result<(), ObjectBrowserControllerError> {
        let state = self
            .active_state_mut()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        state.select(selection);
        Ok(())
    }

    pub(crate) fn focus(
        &mut self,
        selection: CardAddress,
    ) -> Result<(), ObjectBrowserControllerError> {
        let state = self
            .active_state_mut()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        state.select(selection.clone());
        state.set_viewport_anchor(selection);
        Ok(())
    }

    pub(crate) async fn navigate_card(
        &mut self,
        direction: CardNavigation,
        max_work: usize,
    ) -> Result<QueryProgressState<ValueAddress>, ObjectBrowserControllerError> {
        let from = match self
            .active_state()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?
            .selection()
        {
            CardAddress::Value(address) => address.clone(),
            CardAddress::NewSlot => return Ok(QueryProgressState::Complete),
        };
        let progress = self
            .browse
            .navigate(from.clone(), direction, max_work)
            .await?;
        if let Some(state) = self.active_state_mut() {
            state.set_query_total(progress.total());
        }
        let result = progress.into_state();
        if let QueryProgressState::Ready(target) = &result {
            let target_card = CardAddress::Value(target.clone());
            let target_is_visible = self.window.as_ref().is_some_and(|window| {
                window
                    .cards()
                    .iter()
                    .any(|card| card.address() == &target_card)
            });
            let state = self
                .active_state_mut()
                .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
            state.select(target_card.clone());
            if !target_is_visible {
                let anchor = match direction {
                    CardNavigation::Next => CardAddress::Value(from),
                    CardNavigation::Previous => target_card,
                };
                state.set_viewport_anchor(anchor);
            }
        }
        Ok(result)
    }

    pub(crate) async fn refresh_window(
        &mut self,
        max_work: usize,
        max_cards: NonZeroUsize,
        max_relationship_rows: usize,
    ) -> Result<(), ObjectBrowserControllerError> {
        let anchor = self
            .active_state()
            .and_then(|state| match state.viewport_anchor() {
                CardAddress::Value(address) => Some(address.clone()),
                CardAddress::NewSlot => None,
            });
        let progress = self
            .browse
            .fill_card_window(
                anchor,
                CardWindowBudget::new(max_work, max_cards, max_relationship_rows),
            )
            .await?;
        if let Some(state) = self.active_state_mut() {
            state.set_query_total(progress.total());
        }
        match progress.into_state() {
            QueryProgressState::Ready(window) => {
                if self.reanchor_on_next_window {
                    if let Some(first) = window.cards().first() {
                        let address = first.address().clone();
                        if let Some(state) = self.active_state_mut() {
                            state.select(address.clone());
                            state.set_viewport_anchor(address);
                            state.focus_row(None);
                        }
                    }
                    self.reanchor_on_next_window = false;
                }
                self.window = Some(window);
                self.scanning = false;
            }
            QueryProgressState::Pending => self.scanning = true,
            QueryProgressState::Complete => {
                self.window = None;
                self.scanning = false;
                if self.reanchor_on_next_window {
                    if let Some(state) = self.active_state_mut() {
                        state.select(CardAddress::NewSlot);
                        state.set_viewport_anchor(CardAddress::NewSlot);
                        state.focus_row(None);
                    }
                    self.reanchor_on_next_window = false;
                }
            }
            QueryProgressState::Cancelled => self.scanning = false,
            QueryProgressState::Stale => {
                self.window = None;
                self.scanning = false;
            }
        }
        Ok(())
    }

    pub(crate) async fn insert_owned<T>(
        &self,
        value: T,
    ) -> Result<SlotId, ObjectBrowserControllerError>
    where
        T: Facet<'static> + Send + 'static,
    {
        Ok(self
            .handle
            .insert_ready(OwnedValuePacket::new(value))
            .await?)
    }

    pub(crate) async fn create_builder(
        &self,
        shape: &'static Shape,
    ) -> Result<(SlotId, BuilderTransition), ObjectBrowserControllerError> {
        Ok(self.handle.create_builder(shape).await?)
    }

    pub(crate) async fn reserve_builder(&self) -> Result<SlotId, ObjectBrowserControllerError> {
        Ok(self.handle.reserve_builder().await?)
    }

    pub(crate) async fn set_builder_shape(
        &self,
        slot: SlotId,
        shape: &'static Shape,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError> {
        Ok(self.handle.set_builder_shape(slot, shape).await?)
    }

    pub(crate) async fn set_field_default(
        &self,
        destination: SlotId,
        field: usize,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError> {
        Ok(self
            .handle
            .set_builder_field(destination, field, FieldBindingPacket::Default)
            .await?)
    }

    pub(crate) async fn set_field_inline<T>(
        &self,
        destination: SlotId,
        field: usize,
        value: T,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError>
    where
        T: Facet<'static> + Send + 'static,
    {
        Ok(self
            .handle
            .set_builder_field(
                destination,
                field,
                FieldBindingPacket::InlineOwned(OwnedValuePacket::new(value)),
            )
            .await?)
    }

    pub(crate) async fn set_field_candidate(
        &self,
        destination: SlotId,
        field: usize,
        source: ValueAddress,
        action: FieldCandidateAction,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError> {
        let binding = match action {
            FieldCandidateAction::Borrow => FieldBindingPacket::BorrowFrom(source),
            FieldCandidateAction::Move => FieldBindingPacket::move_from_address(source)
                .map_err(|error| ObjectBrowserControllerError::Engine(error.to_string()))?,
            FieldCandidateAction::Clone => FieldBindingPacket::CloneFrom(source),
        };
        Ok(self
            .handle
            .set_builder_field(destination, field, binding)
            .await?)
    }

    pub(crate) async fn unset_field(
        &self,
        destination: SlotId,
        field: usize,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError> {
        Ok(self.handle.unset_builder_field(destination, field).await?)
    }

    pub(crate) async fn select_variant(
        &self,
        slot: SlotId,
        variant: usize,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError> {
        Ok(self.handle.select_builder_variant(slot, variant).await?)
    }

    pub(crate) async fn set_scalar<T>(
        &self,
        slot: SlotId,
        value: T,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError>
    where
        T: Facet<'static> + Send + 'static,
    {
        Ok(self
            .handle
            .set_builder_scalar(slot, OwnedValuePacket::new(value))
            .await?)
    }

    pub(crate) async fn set_scalar_text(
        &self,
        slot: SlotId,
        text: String,
    ) -> Result<BuilderTransition, ObjectBrowserControllerError> {
        Ok(self.handle.set_builder_scalar_text(slot, text).await?)
    }

    pub(crate) async fn delete(&self, slot: SlotId) -> Result<(), ObjectBrowserControllerError> {
        Ok(self.handle.delete(slot).await?)
    }

    pub(crate) async fn invoke(
        &self,
        input: SlotId,
        function: &'static Function,
        mode: InvocationMode,
    ) -> Result<InvocationStart, ObjectBrowserControllerError> {
        let input_thing = cloud_terrastodon_registry::known_thing_for_shape(function.input_shape)
            .ok_or_else(|| {
            ObjectBrowserControllerError::Engine(format!(
                "{} has no registered runtime Thing",
                cloud_terrastodon_registry::describe_shape(function.input_shape)
            ))
        })?;
        Ok(self
            .handle
            .invoke(input, input_thing, function, mode)
            .await?)
    }

    pub(crate) async fn invoke_arbitrary(
        &self,
        request: SlotId,
        request_function: &'static Function,
        constructor: &'static Function,
        bytes: Vec<u8>,
    ) -> Result<ArbitraryInvocationStart, ObjectBrowserControllerError> {
        Ok(self
            .handle
            .invoke_arbitrary(request, request_function, constructor, bytes)
            .await?)
    }

    pub(crate) async fn poll_invocations(
        &self,
    ) -> Result<Vec<InvocationEvent>, ObjectBrowserControllerError> {
        Ok(self.handle.poll_invocations().await?)
    }

    pub(crate) async fn start_production(
        &self,
        destination: SlotId,
        field: usize,
        function: &'static Function,
        strategy: ProductionStrategy,
        max_work: usize,
    ) -> Result<ProductionBatch, ObjectBrowserControllerError> {
        Ok(self
            .handle
            .start_production(destination, field, function, strategy, max_work)
            .await?)
    }

    pub(crate) async fn advance_productions(
        &self,
        max_work: usize,
    ) -> Result<ProductionBatch, ObjectBrowserControllerError> {
        Ok(self.handle.advance_productions(max_work).await?)
    }

    pub(crate) async fn inspect_field_candidate(
        &self,
        destination: SlotId,
        field: usize,
        source: ValueAddress,
    ) -> Result<FieldCandidateActions, ObjectBrowserControllerError> {
        Ok(self
            .handle
            .inspect_field_candidate(destination, field, source)
            .await?)
    }

    pub(crate) async fn inspect_root(
        &self,
        slot: SlotId,
        max_relationship_rows: usize,
    ) -> Result<RootSnapshot, ObjectBrowserControllerError> {
        Ok(self
            .handle
            .inspect_root(slot, max_relationship_rows)
            .await?)
    }

    pub(crate) async fn begin_value_candidates(
        &self,
        target_shape: &'static Shape,
    ) -> Result<(), ObjectBrowserControllerError> {
        Ok(self.browse.set_candidate_shape(target_shape).await?)
    }

    pub(crate) async fn fill_value_candidates(
        &self,
        anchor: Option<ValueAddress>,
        max_work: usize,
        max_candidates: NonZeroUsize,
    ) -> Result<
        crate::object_explorer::QueryProgress<ValueCandidateWindow>,
        ObjectBrowserControllerError,
    > {
        Ok(self
            .browse
            .fill_value_candidates(
                anchor,
                ValueCandidateWindowBudget::new(max_work, max_candidates),
            )
            .await?)
    }

    pub(crate) async fn end_value_candidates(&self) -> Result<(), ObjectBrowserControllerError> {
        Ok(self.browse.clear_value_candidates().await?)
    }

    pub(crate) async fn update_active_tab(
        &mut self,
        update: TabUpdate,
    ) -> Result<(), ObjectBrowserControllerError> {
        let tab_slot = self
            .active_tab()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        let query_changed = !matches!(&update, TabUpdate::Rename(_));
        let tab = self.handle.update_tab(tab_slot, update).await?;
        if query_changed {
            self.browse.set_query(tab.breadcrumbs().clone()).await?;
            self.window = None;
            self.scanning = false;
            self.reanchor_on_next_window = true;
            if let Some(state) = self.active_state_mut() {
                state.select(CardAddress::NewSlot);
                state.set_viewport_anchor(CardAddress::NewSlot);
                state.focus_row(None);
            }
        }
        self.active_tab_header = TabHeaderSnapshot::observe(tab_slot, &tab, 12);
        Ok(())
    }

    pub(crate) async fn inspect_breadcrumb_context(
        &self,
        prefix_len: usize,
        max_work: usize,
        max_choices: usize,
    ) -> Result<BreadcrumbContextSnapshot, ObjectBrowserControllerError> {
        let tab_slot = self
            .active_tab()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        let tab = self.handle.inspect_tab(tab_slot).await?;
        if prefix_len > tab.breadcrumbs().operations().len() {
            return Err(ObjectBrowserControllerError::Engine(format!(
                "breadcrumb prefix {prefix_len} exceeds the active query length {}",
                tab.breadcrumbs().operations().len()
            )));
        }
        let prefix = Breadcrumbs::new(tab.breadcrumbs().operations()[..prefix_len].to_vec());
        Ok(self
            .handle
            .inspect_breadcrumb_context(prefix, max_work, max_choices)
            .await?)
    }

    pub(crate) async fn inspect_breadcrumb_values(
        &self,
        prefix_len: usize,
        field_shape: String,
        field_name: String,
        max_work: usize,
        max_choices: usize,
    ) -> Result<crate::object_explorer::BreadcrumbContextValueSnapshot, ObjectBrowserControllerError>
    {
        let tab_slot = self
            .active_tab()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        let tab = self.handle.inspect_tab(tab_slot).await?;
        if prefix_len > tab.breadcrumbs().operations().len() {
            return Err(ObjectBrowserControllerError::Engine(format!(
                "breadcrumb prefix {prefix_len} exceeds the active query length {}",
                tab.breadcrumbs().operations().len()
            )));
        }
        let prefix = Breadcrumbs::new(tab.breadcrumbs().operations()[..prefix_len].to_vec());
        Ok(self
            .handle
            .inspect_breadcrumb_values(prefix, field_shape, field_name, max_work, max_choices)
            .await?)
    }

    pub(crate) async fn active_breadcrumbs(
        &self,
    ) -> Result<Breadcrumbs, ObjectBrowserControllerError> {
        let tab_slot = self
            .active_tab()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        Ok(self
            .handle
            .inspect_tab(tab_slot)
            .await?
            .breadcrumbs()
            .clone())
    }

    pub(crate) async fn create_tab(
        &mut self,
        name: impl Into<String>,
    ) -> Result<SlotId, ObjectBrowserControllerError> {
        self.create_tab_with_query(name, Breadcrumbs::default(), None)
            .await
    }

    pub(crate) async fn create_output_tab(
        &mut self,
        name: impl Into<String>,
        output: SlotId,
    ) -> Result<SlotId, ObjectBrowserControllerError> {
        let breadcrumbs = Breadcrumbs::new(vec![Breadcrumb::projection(output.get(), Vec::new())]);
        self.create_tab_with_query(
            name,
            breadcrumbs,
            Some(CardAddress::Value(ValueAddress::root(output))),
        )
        .await
    }

    pub(crate) async fn create_projection_tab(
        &mut self,
        name: impl Into<String>,
        address: ValueAddress,
    ) -> Result<SlotId, ObjectBrowserControllerError> {
        let breadcrumbs = Breadcrumbs::new(vec![Breadcrumb::projection(
            address.root_id().get(),
            address.path().segments().to_vec(),
        )]);
        self.create_tab_with_query(name, breadcrumbs, Some(CardAddress::Value(address)))
            .await
    }

    async fn create_tab_with_query(
        &mut self,
        name: impl Into<String>,
        breadcrumbs: Breadcrumbs,
        selection: Option<CardAddress>,
    ) -> Result<SlotId, ObjectBrowserControllerError> {
        let tab_value = Tab::new(name, breadcrumbs);
        let tab = self
            .handle
            .insert_ready(OwnedValuePacket::new(tab_value.clone()))
            .await?;
        self.open_tabs.open(tab);
        let address = selection.unwrap_or_else(|| CardAddress::Value(ValueAddress::root(tab)));
        let state = self.tab_states.for_tab_mut(tab);
        state.select(address.clone());
        state.set_viewport_anchor(address);
        state.focus_row(None);
        self.browse
            .set_query(tab_value.breadcrumbs().clone())
            .await?;
        self.active_tab_header = TabHeaderSnapshot::observe(tab, &tab_value, 12);
        self.window = None;
        self.scanning = false;
        self.reanchor_on_next_window = false;
        Ok(tab)
    }

    pub(crate) async fn switch_tab_previous(
        &mut self,
    ) -> Result<bool, ObjectBrowserControllerError> {
        let Some(tab) = self.open_tabs.previous() else {
            return Ok(false);
        };
        self.activate_tab(tab).await?;
        Ok(true)
    }

    pub(crate) async fn switch_tab_next(&mut self) -> Result<SlotId, ObjectBrowserControllerError> {
        if let Some(tab) = self.open_tabs.next() {
            self.activate_tab(tab).await?;
            Ok(tab)
        } else {
            self.create_tab("unnamed").await
        }
    }

    pub(crate) async fn close_active_tab(
        &mut self,
    ) -> Result<SlotId, ObjectBrowserControllerError> {
        let closing = self
            .active_tab()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        self.handle.delete(closing).await?;
        self.open_tabs.close(closing);
        self.tab_states.discard(closing);
        if let Some(tab) = self.open_tabs.active() {
            self.activate_tab(tab).await?;
            Ok(tab)
        } else {
            self.create_tab("unnamed").await
        }
    }

    async fn activate_tab(&mut self, tab_slot: SlotId) -> Result<(), ObjectBrowserControllerError> {
        let tab = self.handle.inspect_tab(tab_slot).await?;
        if !self.open_tabs.activate(tab_slot) {
            self.open_tabs.open(tab_slot);
        }
        if self.tab_states.for_tab(tab_slot).is_none() {
            let address = CardAddress::Value(ValueAddress::root(tab_slot));
            let state = self.tab_states.for_tab_mut(tab_slot);
            state.select(address.clone());
            state.set_viewport_anchor(address);
        }
        self.browse.set_query(tab.breadcrumbs().clone()).await?;
        self.active_tab_header = TabHeaderSnapshot::observe(tab_slot, &tab, 12);
        self.window = None;
        self.scanning = false;
        self.reanchor_on_next_window = false;
        Ok(())
    }

    pub(crate) async fn close(self) -> Result<(), ObjectBrowserControllerError> {
        self.browse.close().await?;
        drop(self.handle);
        drop(self.context);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::ExplorerEngine;
    use crate::object_explorer::ProduceJsonRequest;
    use crate::object_explorer::QueryProgressState;
    use crate::object_explorer::ValueOwner;
    use crate::object_explorer::ValuePathSegment;
    use cloud_terrastodon_registry::RuntimeValue;

    #[derive(Facet)]
    #[repr(C)]
    struct ControllerBuilderThing {
        name: String,
    }

    #[test]
    fn controller_cannot_own_runtime_values_by_construction() {
        assert!(
            std::mem::size_of::<ObjectBrowserController>()
                < std::mem::size_of::<RuntimeValue>() + 512,
            "the controller stores handles and bounded UI state, not an object pool"
        );
    }

    #[tokio::test]
    async fn breadcrumb_context_uses_only_the_prefix_before_the_edited_operation() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut controller = ObjectBrowserController::bootstrap(context).await.unwrap();
            controller
                .insert_owned(ControllerBuilderThing {
                    name: "Ada".to_owned(),
                })
                .await
                .unwrap();
            let owner_shape =
                cloud_terrastodon_registry::describe_shape(ControllerBuilderThing::SHAPE);
            controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec![owner_shape.clone()],
                }))
                .await
                .unwrap();
            controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec!["String".to_owned()],
                }))
                .await
                .unwrap();

            let at_second_breadcrumb = controller
                .inspect_breadcrumb_context(1, 64, 64)
                .await
                .unwrap();
            assert_eq!(at_second_breadcrumb.shapes(), [owner_shape]);
            assert!(at_second_breadcrumb.fields().iter().any(|field| {
                field.selection().owner_shape() == "ControllerBuilderThing"
                    && field.selection().field_name() == "name"
            }));

            let after_second_breadcrumb = controller
                .inspect_breadcrumb_context(2, 64, 64)
                .await
                .unwrap();
            assert!(after_second_breadcrumb.shapes().is_empty());
            controller.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn controller_bootstrap_and_refresh_use_one_tab_root_and_bounded_cards() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut controller = ObjectBrowserController::bootstrap(context)
                .await
                .expect("controller bootstraps");
            let tab = controller.active_tab().expect("initial tab");
            controller
                .refresh_window(16, NonZeroUsize::new(5).unwrap(), 3)
                .await
                .expect("bounded first frame");
            let window = controller.window().expect("first frame is ready");
            assert!(window.cards().len() <= 5);
            assert_eq!(
                window.cards()[0].address(),
                &CardAddress::Value(ValueAddress::root(tab))
            );
            assert!(
                window
                    .cards()
                    .iter()
                    .skip(1)
                    .all(|card| card.owned_slot().is_none())
            );
            controller.close().await.expect("controller closes");
            tab
        };

        let (engine, tab) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.arena().allocated_slot_count(), 1);
        assert!(engine.arena().ready_value(tab).is_some());
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn controller_navigation_scrolls_only_after_selection_reaches_window_edge() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut controller = ObjectBrowserController::bootstrap(context)
                .await
                .expect("controller bootstraps");
            let tab = controller.active_tab().unwrap();
            let tab_address = CardAddress::Value(ValueAddress::root(tab));
            controller
                .refresh_window(16, NonZeroUsize::new(3).unwrap(), 3)
                .await
                .unwrap();
            assert_eq!(
                controller.active_state().unwrap().viewport_anchor(),
                &tab_address
            );

            let name = match controller
                .navigate_card(CardNavigation::Next, 8)
                .await
                .unwrap()
            {
                QueryProgressState::Ready(address) => address,
                state => panic!("expected name address, got {state:?}"),
            };
            assert_eq!(
                controller.active_state().unwrap().viewport_anchor(),
                &tab_address,
                "moving inside the window must not scroll"
            );
            let breadcrumbs = match controller
                .navigate_card(CardNavigation::Next, 8)
                .await
                .unwrap()
            {
                QueryProgressState::Ready(address) => address,
                state => panic!("expected breadcrumbs address, got {state:?}"),
            };
            assert_ne!(name, breadcrumbs);
            assert_eq!(
                controller.active_state().unwrap().viewport_anchor(),
                &tab_address,
                "reaching the last visible card still does not scroll"
            );

            let operations = match controller
                .navigate_card(CardNavigation::Next, 8)
                .await
                .unwrap()
            {
                QueryProgressState::Ready(address) => address,
                state => panic!("expected operations address, got {state:?}"),
            };
            assert_eq!(
                controller.active_state().unwrap().viewport_anchor(),
                &CardAddress::Value(breadcrumbs.clone()),
                "crossing the edge advances the logical viewport anchor once"
            );
            controller
                .refresh_window(16, NonZeroUsize::new(3).unwrap(), 3)
                .await
                .unwrap();
            let tail_window = controller.window().unwrap();
            assert_eq!(tail_window.cards().len(), 3);
            assert_eq!(
                tail_window.cards()[0].address(),
                &CardAddress::Value(name),
                "a completed tail window backfills instead of collapsing"
            );
            assert_eq!(
                controller.active_state().unwrap().selection(),
                &CardAddress::Value(operations)
            );
            controller.close().await.unwrap();
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn controller_candidate_window_keeps_generic_sequence_owner_context() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let controller = ObjectBrowserController::bootstrap(context)
                .await
                .expect("controller bootstraps");
            let list = controller
                .insert_owned((0_usize..1_000_000).collect::<Vec<_>>())
                .await
                .expect("list enters the engine");
            controller
                .begin_value_candidates(usize::SHAPE)
                .await
                .expect("picker scan opens");
            let progress = controller
                .fill_value_candidates(None, 16, NonZeroUsize::new(8).unwrap())
                .await
                .expect("candidate frame resolves");
            assert!(progress.work_spent() <= 16);
            let window = match progress.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected candidate window, got {state:?}"),
            };
            assert_eq!(window.candidates().len(), 8);
            assert!(window.candidates().iter().all(|candidate| matches!(
                candidate.owner(),
                ValueOwner::SequenceElement { owner, .. }
                    if owner == &ValueAddress::root(list)
            )));
            controller.end_value_candidates().await.unwrap();
            controller.close().await.unwrap();
            list
        };

        let (engine, list) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(list).is_some());
        assert_eq!(engine.arena().allocated_slot_count(), 2);
    }

    #[tokio::test]
    async fn controller_reads_building_roots_as_bounded_snapshots() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let controller = ObjectBrowserController::bootstrap(context)
                .await
                .expect("controller bootstraps");
            let (slot, transition) = controller
                .create_builder(ControllerBuilderThing::SHAPE)
                .await
                .expect("builder is created through the engine");
            assert_eq!(transition, BuilderTransition::Building);
            let snapshot = controller
                .inspect_root(slot, 1)
                .await
                .expect("building root can be presented");
            assert_eq!(
                snapshot.lifecycle(),
                &crate::object_explorer::RootLifecycleSnapshot::Building
            );
            assert_eq!(snapshot.builder().unwrap().fields().len(), 1);
            controller.close().await.unwrap();
            slot
        };

        let (engine, slot) = tokio::join!(engine.run(inbox), client);
        assert!(engine.builders().builder(slot).is_some());
        assert!(engine.arena().ready_value(slot).is_none());
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn building_roots_remain_in_the_browse_window_after_focus_moves() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut controller = ObjectBrowserController::bootstrap(context)
                .await
                .expect("controller bootstraps");
            let (building, transition) = controller
                .create_builder(ControllerBuilderThing::SHAPE)
                .await
                .expect("builder is created");
            assert_eq!(transition, BuilderTransition::Building);
            let later = controller
                .insert_owned(String::from("later root"))
                .await
                .expect("a later Ready root is inserted");
            controller
                .focus(CardAddress::Value(ValueAddress::root(building)))
                .unwrap();
            controller
                .refresh_window(64, NonZeroUsize::new(16).unwrap(), 3)
                .await
                .expect("object-pool window refreshes");

            assert!(
                controller.window().unwrap().cards().iter().any(|card| {
                    card.address() == &CardAddress::Value(ValueAddress::root(building))
                }),
                "a Building root is an object-pool card even though it is not queryable data yet"
            );
            let next = controller
                .navigate_card(CardNavigation::Next, 64)
                .await
                .unwrap();
            assert_eq!(next, QueryProgressState::Ready(ValueAddress::root(later)));
            controller.close().await.unwrap();
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn controller_builds_produce_json_from_generic_tab_breadcrumbs_projection() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let controller = ObjectBrowserController::bootstrap(context)
                .await
                .expect("controller bootstraps");
            let tab = controller.active_tab().unwrap();
            let source =
                ValueAddress::root(tab).child(ValuePathSegment::Field("breadcrumbs".to_owned()));
            let (request, transition) = controller
                .create_builder(ProduceJsonRequest::SHAPE)
                .await
                .expect("request builder is created");
            assert_eq!(transition, BuilderTransition::Building);

            controller
                .begin_value_candidates(Breadcrumbs::SHAPE)
                .await
                .unwrap();
            let candidates = controller
                .fill_value_candidates(None, 16, NonZeroUsize::new(4).unwrap())
                .await
                .unwrap();
            let candidates = match candidates.into_state() {
                QueryProgressState::Ready(window) => window,
                state => panic!("expected Breadcrumbs candidates, got {state:?}"),
            };
            let candidate = candidates
                .candidates()
                .iter()
                .find(|candidate| candidate.address() == &source)
                .expect("Tab.breadcrumbs is an ordinary projected candidate");
            assert!(candidate.display_label().contains("field breadcrumbs"));
            assert!(candidate.display_label().contains("(Tab)"));

            let actions = controller
                .inspect_field_candidate(request, 0, source.clone())
                .await
                .unwrap();
            assert_eq!(
                actions
                    .consequences()
                    .iter()
                    .map(|consequence| consequence.action())
                    .collect::<Vec<_>>(),
                [FieldCandidateAction::Clone]
            );
            assert_eq!(
                controller
                    .set_field_candidate(request, 0, source, FieldCandidateAction::Clone)
                    .await
                    .unwrap(),
                BuilderTransition::Building
            );
            assert_eq!(
                controller
                    .set_field_inline(request, 1, "project-admins.json".to_owned())
                    .await
                    .unwrap(),
                BuilderTransition::Ready
            );
            let snapshot = controller.inspect_root(request, 4).await.unwrap();
            assert_eq!(
                snapshot.lifecycle(),
                &crate::object_explorer::RootLifecycleSnapshot::Ready
            );
            assert_eq!(snapshot.card().shape(), "ProduceJsonRequest");
            controller.end_value_candidates().await.unwrap();
            controller.close().await.unwrap();
            request
        };

        let (engine, request) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(request).is_some());
        assert_eq!(engine.arena().allocated_slot_count(), 2);
        assert_eq!(engine.json_serialization_count(), 0);
    }
}
