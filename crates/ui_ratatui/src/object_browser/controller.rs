pub(crate) use super::object_browser_controller_error::ObjectBrowserControllerError;
use crate::object_explorer::ArbitraryInvocationStart;
use crate::object_explorer::ArenaBrowseSession;
use crate::object_explorer::ArenaQueryContext;
use crate::object_explorer::Breadcrumb;
use crate::object_explorer::BreadcrumbContextSnapshot;
use crate::object_explorer::Breadcrumbs;
use crate::object_explorer::BuilderTransition;
use crate::object_explorer::CardAddress;
use crate::object_explorer::CardNavigation;
use crate::object_explorer::CardWindow;
use crate::object_explorer::CardWindowBudget;
use crate::object_explorer::ExplorerHandle;
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
use crate::object_explorer::QueryTotal;
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
use std::collections::BTreeMap;
use std::num::NonZeroUsize;

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
    /// Presentation before each breadcrumb, indexed by its position in the
    /// tab's query. Editing a prefix discards return positions beneath it.
    breadcrumb_return_states: BTreeMap<SlotId, BTreeMap<usize, TabUiState>>,
    active_tab_header: TabHeaderSnapshot,
    window: Option<CardWindow>,
    scanning: bool,
    reanchor_on_next_window: bool,
}

impl ObjectBrowserController {
    pub(super) fn query_context(&self) -> ArenaQueryContext {
        self.context.clone()
    }

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
            breadcrumb_return_states: BTreeMap::new(),
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

    pub(crate) async fn focus_first_card(
        &mut self,
        max_work: usize,
        max_cards: NonZeroUsize,
        max_relationship_rows: usize,
    ) -> Result<(), ObjectBrowserControllerError> {
        self.reanchor_on_next_window = true;
        let state = self
            .active_state_mut()
            .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
        state.select(CardAddress::NewSlot);
        state.set_viewport_anchor(CardAddress::NewSlot);
        state.focus_row(None);
        self.refresh_window(max_work, max_cards, max_relationship_rows)
            .await
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
                let start = self
                    .active_state()
                    .and_then(|state| {
                        window
                            .cards()
                            .iter()
                            .position(|card| card.address() == state.viewport_anchor())
                    })
                    .unwrap_or_default();
                // Tail queries may backfill cards before the visual anchor to
                // keep their bounded result full. Those cards are not visible
                // when the create-object card occupies the rightmost column.
                window
                    .cards()
                    .iter()
                    .skip(start)
                    .any(|card| card.address() == &target_card)
            });
            let anchor = (!target_is_visible).then(|| match direction {
                CardNavigation::Next => self
                    .window
                    .as_ref()
                    .filter(|window| {
                        window.cards().last().is_some_and(|card| {
                            matches!(card.address(), CardAddress::Value(address) if address == &from)
                        })
                    })
                    // Reveal one new card at the right edge by discarding only
                    // the leftmost card. Anchoring at `from` jumps a whole page
                    // minus one and leaves the selection in the second column.
                    .and_then(|window| window.cards().get(1))
                    .map(|card| card.address().clone())
                    // A single-card viewport must start at the new selection.
                    .unwrap_or_else(|| target_card.clone()),
                CardNavigation::Previous => target_card.clone(),
            });
            let state = self
                .active_state_mut()
                .ok_or(ObjectBrowserControllerError::NoActiveTab)?;
            state.select(target_card);
            if let Some(anchor) = anchor {
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
        let total = progress.total();
        if let Some(state) = self.active_state_mut() {
            state.set_query_total(total);
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
            QueryProgressState::Complete if matches!(total, QueryTotal::Exact(count) if count > 0) =>
            {
                // The scan found results but not the old viewport anchor (for
                // example a removed list element). Restart at the first match
                // instead of reporting an empty query or seeking it every frame.
                self.window = None;
                self.scanning = true;
                self.reanchor_on_next_window = true;
                if let Some(state) = self.active_state_mut() {
                    state.set_viewport_anchor(CardAddress::NewSlot);
                }
            }
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
        let previous_count = self.active_tab_header.breadcrumb_count();
        let previous_state = self.active_state().cloned();
        let navigation_update = update.clone();
        let tab = self.handle.update_tab(tab_slot, update).await?;
        if query_changed {
            self.browse.set_query(tab.breadcrumbs().clone()).await?;
            let history = self.breadcrumb_return_states.entry(tab_slot).or_default();
            let restored = match navigation_update {
                TabUpdate::PushBreadcrumb(_) => {
                    if let Some(state) = previous_state {
                        history.insert(previous_count, state);
                    }
                    None
                }
                TabUpdate::RemoveBreadcrumb(index) => {
                    let state = history.remove(&index);
                    history.retain(|depth, _| *depth < index);
                    (index + 1 == previous_count).then_some(state).flatten()
                }
                TabUpdate::ReplaceBreadcrumb { index, .. } => {
                    history.retain(|depth, _| *depth <= index);
                    None
                }
                TabUpdate::ReplaceBreadcrumbs(_) => {
                    history.clear();
                    None
                }
                TabUpdate::Rename(_) => unreachable!("renaming does not change the query"),
            };
            self.window = None;
            self.scanning = false;
            self.reanchor_on_next_window = restored.is_none();
            if let Some(state) = self.active_state_mut() {
                if let Some(restored) = restored {
                    *state = restored;
                } else {
                    state.select(CardAddress::NewSlot);
                    state.set_viewport_anchor(CardAddress::NewSlot);
                    state.focus_row(None);
                }
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
        let origin = self.active_state().cloned();
        let breadcrumbs = Breadcrumbs::new(vec![Breadcrumb::projection(
            address.root_id().get(),
            address.path().segments().to_vec(),
        )]);
        let tab = self
            .create_tab_with_query(name, breadcrumbs, Some(CardAddress::Value(address)))
            .await?;
        if let Some(origin) = origin {
            self.breadcrumb_return_states
                .entry(tab)
                .or_default()
                .insert(0, origin);
        }
        Ok(tab)
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
        self.breadcrumb_return_states.remove(&closing);
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
    async fn popping_nested_breadcrumbs_restores_each_viewport_after_tab_switching() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut controller = ObjectBrowserController::bootstrap(context).await.unwrap();
            let mut cards = Vec::new();
            for value in 0_usize..10 {
                let slot = controller.insert_owned(value).await.unwrap();
                cards.push(CardAddress::Value(ValueAddress::root(slot)));
            }
            let capacity = NonZeroUsize::new(3).unwrap();
            controller.focus(cards[3].clone()).unwrap();
            controller.select(cards[4].clone()).unwrap();
            controller
                .focus_row(Some(crate::object_explorer::CardRowKey::Value))
                .unwrap();
            controller.refresh_window(128, capacity, 8).await.unwrap();
            let origin = controller.active_state().unwrap().clone();
            controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec![
                        cloud_terrastodon_registry::describe_shape(usize::SHAPE).to_owned(),
                    ],
                }))
                .await
                .unwrap();
            controller.refresh_window(128, capacity, 8).await.unwrap();
            controller.focus(cards[6].clone()).unwrap();
            controller.select(cards[7].clone()).unwrap();
            controller
                .focus_row(Some(crate::object_explorer::CardRowKey::Value))
                .unwrap();
            controller.refresh_window(128, capacity, 8).await.unwrap();
            let middle = controller.active_state().unwrap().clone();
            controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec![
                        cloud_terrastodon_registry::describe_shape(usize::SHAPE).to_owned(),
                    ],
                }))
                .await
                .unwrap();
            controller
                .update_active_tab(TabUpdate::Rename("nested".to_owned()))
                .await
                .unwrap();
            controller.create_tab("other").await.unwrap();
            assert!(controller.switch_tab_previous().await.unwrap());
            for (index, expected) in [(1, middle), (0, origin)] {
                controller
                    .update_active_tab(TabUpdate::RemoveBreadcrumb(index))
                    .await
                    .unwrap();
                controller.refresh_window(128, capacity, 8).await.unwrap();
                let state = controller.active_state().unwrap();
                assert_eq!(state.selection(), expected.selection());
                assert_eq!(state.focused_row(), expected.focused_row());
                assert_eq!(state.viewport_anchor(), expected.viewport_anchor());
                let visible = controller.window().unwrap().cards();
                assert_eq!(visible[0].address(), expected.viewport_anchor());
                assert_eq!(visible[1].address(), expected.selection());
            }
            controller.close().await.unwrap();
        };
        let (_engine, ()) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
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

        let (_engine, ()) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
    }

    #[tokio::test]
    async fn missing_viewport_anchor_recovers_existing_projection_results() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut controller = ObjectBrowserController::bootstrap(context).await.unwrap();
            let list = controller
                .insert_owned((0_u64..600).collect::<Vec<_>>())
                .await
                .unwrap();
            controller.create_output_tab("list", list).await.unwrap();
            let query = controller.active_breadcrumbs().await.unwrap();
            controller
                .focus(CardAddress::Value(
                    ValueAddress::root(list).child(ValuePathSegment::Index(600)),
                ))
                .unwrap();
            let capacity = NonZeroUsize::new(3).unwrap();
            for _ in 0..100 {
                controller.refresh_window(16, capacity, 3).await.unwrap();
                if controller.window().is_some() {
                    break;
                }
                assert!(
                    controller.is_scanning(),
                    "a missing anchor does not mean the query is empty"
                );
            }
            let expected = CardAddress::Value(ValueAddress::root(list));
            assert_eq!(controller.window().unwrap().cards()[0].address(), &expected);
            assert_eq!(controller.active_state().unwrap().selection(), &expected);
            for _ in 0..8 {
                controller.refresh_window(16, capacity, 3).await.unwrap();
                assert!(!controller.is_scanning());
                assert_eq!(controller.window().unwrap().cards()[0].address(), &expected);
            }
            assert_eq!(controller.active_breadcrumbs().await.unwrap(), query);
            controller.close().await.unwrap();
        };
        tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
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

        let (engine, tab) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
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
                &CardAddress::Value(name.clone()),
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

        let (engine, ()) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn controller_edge_navigation_reveals_only_one_new_card_at_any_capacity() {
        for capacity in [1, 2, 3, 5] {
            let engine = ExplorerEngine::empty();
            let (context, inbox) = ArenaQueryContext::channel(8);
            let client = async move {
                let mut controller = ObjectBrowserController::bootstrap(context).await.unwrap();
                let mut cards = Vec::new();
                for value in 0_usize..20 {
                    let slot = controller.insert_owned(value).await.unwrap();
                    cards.push(CardAddress::Value(ValueAddress::root(slot)));
                }
                controller.focus(cards[0].clone()).unwrap();
                let budget = NonZeroUsize::new(capacity).unwrap();
                controller.refresh_window(128, budget, 1).await.unwrap();

                let mut start = 0;
                for selected in 1..=capacity + 3 {
                    controller
                        .navigate_card(CardNavigation::Next, 128)
                        .await
                        .unwrap();
                    controller.refresh_window(128, budget, 1).await.unwrap();
                    start = selected.saturating_sub(capacity - 1);
                    assert_eq!(
                        controller.active_state().unwrap().selection(),
                        &cards[selected]
                    );
                    assert_eq!(
                        controller
                            .window()
                            .unwrap()
                            .cards()
                            .iter()
                            .map(|card| card.address())
                            .collect::<Vec<_>>(),
                        cards[start..start + capacity].iter().collect::<Vec<_>>(),
                        "capacity {capacity}: right movement reveals only the next card"
                    );
                }
                for selected in (0..capacity + 3).rev() {
                    controller
                        .navigate_card(CardNavigation::Previous, 128)
                        .await
                        .unwrap();
                    controller.refresh_window(128, budget, 1).await.unwrap();
                    start = start.min(selected);
                    assert_eq!(
                        controller.active_state().unwrap().selection(),
                        &cards[selected]
                    );
                    assert_eq!(
                        controller
                            .window()
                            .unwrap()
                            .cards()
                            .iter()
                            .map(|card| card.address())
                            .collect::<Vec<_>>(),
                        cards[start..start + capacity].iter().collect::<Vec<_>>(),
                        "capacity {capacity}: left movement keeps the window until its edge"
                    );
                }
                controller.close().await.unwrap();
            };
            let (_engine, ()) = tokio::join!(
                async {
                    engine
                        .run(inbox)
                        .await
                        .expect("synthetic engine should finish")
                },
                client
            );
        }
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

        let (engine, list) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
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

        let (engine, slot) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
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

        let (engine, ()) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
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

        let (engine, request) = tokio::join!(
            async {
                engine
                    .run(inbox)
                    .await
                    .expect("synthetic engine should finish")
            },
            client
        );
        assert!(engine.arena().ready_value(request).is_some());
        assert_eq!(engine.arena().allocated_slot_count(), 2);
        assert_eq!(engine.json_serialization_count(), 0);
    }
}
