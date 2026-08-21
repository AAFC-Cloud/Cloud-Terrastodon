use super::app::BreadcrumbValueEditorState;
use super::app::BrowserMode;
use super::app::ObjectBrowserApp;
use super::app::PendingFieldLink;
use super::breadcrumb_bar_focus::BreadcrumbBarFocus;
use super::breadcrumb_filter_field_picker_task::BreadcrumbFilterFieldPickerOutcome;
use super::breadcrumb_filter_field_picker_task::BreadcrumbFilterFieldPickerTask;
use super::breadcrumb_menu_task::BreadcrumbMenuOutcome;
use super::breadcrumb_menu_task::BreadcrumbMenuTask;
use super::breadcrumb_picker_task::BreadcrumbPickerOutcome;
use super::breadcrumb_picker_task::BreadcrumbPickerTarget;
use super::breadcrumb_picker_task::BreadcrumbPickerTask;
use super::breadcrumb_picker_task::BreadcrumbPickerValue;
use super::breadcrumb_value_picker_task::BreadcrumbValuePickerOutcome;
use super::breadcrumb_value_picker_task::BreadcrumbValuePickerTask;
use super::controller::ObjectBrowserControllerError;
use super::link_action_picker_task::LinkActionPickerOutcome;
use super::link_action_picker_task::LinkActionPickerTask;
use super::pickers::BreadcrumbPicker;
use super::pickers::BreadcrumbPickerChoice;
use super::pickers::FieldValuePicker;
use super::pickers::LinkActionPicker;
use super::pickers::ValuePickerChoice;
use super::pickers::VariantPicker;
use super::render::CardLayoutAxis;
use super::shape_picker_task::ShapePickerOutcome;
use super::shape_picker_task::ShapePickerTask;
use super::value_picker_task::ValuePickerOutcome;
use super::value_picker_task::ValuePickerTask;
use super::variant_picker_task::VariantPickerOutcome;
use super::variant_picker_task::VariantPickerTask;
use crate::object_explorer::Breadcrumb;
use crate::object_explorer::BuilderKindSnapshot;
use crate::object_explorer::BuilderTransition;
use crate::object_explorer::CardAddress;
use crate::object_explorer::CardNavigation;
use crate::object_explorer::CardRowContent;
use crate::object_explorer::CardRowKey;
use crate::object_explorer::FieldBindingSnapshot;
use crate::object_explorer::FieldCandidateAction;
use crate::object_explorer::ProductionStrategy;
use crate::object_explorer::QueryProgressState;
use crate::object_explorer::SlotId;
use crate::object_explorer::TabUpdate;
use crate::object_explorer::ValueAddress;
use rand::RngExt;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::KeyModifiers;
use std::time::Duration;
use std::time::Instant;

impl ObjectBrowserApp {
    pub(crate) async fn handle_event(
        &mut self,
        event: &Event,
    ) -> Result<(), ObjectBrowserControllerError> {
        let Event::Key(key) = event else {
            return Ok(());
        };
        if key.kind == KeyEventKind::Release {
            return Ok(());
        }
        if key.code == KeyCode::Esc && key.kind != KeyEventKind::Press {
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
        {
            self.should_quit = true;
            return Ok(());
        }
        match self.mode {
            BrowserMode::Pool => self.handle_pool_key(*key).await,
            BrowserMode::RowSearch => self.handle_row_search_key(*key).await,
            BrowserMode::Variant => self.handle_variant_key(*key).await,
            BrowserMode::Value => self.handle_value_key(*key).await,
            BrowserMode::LinkAction => self.handle_link_action_key(*key).await,
            BrowserMode::Text => self.handle_text_key(*key).await,
            BrowserMode::BreadcrumbValue => self.handle_breadcrumb_value_key(*key).await,
            BrowserMode::NestedPicker => Ok(()),
            BrowserMode::TabName => self.handle_tab_name_key(*key).await,
        }
    }

    async fn handle_pool_key(&mut self, key: KeyEvent) -> Result<(), ObjectBrowserControllerError> {
        if key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char('[') | KeyCode::Char('{'))
        {
            if self.controller.switch_tab_previous().await? {
                self.active_root = None;
                self.breadcrumb_focus = None;
                self.status = format!(
                    "Switched to tab slot {}.",
                    self.controller.active_tab_header().slot()
                );
            }
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char(']') | KeyCode::Char('}'))
        {
            let tab = self.controller.switch_tab_next().await?;
            self.active_root = None;
            self.breadcrumb_focus = None;
            self.status = format!("Switched to tab slot {tab}.");
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char(';') | KeyCode::Char(':'))
        {
            self.open_breadcrumb_picker().await?;
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Backspace {
            self.remove_last_breadcrumb().await?;
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('w') | KeyCode::Char('W'))
        {
            let tab = self.controller.close_active_tab().await?;
            self.active_root = None;
            self.breadcrumb_focus = None;
            self.status = format!("Closed tab; active tab is slot {tab}.");
            return Ok(());
        }
        if key.code == KeyCode::F(2) {
            self.breadcrumb_focus = None;
            self.tab_name_editor = Some(self.controller.active_tab_header().name().to_owned());
            self.mode = BrowserMode::TabName;
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('-') {
            self.resize_cards(-1);
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::ALT)
            && matches!(key.code, KeyCode::Char('+') | KeyCode::Char('='))
        {
            self.resize_cards(1);
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('t') | KeyCode::Char('T'))
        {
            self.axis = match self.axis {
                CardLayoutAxis::Horizontal => CardLayoutAxis::Vertical,
                CardLayoutAxis::Vertical => CardLayoutAxis::Horizontal,
            };
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Home {
            self.active_root = None;
            self.breadcrumb_focus = None;
            self.controller
                .focus_first_card(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
                .await?;
            return Ok(());
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::End {
            self.active_root = None;
            self.breadcrumb_focus = None;
            self.controller.select(CardAddress::NewSlot)?;
            self.controller.focus_row(None)?;
            return Ok(());
        }
        if self.breadcrumb_focus.is_some() {
            return self.handle_breadcrumb_bar_key(key).await;
        }
        match key.code {
            KeyCode::Esc => {
                if self.active_root.take().is_some() || self.focused_card_fill {
                    self.recent_escape_presses.clear();
                    self.focused_card_fill = false;
                    self.controller
                        .refresh_window(
                            Self::FRAME_WORK,
                            self.max_cards,
                            self.max_relationship_rows,
                        )
                        .await?;
                } else if self.controller.active_tab_header().breadcrumb_count() > 0 {
                    self.recent_escape_presses.clear();
                    self.remove_last_breadcrumb().await?;
                } else {
                    let now = Instant::now();
                    self.recent_escape_presses.retain(|pressed_at| {
                        now.duration_since(*pressed_at) <= Duration::from_secs(5)
                    });
                    self.recent_escape_presses.push(now);
                    let remaining = 3usize.saturating_sub(self.recent_escape_presses.len());
                    if remaining == 0 {
                        self.should_quit = true;
                    } else {
                        self.status = format!(
                            "Hit Esc {remaining} more time{} within 5 seconds to exit.",
                            if remaining == 1 { "" } else { "s" }
                        );
                    }
                }
            }
            KeyCode::F(3) => self.focused_card_fill = !self.focused_card_fill,
            KeyCode::Left => self.navigate(CardNavigation::Previous).await?,
            KeyCode::Right => self.navigate(CardNavigation::Next).await?,
            KeyCode::Up => self.move_row(false)?,
            KeyCode::Down => self.move_row(true)?,
            KeyCode::Home => self.focus_row_edge(true)?,
            KeyCode::End => self.focus_row_edge(false)?,
            KeyCode::Enter | KeyCode::Char(' ') => self.activate_pool_selection().await?,
            KeyCode::Char(character)
                if character != ' '
                    && !character.is_control()
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.start_row_search(character)?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_breadcrumb_bar_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        let breadcrumb_count = self.controller.active_tab_header().breadcrumb_count();
        match key.code {
            KeyCode::Esc => {
                self.breadcrumb_focus = None;
                self.status = "Returned focus to the object pool.".to_owned();
            }
            KeyCode::Down => {
                self.breadcrumb_focus = None;
                self.focus_row_edge(true)?;
            }
            KeyCode::Left => {
                if let Some(focus) = self.breadcrumb_focus.as_mut() {
                    focus.move_previous();
                }
            }
            KeyCode::Right => {
                if let Some(focus) = self.breadcrumb_focus.as_mut() {
                    focus.move_next(breadcrumb_count);
                }
            }
            KeyCode::Home => {
                self.breadcrumb_focus = Some(BreadcrumbBarFocus::first());
            }
            KeyCode::End => {
                self.breadcrumb_focus = Some(BreadcrumbBarFocus::add(breadcrumb_count));
            }
            KeyCode::Delete | KeyCode::Backspace => {
                if let Some(index) = self
                    .breadcrumb_focus
                    .and_then(|focus| focus.operation(breadcrumb_count))
                {
                    self.remove_breadcrumb(index).await?;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_breadcrumb_bar_focus().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn activate_breadcrumb_bar_focus(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let Some(focus) = self.breadcrumb_focus else {
            return Ok(());
        };
        let breadcrumb_count = self.controller.active_tab_header().breadcrumb_count();
        let Some(index) = focus.operation(breadcrumb_count) else {
            return self.open_breadcrumb_picker().await;
        };
        let breadcrumbs = self.controller.active_breadcrumbs().await?;
        let Some(breadcrumb) = breadcrumbs.operations().get(index).cloned() else {
            self.breadcrumb_focus = Some(BreadcrumbBarFocus::add(breadcrumb_count));
            return Ok(());
        };
        match breadcrumb {
            Breadcrumb::ShapeFilter { included_shapes } => {
                self.breadcrumb_focus = None;
                self.launch_shape_breadcrumb_picker(Some(index), included_shapes)
                    .await?;
            }
            Breadcrumb::ProjectFields {
                mode,
                included_fields,
            } => {
                self.breadcrumb_focus = None;
                self.launch_field_breadcrumb_picker(Some(index), mode, included_fields)
                    .await?;
            }
            Breadcrumb::ValueFilter {
                field_shape,
                field_name,
                operator,
                value,
            } => {
                self.breadcrumb_focus = None;
                self.launch_breadcrumb_value_picker(
                    Some(index),
                    field_shape,
                    field_name,
                    operator,
                    Some(value),
                )
                .await?;
            }
            Breadcrumb::Projection { .. }
            | Breadcrumb::AddressKindFilter { .. }
            | Breadcrumb::Pop => {
                self.status =
                    "This breadcrumb has no editable values; press Delete to remove it.".to_owned();
            }
        }
        Ok(())
    }

    fn start_row_search(&mut self, character: char) -> Result<(), ObjectBrowserControllerError> {
        let Some(card) = self.selected_card().cloned() else {
            return Ok(());
        };
        if card.address() == &CardAddress::NewSlot {
            return Ok(());
        }
        let search = super::row_search::RowSearchState::new(&card, character.to_string());
        let focused = search.selected().cloned();
        if let Some(state) = self.controller.active_state_mut() {
            state.set_search_query(search.query());
        }
        self.controller.focus_row(focused)?;
        self.row_search = Some(search);
        self.mode = BrowserMode::RowSearch;
        Ok(())
    }

    async fn handle_row_search_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.close_row_search();
            }
            KeyCode::Enter => {
                let has_match = self
                    .row_search
                    .as_ref()
                    .and_then(super::row_search::RowSearchState::selected)
                    .is_some();
                self.close_row_search();
                if has_match {
                    self.activate_pool_selection().await?;
                }
            }
            KeyCode::Up => self.move_row_search(-1)?,
            KeyCode::Down => self.move_row_search(1)?,
            KeyCode::PageUp => self.move_row_search(-10)?,
            KeyCode::PageDown => self.move_row_search(10)?,
            KeyCode::Home => self.move_row_search_edge(true)?,
            KeyCode::End => self.move_row_search_edge(false)?,
            KeyCode::Backspace => {
                let Some(card) = self.selected_card().cloned() else {
                    self.close_row_search();
                    return Ok(());
                };
                if let Some(search) = self.row_search.as_mut() {
                    search.pop(&card);
                }
                if self
                    .row_search
                    .as_ref()
                    .is_none_or(|search| search.query().is_empty())
                {
                    self.close_row_search();
                } else {
                    self.sync_row_search_focus()?;
                }
            }
            KeyCode::Char(character)
                if character != ' '
                    && !character.is_control()
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                let Some(card) = self.selected_card().cloned() else {
                    self.close_row_search();
                    return Ok(());
                };
                if let Some(search) = self.row_search.as_mut() {
                    search.push(&card, character);
                }
                self.sync_row_search_focus()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn move_row_search(&mut self, delta: isize) -> Result<(), ObjectBrowserControllerError> {
        if let Some(search) = self.row_search.as_mut() {
            search.move_by(delta);
        }
        self.sync_row_search_focus()
    }

    fn move_row_search_edge(&mut self, first: bool) -> Result<(), ObjectBrowserControllerError> {
        if let Some(search) = self.row_search.as_mut() {
            search.move_to_edge(first);
        }
        self.sync_row_search_focus()
    }

    fn sync_row_search_focus(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let (query, focused) = self
            .row_search
            .as_ref()
            .map(|search| (search.query().to_owned(), search.selected().cloned()))
            .unwrap_or_default();
        if let Some(state) = self.controller.active_state_mut() {
            state.set_search_query(query);
        }
        self.controller.focus_row(focused)
    }

    fn close_row_search(&mut self) {
        self.row_search = None;
        self.mode = BrowserMode::Pool;
        if let Some(state) = self.controller.active_state_mut() {
            state.set_search_query(String::new());
        }
    }

    fn resize_cards(&mut self, direction: isize) {
        let (current, minimum, dimension) = match self.axis {
            CardLayoutAxis::Horizontal => (self.card_width, Self::MIN_CARD_WIDTH, "width"),
            CardLayoutAxis::Vertical => (self.card_height, Self::MIN_CARD_HEIGHT, "height"),
        };
        let resized = resize_card_breadth(current, minimum, self.last_card_main_axis, direction);
        match self.axis {
            CardLayoutAxis::Horizontal => self.card_width = resized,
            CardLayoutAxis::Vertical => self.card_height = resized,
        }
        self.status = format!("Card {dimension}: {resized}.");
    }

    async fn navigate(
        &mut self,
        direction: CardNavigation,
    ) -> Result<(), ObjectBrowserControllerError> {
        if self.selected_address() == CardAddress::NewSlot {
            if direction == CardNavigation::Previous
                && let Some(card) = self
                    .controller
                    .window()
                    .and_then(|window| window.cards().last())
            {
                self.controller.focus(card.address().clone())?;
            }
            return Ok(());
        }
        self.active_root = None;
        match self
            .controller
            .navigate_card(direction, Self::FRAME_WORK)
            .await?
        {
            QueryProgressState::Pending => self.status = "Scanning…".to_owned(),
            QueryProgressState::Complete if direction == CardNavigation::Next => {
                self.controller.select(CardAddress::NewSlot)?;
            }
            _ => {}
        }
        self.controller.focus_row(None)?;
        self.controller
            .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
            .await?;
        Ok(())
    }

    fn move_row(&mut self, next: bool) -> Result<(), ObjectBrowserControllerError> {
        let rows = self
            .selected_card()
            .map(|card| {
                card.rows()
                    .iter()
                    .map(|row| row.key().clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let current = self
            .controller
            .active_state()
            .and_then(|state| state.focused_row())
            .and_then(|key| rows.iter().position(|candidate| candidate == key));
        if !next && (rows.is_empty() || current.is_none() || current == Some(0)) {
            self.controller.focus_row(None)?;
            self.breadcrumb_focus = Some(BreadcrumbBarFocus::add(
                self.controller.active_tab_header().breadcrumb_count(),
            ));
            self.status =
                "Breadcrumbs focused; Left/Right selects, Enter edits, Delete removes.".to_owned();
            return Ok(());
        }
        if rows.is_empty() {
            return self.controller.focus_row(None);
        }
        let index = match (current, next) {
            (Some(index), true) => (index + 1).min(rows.len() - 1),
            (Some(index), false) => index.saturating_sub(1),
            (None, true) => 0,
            (None, false) => rows.len() - 1,
        };
        self.controller.focus_row(Some(rows[index].clone()))
    }

    fn focus_row_edge(&mut self, first: bool) -> Result<(), ObjectBrowserControllerError> {
        let key = self.selected_card().and_then(|card| {
            if first {
                card.rows().first()
            } else {
                card.rows().last()
            }
            .map(|row| row.key().clone())
        });
        self.controller.focus_row(key)
    }

    async fn activate_pool_selection(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let selected = self.selected_address();
        if selected == CardAddress::NewSlot {
            return self.create_shape_unset_root().await;
        }
        let CardAddress::Value(address) = selected else {
            return Ok(());
        };
        let focused = self
            .controller
            .active_state()
            .and_then(|state| state.focused_row())
            .cloned();
        let focused_content = focused.as_ref().and_then(|key| {
            self.selected_card()
                .and_then(|card| card.rows().iter().find(|row| row.key() == key))
                .map(|row| row.content().clone())
        });
        let focused_type_name = focused.as_ref().and_then(|key| {
            self.selected_card()
                .and_then(|card| card.rows().iter().find(|row| row.key() == key))
                .and_then(|row| row.type_name().map(ToOwned::to_owned))
        });

        if address.path().segments().is_empty()
            && self
                .active_root
                .as_ref()
                .is_none_or(|root| root.slot() != address.root_id())
        {
            let snapshot = self
                .controller
                .inspect_root(address.root_id(), self.max_relationship_rows)
                .await?;
            if snapshot.builder().is_some() {
                self.active_root = Some(snapshot);
            }
        }

        if address.path().segments().is_empty()
            && let Some(builder) = self.active_root.as_ref().and_then(|root| root.builder())
        {
            match focused.as_ref() {
                Some(CardRowKey::Shape) => {
                    self.start_shape_picker()?;
                    return Ok(());
                }
                Some(CardRowKey::Variant) => {
                    if let Some(shape) = builder.shape() {
                        self.variant_picker = VariantPicker::new(address.root_id(), shape);
                        if !self.start_variant_picker_task() {
                            self.mode = BrowserMode::Variant;
                        }
                    }
                    return Ok(());
                }
                Some(CardRowKey::Value) => {
                    self.scalar_editor_for_root(address.root_id());
                    return Ok(());
                }
                Some(CardRowKey::Field(name)) => {
                    let field = builder
                        .fields()
                        .iter()
                        .find(|field| field.name() == name)
                        .map(|field| (field.index(), field.name().to_owned(), field.shape()));
                    if let Some((field, name, shape)) = field {
                        return self
                            .open_field_value_picker(address.root_id(), field, name, shape)
                            .await;
                    }
                }
                _ => {}
            }
        }

        if let Some(CardRowContent::RootAction(action)) = focused_content {
            let (function, output, provenance) =
                if let Some((request_function, constructor)) = action.arbitrary_invocation() {
                    let mut bytes = vec![0_u8; 4096];
                    rand::rng().fill(bytes.as_mut_slice());
                    let start = self
                        .controller
                        .invoke_arbitrary(address.root_id(), request_function, constructor, bytes)
                        .await?;
                    (
                        request_function,
                        start.output(),
                        format!(
                            " via arbitrary plan {} and source slot {}",
                            start.plan_id(),
                            start.source()
                        ),
                    )
                } else if let Some((function, mode)) = action.invocation() {
                    let start = self
                        .controller
                        .invoke(address.root_id(), function, mode)
                        .await?;
                    (function, start.output(), String::new())
                } else {
                    return Ok(());
                };
            self.controller
                .create_output_tab(
                    cloud_terrastodon_registry::describe_shape(function.output_shape),
                    output,
                )
                .await?;
            self.active_root = Some(
                self.controller
                    .inspect_root(output, self.max_relationship_rows)
                    .await?,
            );
            self.status = format!(
                "{} created output slot {output}{provenance} in a new tab",
                action.label()
            );
        } else if let Some(CardRowContent::Address(target)) = focused_content {
            self.active_root = None;
            let source_tab = self.controller.active_tab();
            let name = focused_type_name.unwrap_or_else(|| target.to_string());
            let projection_tab = self
                .controller
                .create_projection_tab(name, target.clone())
                .await?;
            self.controller
                .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
                .await?;
            self.status = source_tab.map_or_else(
                || format!("Opened {target} in projection tab slot {projection_tab}."),
                |source_tab| {
                    format!(
                        "Opened {target} in projection tab slot {projection_tab}; source tab slot {source_tab} is unchanged."
                    )
                },
            );
        }
        Ok(())
    }

    async fn create_shape_unset_root(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let slot = self.controller.reserve_builder().await?;
        self.controller
            .focus(CardAddress::Value(ValueAddress::root(slot)))?;
        self.controller.focus_row(Some(CardRowKey::Shape))?;
        self.active_root = Some(
            self.controller
                .inspect_root(slot, self.max_relationship_rows)
                .await?,
        );
        self.controller
            .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
            .await?;
        self.start_shape_picker()?;
        Ok(())
    }

    async fn open_field_value_picker(
        &mut self,
        destination: SlotId,
        field: usize,
        name: String,
        shape: &'static facet::Shape,
    ) -> Result<(), ObjectBrowserControllerError> {
        self.value_picker = Some(FieldValuePicker::new(destination, field, name, shape));
        self.controller.begin_value_candidates(shape).await?;
        self.mode = BrowserMode::Value;
        self.refresh_value_candidates().await
    }

    pub(super) fn start_value_picker_task(&mut self) -> bool {
        let Some(picker) = self.value_picker.as_ref() else {
            return false;
        };
        match ValuePickerTask::spawn(picker) {
            Ok(task) => {
                self.value_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = "PickerTui is choosing a compatible object.".to_owned();
                true
            }
            Err(_) => false,
        }
    }

    fn start_variant_picker_task(&mut self) -> bool {
        let Some(picker) = self.variant_picker.as_ref() else {
            return false;
        };
        match VariantPickerTask::spawn(picker) {
            Ok(task) => {
                self.variant_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = "PickerTui is choosing an enum variant.".to_owned();
                true
            }
            Err(_) => false,
        }
    }

    pub(super) fn start_link_action_picker_task(&mut self) -> bool {
        let Some(picker) = self.link_action_picker.as_ref() else {
            return false;
        };
        match LinkActionPickerTask::spawn(picker) {
            Ok(task) => {
                self.link_action_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = "PickerTui is choosing how to set the field.".to_owned();
                true
            }
            Err(_) => false,
        }
    }

    pub(super) async fn finish_variant_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .variant_picker_task
            .as_ref()
            .is_some_and(VariantPickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .variant_picker_task
            .take()
            .expect("a finished variant picker task is present");
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            VariantPickerOutcome::Selected { slot, variant } => {
                let transition = self.controller.select_variant(slot, variant).await?;
                self.active_root = Some(
                    self.controller
                        .inspect_root(slot, self.max_relationship_rows)
                        .await?,
                );
                self.variant_picker = None;
                self.mode = BrowserMode::Pool;
                self.focus_after_builder_transition(slot, transition)?;
                self.controller
                    .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
                    .await?;
            }
            VariantPickerOutcome::Cancelled => {
                self.mode = BrowserMode::Pool;
                self.status = "Variant selection cancelled.".to_owned();
            }
            VariantPickerOutcome::Failed(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Variant PickerTui failed: {message}");
            }
        }
        Ok(())
    }

    pub(super) async fn finish_value_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .value_picker_task
            .as_ref()
            .is_some_and(ValuePickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .value_picker_task
            .take()
            .expect("a finished value picker task is present");
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            ValuePickerOutcome::Selected {
                destination,
                field,
                choice,
            } => {
                self.controller.end_value_candidates().await?;
                self.value_picker = None;
                self.accept_value_picker_choice_data(destination, field, choice)
                    .await?;
            }
            ValuePickerOutcome::Cancelled => {
                self.controller.end_value_candidates().await?;
                self.value_picker = None;
                self.mode = BrowserMode::Pool;
                self.status = "Object selection cancelled.".to_owned();
            }
            ValuePickerOutcome::Failed(message) => {
                self.controller.end_value_candidates().await?;
                self.value_picker = None;
                self.mode = BrowserMode::Pool;
                self.status = format!("Object PickerTui failed: {message}");
            }
        }
        Ok(())
    }

    pub(super) async fn finish_link_action_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .link_action_picker_task
            .as_ref()
            .is_some_and(LinkActionPickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .link_action_picker_task
            .take()
            .expect("a finished link action picker task is present");
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            LinkActionPickerOutcome::Selected(action) => {
                self.accept_link_action_value(action).await?;
            }
            LinkActionPickerOutcome::Cancelled => {
                self.pending_link = None;
                self.link_action_picker = None;
                self.mode = BrowserMode::Pool;
                self.status = "Transfer action selection cancelled.".to_owned();
            }
            LinkActionPickerOutcome::Failed(message) => {
                self.pending_link = None;
                self.link_action_picker = None;
                self.mode = BrowserMode::Pool;
                self.status = format!("Link action PickerTui failed: {message}");
            }
        }
        Ok(())
    }

    fn start_shape_picker(&mut self) -> Result<(), ObjectBrowserControllerError> {
        match ShapePickerTask::spawn() {
            Ok(task) => {
                self.shape_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = "Pick a reflected shape; type to fuzzy-search.".to_owned();
            }
            Err(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Could not open shape picker: {message}");
            }
        }
        Ok(())
    }

    pub(super) async fn finish_shape_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .shape_picker_task
            .as_ref()
            .is_some_and(ShapePickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .shape_picker_task
            .take()
            .expect("a finished shape picker task is present");
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            ShapePickerOutcome::Selected(label) => {
                let Some(shape) = cloud_terrastodon_registry::known_shapes()
                    .into_iter()
                    .find(|candidate| candidate.label == label)
                    .map(|candidate| candidate.thing.shape)
                else {
                    self.mode = BrowserMode::Pool;
                    self.status = format!("Shape {label} is no longer registered.");
                    return Ok(());
                };
                let Some(slot) = self.active_root.as_ref().map(|root| root.slot()) else {
                    self.mode = BrowserMode::Pool;
                    return Ok(());
                };
                let transition = self.controller.set_builder_shape(slot, shape).await?;
                self.active_root = Some(
                    self.controller
                        .inspect_root(slot, self.max_relationship_rows)
                        .await?,
                );
                self.mode = BrowserMode::Pool;
                self.focus_after_builder_transition(slot, transition)?;
                self.controller
                    .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
                    .await?;
                if transition == BuilderTransition::Ready {
                    self.status = format!("slot {slot} is ready");
                }
            }
            ShapePickerOutcome::Cancelled => {
                self.mode = BrowserMode::Pool;
                self.status = "Shape selection cancelled.".to_owned();
            }
            ShapePickerOutcome::Failed(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Shape PickerTui failed: {message}");
            }
        }
        Ok(())
    }

    async fn handle_variant_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.variant_picker = None;
                self.mode = BrowserMode::Pool;
            }
            KeyCode::Up => {
                if let Some(picker) = self.variant_picker.as_mut() {
                    picker.move_previous();
                }
            }
            KeyCode::Down => {
                if let Some(picker) = self.variant_picker.as_mut() {
                    picker.move_next();
                }
            }
            KeyCode::Backspace => {
                if let Some(picker) = self.variant_picker.as_mut() {
                    picker.pop();
                }
            }
            KeyCode::Enter => {
                let choice = self.variant_picker.as_ref().and_then(|picker| {
                    picker
                        .selected_variant()
                        .map(|variant| (picker.slot(), variant))
                });
                if let Some((slot, variant)) = choice {
                    self.controller.select_variant(slot, variant).await?;
                    self.active_root = Some(
                        self.controller
                            .inspect_root(slot, self.max_relationship_rows)
                            .await?,
                    );
                    self.variant_picker = None;
                    self.mode = BrowserMode::Pool;
                    self.focus_next_builder_input()?;
                }
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(picker) = self.variant_picker.as_mut() {
                    picker.push(character);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_value_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.controller.end_value_candidates().await?;
                self.value_picker = None;
                self.mode = BrowserMode::Pool;
            }
            KeyCode::Up => {
                if let Some(picker) = self.value_picker.as_mut() {
                    picker.move_previous();
                }
            }
            KeyCode::Down => {
                if let Some(picker) = self.value_picker.as_mut() {
                    picker.move_next();
                }
            }
            KeyCode::Backspace => {
                if let Some(picker) = self.value_picker.as_mut() {
                    picker.pop();
                }
            }
            KeyCode::Enter => self.accept_value_picker_choice().await?,
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(picker) = self.value_picker.as_mut() {
                    picker.push(character);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn accept_value_picker_choice(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let Some((destination, field, choice)) = self.value_picker.as_ref().and_then(|picker| {
            picker
                .selected()
                .map(|row| (picker.destination(), picker.field(), row.choice().clone()))
        }) else {
            return Ok(());
        };
        self.controller.end_value_candidates().await?;
        self.value_picker = None;
        self.accept_value_picker_choice_data(destination, field, choice)
            .await
    }

    async fn accept_value_picker_choice_data(
        &mut self,
        destination: SlotId,
        field: usize,
        choice: ValuePickerChoice,
    ) -> Result<(), ObjectBrowserControllerError> {
        match choice {
            ValuePickerChoice::Candidate { address } => {
                self.pending_link = Some(PendingFieldLink {
                    destination,
                    field,
                    source: address,
                });
                self.open_pending_link_actions().await?;
            }
            ValuePickerChoice::CreateNewOwned { shape } => {
                let (source, transition) = self.controller.create_builder(shape).await?;
                self.pending_link = Some(PendingFieldLink {
                    destination,
                    field,
                    source: ValueAddress::root(source),
                });
                self.controller
                    .focus(CardAddress::Value(ValueAddress::root(source)))?;
                self.active_root = Some(
                    self.controller
                        .inspect_root(source, self.max_relationship_rows)
                        .await?,
                );
                if transition == BuilderTransition::Ready {
                    self.open_pending_link_actions().await?;
                } else {
                    self.mode = BrowserMode::Pool;
                    self.focus_next_builder_input()?;
                    self.scalar_editor_for_root(source);
                }
            }
            ValuePickerChoice::InvokeDefaultProducer { function } => {
                let batch = self
                    .controller
                    .start_production(
                        destination,
                        field,
                        function,
                        ProductionStrategy::Default,
                        Self::FRAME_WORK,
                    )
                    .await?;
                self.apply_production_batch(&batch);
                self.focus_destination_after_producer_start(destination)
                    .await?;
            }
            ValuePickerChoice::CreateProducer { function } => {
                let batch = self
                    .controller
                    .start_production(
                        destination,
                        field,
                        function,
                        ProductionStrategy::Manual,
                        Self::FRAME_WORK,
                    )
                    .await?;
                let input = batch.updates().first().and_then(|update| update.input());
                self.apply_production_batch(&batch);
                if let Some(input) = input {
                    self.controller
                        .focus(CardAddress::Value(ValueAddress::root(input)))?;
                    self.active_root = Some(
                        self.controller
                            .inspect_root(input, self.max_relationship_rows)
                            .await?,
                    );
                    self.mode = BrowserMode::Pool;
                    self.focus_next_builder_input()?;
                    self.scalar_editor_for_root(input);
                } else {
                    self.focus_destination_after_producer_start(destination)
                        .await?;
                }
            }
            ValuePickerChoice::InvokeArbitraryProducer { function } => {
                let mut bytes = vec![0_u8; 4096];
                rand::rng().fill(bytes.as_mut_slice());
                let batch = self
                    .controller
                    .start_production(
                        destination,
                        field,
                        function,
                        ProductionStrategy::Arbitrary { bytes },
                        Self::FRAME_WORK,
                    )
                    .await?;
                self.apply_production_batch(&batch);
                self.focus_destination_after_producer_start(destination)
                    .await?;
            }
        }
        Ok(())
    }

    async fn focus_destination_after_producer_start(
        &mut self,
        destination: SlotId,
    ) -> Result<(), ObjectBrowserControllerError> {
        self.controller
            .focus(CardAddress::Value(ValueAddress::root(destination)))?;
        self.active_root = Some(
            self.controller
                .inspect_root(destination, self.max_relationship_rows)
                .await?,
        );
        self.mode = BrowserMode::Pool;
        self.focus_next_builder_input()?;
        Ok(())
    }

    async fn handle_link_action_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.link_action_picker = None;
                self.pending_link = None;
                self.mode = BrowserMode::Pool;
            }
            KeyCode::Up => {
                if let Some(picker) = self.link_action_picker.as_mut() {
                    picker.move_previous();
                }
            }
            KeyCode::Down => {
                if let Some(picker) = self.link_action_picker.as_mut() {
                    picker.move_next();
                }
            }
            KeyCode::Enter => self.accept_link_action().await?,
            _ => {}
        }
        Ok(())
    }

    async fn accept_link_action(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let Some(action) = self
            .link_action_picker
            .as_ref()
            .and_then(LinkActionPicker::selected_action)
        else {
            return Ok(());
        };
        self.accept_link_action_value(action).await
    }

    async fn accept_link_action_value(
        &mut self,
        action: FieldCandidateAction,
    ) -> Result<(), ObjectBrowserControllerError> {
        let Some(link) = self.pending_link.clone() else {
            return Ok(());
        };
        let transition = self
            .controller
            .set_field_candidate(link.destination, link.field, link.source, action)
            .await?;
        self.pending_link = None;
        self.link_action_picker = None;
        self.controller
            .focus(CardAddress::Value(ValueAddress::root(link.destination)))?;
        self.active_root = Some(
            self.controller
                .inspect_root(link.destination, self.max_relationship_rows)
                .await?,
        );
        self.mode = BrowserMode::Pool;
        self.focus_after_builder_transition(link.destination, transition)?;
        self.controller
            .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
            .await?;
        if transition == BuilderTransition::Ready {
            self.status = format!("slot {} is ready", link.destination);
        }
        Ok(())
    }

    async fn open_breadcrumb_picker(&mut self) -> Result<(), ObjectBrowserControllerError> {
        self.breadcrumb_focus = None;
        let selected = self.selected_card().and_then(|card| {
            let CardAddress::Value(address) = card.address() else {
                return None;
            };
            let focused = self
                .controller
                .active_state()
                .and_then(|state| state.focused_row())
                .and_then(|key| card.rows().iter().find(|row| row.key() == key));
            if let Some(row) = focused
                && let CardRowContent::Address(field_address) = row.content()
            {
                return Some((
                    field_address.clone(),
                    row.type_name().unwrap_or(card.shape()).to_owned(),
                ));
            }
            Some((address.clone(), card.shape().to_owned()))
        });
        let breadcrumbs = self.controller.active_breadcrumbs().await?;
        let context = self
            .controller
            .inspect_breadcrumb_context(breadcrumbs.operations().len(), Self::FRAME_WORK, 2_048)
            .await?;
        let picker = BreadcrumbPicker::new(
            selected
                .as_ref()
                .map(|(address, shape)| (address, shape.as_str())),
            breadcrumbs.operations(),
            Some(&context),
        );
        match BreadcrumbMenuTask::spawn(&picker) {
            Ok(task) => {
                self.breadcrumb_menu_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = "PickerTui is choosing a breadcrumb operation.".to_owned();
            }
            Err(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Could not open breadcrumb PickerTui: {message}");
            }
        }
        Ok(())
    }

    async fn accept_breadcrumb_choice(
        &mut self,
        choice: BreadcrumbPickerChoice,
    ) -> Result<(), ObjectBrowserControllerError> {
        match choice {
            BreadcrumbPickerChoice::Add(breadcrumb) => {
                self.controller
                    .update_active_tab(TabUpdate::PushBreadcrumb(breadcrumb))
                    .await?;
                self.finish_query_change().await?;
            }
            BreadcrumbPickerChoice::PromptValue {
                edit_index,
                initial_value,
                field_shape,
                field_name,
                operator,
            } => {
                self.launch_breadcrumb_value_picker(
                    edit_index,
                    field_shape,
                    field_name,
                    operator,
                    initial_value,
                )
                .await?;
            }
            BreadcrumbPickerChoice::PickShapes {
                edit_index,
                initially_included,
            } => {
                self.launch_shape_breadcrumb_picker(edit_index, initially_included)
                    .await?;
            }
            BreadcrumbPickerChoice::PickFilterFields => {
                self.launch_filter_field_breadcrumb_picker().await?;
            }
            BreadcrumbPickerChoice::PickFields {
                edit_index,
                mode,
                initially_included,
            } => {
                self.launch_field_breadcrumb_picker(edit_index, mode, initially_included)
                    .await?;
            }
        }
        Ok(())
    }

    async fn launch_breadcrumb_value_picker(
        &mut self,
        edit_index: Option<usize>,
        field_shape: String,
        field_name: String,
        operator: crate::object_explorer::ValueFilterOperator,
        initial_value: Option<String>,
    ) -> Result<(), ObjectBrowserControllerError> {
        let prefix_len =
            edit_index.unwrap_or(self.controller.active_tab_header().breadcrumb_count());
        let snapshot = self
            .controller
            .inspect_breadcrumb_values(
                prefix_len,
                field_shape.clone(),
                field_name.clone(),
                32 * Self::FRAME_WORK,
                2_048,
            )
            .await?;
        let inspected = snapshot.inspected();
        match BreadcrumbValuePickerTask::spawn(
            edit_index,
            field_shape.clone(),
            field_name.clone(),
            operator,
            snapshot,
            initial_value.clone(),
        ) {
            Ok(task) => {
                self.breadcrumb_value_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = format!(
                    "PickerTui is selecting values for {field_name} ({inspected} addresses inspected)."
                );
            }
            Err(message) => {
                self.breadcrumb_value_editor = Some(BreadcrumbValueEditorState {
                    edit_index,
                    field_shape,
                    field_name,
                    operator,
                    text: initial_value.unwrap_or_default(),
                });
                self.mode = BrowserMode::BreadcrumbValue;
                self.status =
                    format!("No reflected values available; enter a value manually: {message}");
            }
        }
        Ok(())
    }

    pub(super) async fn finish_breadcrumb_value_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .breadcrumb_value_picker_task
            .as_ref()
            .is_some_and(BreadcrumbValuePickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .breadcrumb_value_picker_task
            .take()
            .expect("a finished breadcrumb value picker task is present");
        let edit_index = task.edit_index;
        let field_shape = task.field_shape.clone();
        let field_name = task.field_name.clone();
        let operator = task.operator;
        let context_complete = task.context_complete;
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            BreadcrumbValuePickerOutcome::Selected(value) => {
                let breadcrumb = Breadcrumb::ValueFilter {
                    field_shape,
                    field_name,
                    operator,
                    value,
                };
                let update = match edit_index {
                    Some(index) => TabUpdate::ReplaceBreadcrumb { index, breadcrumb },
                    None => TabUpdate::PushBreadcrumb(breadcrumb),
                };
                self.controller.update_active_tab(update).await?;
                self.finish_query_change().await?;
                if !context_complete {
                    self.status.push_str(
                        " Values were bounded to the reflected values inspected at that query layer.",
                    );
                }
            }
            BreadcrumbValuePickerOutcome::Cancelled => {
                self.mode = BrowserMode::Pool;
                self.status = "Value filter selection cancelled.".to_owned();
            }
            BreadcrumbValuePickerOutcome::Failed(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Breadcrumb value PickerTui failed: {message}");
            }
        }
        Ok(())
    }

    pub(super) async fn finish_breadcrumb_menu_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .breadcrumb_menu_task
            .as_ref()
            .is_some_and(BreadcrumbMenuTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .breadcrumb_menu_task
            .take()
            .expect("a finished breadcrumb menu task is present");
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            BreadcrumbMenuOutcome::Selected(choice) => {
                self.accept_breadcrumb_choice(choice).await?;
            }
            BreadcrumbMenuOutcome::Cancelled => {
                self.mode = BrowserMode::Pool;
                self.status = "Breadcrumb selection cancelled.".to_owned();
            }
            BreadcrumbMenuOutcome::Failed(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Breadcrumb PickerTui failed: {message}");
            }
        }
        Ok(())
    }

    async fn launch_shape_breadcrumb_picker(
        &mut self,
        edit_index: Option<usize>,
        initially_included: Vec<String>,
    ) -> Result<(), ObjectBrowserControllerError> {
        let prefix_len =
            edit_index.unwrap_or(self.controller.active_tab_header().breadcrumb_count());
        let snapshot = self
            .controller
            .inspect_breadcrumb_context(prefix_len, Self::FRAME_WORK, 2_048)
            .await?;
        let inspected = snapshot.inspected();
        match BreadcrumbPickerTask::shapes(&snapshot, edit_index, &initially_included) {
            Ok(task) => {
                self.breadcrumb_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = format!(
                    "PickerTui is selecting shapes from breadcrumb prefix {prefix_len} ({inspected} addresses inspected)."
                );
            }
            Err(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Could not open shape picker: {message}");
            }
        }
        Ok(())
    }

    async fn launch_filter_field_breadcrumb_picker(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        let prefix_len = self.controller.active_tab_header().breadcrumb_count();
        let snapshot = self
            .controller
            .inspect_breadcrumb_context(prefix_len, Self::FRAME_WORK, 2_048)
            .await?;
        let inspected = snapshot.inspected();
        match BreadcrumbFilterFieldPickerTask::spawn(&snapshot) {
            Ok(task) => {
                self.breadcrumb_filter_field_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = format!(
                    "PickerTui is selecting a filter field from breadcrumb prefix {prefix_len} ({inspected} addresses inspected)."
                );
            }
            Err(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Could not open filter field picker: {message}");
            }
        }
        Ok(())
    }

    async fn launch_field_breadcrumb_picker(
        &mut self,
        edit_index: Option<usize>,
        mode: crate::object_explorer::ProjectFieldsMode,
        initially_included: Vec<crate::object_explorer::ProjectedField>,
    ) -> Result<(), ObjectBrowserControllerError> {
        let prefix_len =
            edit_index.unwrap_or(self.controller.active_tab_header().breadcrumb_count());
        let snapshot = self
            .controller
            .inspect_breadcrumb_context(prefix_len, Self::FRAME_WORK, 2_048)
            .await?;
        let inspected = snapshot.inspected();
        match BreadcrumbPickerTask::fields(&snapshot, edit_index, mode, &initially_included) {
            Ok(task) => {
                self.breadcrumb_picker_task = Some(task);
                self.mode = BrowserMode::NestedPicker;
                self.status = format!(
                    "PickerTui is selecting fields from breadcrumb prefix {prefix_len} ({inspected} addresses inspected)."
                );
            }
            Err(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Could not open field picker: {message}");
            }
        }
        Ok(())
    }

    pub(super) async fn finish_breadcrumb_filter_field_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .breadcrumb_filter_field_picker_task
            .as_ref()
            .is_some_and(BreadcrumbFilterFieldPickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .breadcrumb_filter_field_picker_task
            .take()
            .expect("a finished breadcrumb filter field picker task is present");
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            BreadcrumbFilterFieldPickerOutcome::Selected {
                field_shape,
                field_name,
                operator,
            } => {
                self.launch_breadcrumb_value_picker(None, field_shape, field_name, operator, None)
                    .await?;
            }
            BreadcrumbFilterFieldPickerOutcome::Cancelled => {
                self.mode = BrowserMode::Pool;
                self.status = "Filter field selection cancelled.".to_owned();
            }
            BreadcrumbFilterFieldPickerOutcome::Failed(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Filter field PickerTui failed: {message}");
            }
        }
        Ok(())
    }
    pub(super) async fn finish_breadcrumb_picker_task(
        &mut self,
    ) -> Result<(), ObjectBrowserControllerError> {
        if !self
            .breadcrumb_picker_task
            .as_ref()
            .is_some_and(BreadcrumbPickerTask::is_finished)
        {
            return Ok(());
        }
        let task = self
            .breadcrumb_picker_task
            .take()
            .expect("a finished breadcrumb picker task is present");
        let target = task.target().clone();
        let context_complete = task.context_complete();
        match task
            .finish()
            .await
            .map_err(ObjectBrowserControllerError::Engine)?
        {
            BreadcrumbPickerOutcome::Cancelled => {
                self.mode = BrowserMode::Pool;
                self.status = "Breadcrumb selection cancelled.".to_owned();
            }
            BreadcrumbPickerOutcome::Failed(message) => {
                self.mode = BrowserMode::Pool;
                self.status = format!("Breadcrumb PickerTui failed: {message}");
            }
            BreadcrumbPickerOutcome::Selected(selected) => {
                let breadcrumb = match target {
                    BreadcrumbPickerTarget::Shapes { .. } => {
                        let mut shapes = selected
                            .into_iter()
                            .filter_map(|value| match value {
                                BreadcrumbPickerValue::Shape(shape) => Some(shape),
                                BreadcrumbPickerValue::Field(_) => None,
                            })
                            .collect::<Vec<_>>();
                        shapes.sort();
                        shapes.dedup();
                        Breadcrumb::ShapeFilter {
                            included_shapes: shapes,
                        }
                    }
                    BreadcrumbPickerTarget::Fields { mode, .. } => {
                        let mut fields = selected
                            .into_iter()
                            .filter_map(|value| match value {
                                BreadcrumbPickerValue::Field(field) => Some(field),
                                BreadcrumbPickerValue::Shape(_) => None,
                            })
                            .collect::<Vec<_>>();
                        fields.sort();
                        fields.dedup();
                        Breadcrumb::ProjectFields {
                            mode,
                            included_fields: fields,
                        }
                    }
                };
                let edit_index = match target {
                    BreadcrumbPickerTarget::Shapes { edit_index }
                    | BreadcrumbPickerTarget::Fields { edit_index, .. } => edit_index,
                };
                let update = match edit_index {
                    Some(index) => TabUpdate::ReplaceBreadcrumb { index, breadcrumb },
                    None => TabUpdate::PushBreadcrumb(breadcrumb),
                };
                self.controller.update_active_tab(update).await?;
                self.finish_query_change().await?;
                if !context_complete {
                    self.status.push_str(
                        " Choices were bounded to the reflected values inspected at that query layer.",
                    );
                }
            }
        }
        Ok(())
    }

    async fn handle_breadcrumb_value_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.breadcrumb_value_editor = None;
                self.mode = BrowserMode::Pool;
            }
            KeyCode::Backspace => {
                if let Some(editor) = self.breadcrumb_value_editor.as_mut() {
                    editor.text.pop();
                }
            }
            KeyCode::Enter => {
                let Some(editor) = self.breadcrumb_value_editor.take() else {
                    return Ok(());
                };
                let breadcrumb = Breadcrumb::ValueFilter {
                    field_shape: editor.field_shape,
                    field_name: editor.field_name,
                    operator: editor.operator,
                    value: editor.text,
                };
                let update = match editor.edit_index {
                    Some(index) => TabUpdate::ReplaceBreadcrumb { index, breadcrumb },
                    None => TabUpdate::PushBreadcrumb(breadcrumb),
                };
                self.controller.update_active_tab(update).await?;
                self.finish_query_change().await?;
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(editor) = self.breadcrumb_value_editor.as_mut() {
                    editor.text.push(character);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn remove_last_breadcrumb(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let count = self.controller.active_tab_header().breadcrumb_count();
        if count == 0 {
            self.status = "The active tab has no breadcrumb to remove.".to_owned();
            return Ok(());
        }
        self.remove_breadcrumb(count - 1).await
    }

    async fn remove_breadcrumb(
        &mut self,
        index: usize,
    ) -> Result<(), ObjectBrowserControllerError> {
        self.controller
            .update_active_tab(TabUpdate::RemoveBreadcrumb(index))
            .await?;
        self.finish_query_change().await
    }

    async fn finish_query_change(&mut self) -> Result<(), ObjectBrowserControllerError> {
        self.active_root = None;
        self.breadcrumb_focus = None;
        self.mode = BrowserMode::Pool;
        self.controller
            .refresh_window(Self::FRAME_WORK, self.max_cards, self.max_relationship_rows)
            .await?;
        self.status = format!(
            "Tab slot {} now has {} lazy breadcrumb operations.",
            self.controller.active_tab_header().slot(),
            self.controller.active_tab_header().breadcrumb_count()
        );
        Ok(())
    }

    async fn handle_tab_name_key(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.tab_name_editor = None;
                self.mode = BrowserMode::Pool;
            }
            KeyCode::Backspace => {
                if let Some(name) = self.tab_name_editor.as_mut() {
                    name.pop();
                }
            }
            KeyCode::Enter => {
                let Some(name) = self.tab_name_editor.take() else {
                    return Ok(());
                };
                self.controller
                    .update_active_tab(TabUpdate::Rename(name))
                    .await?;
                self.mode = BrowserMode::Pool;
                self.status = format!(
                    "Renamed tab slot {} to {}.",
                    self.controller.active_tab_header().slot(),
                    self.controller.active_tab_header().name()
                );
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(name) = self.tab_name_editor.as_mut() {
                    name.push(character);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_text_key(&mut self, key: KeyEvent) -> Result<(), ObjectBrowserControllerError> {
        match key.code {
            KeyCode::Esc => {
                self.text_editor = None;
                self.mode = BrowserMode::Pool;
            }
            KeyCode::Backspace => {
                if let Some(editor) = self.text_editor.as_mut() {
                    editor.text.pop();
                }
            }
            KeyCode::Enter => self.accept_text().await?,
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(editor) = self.text_editor.as_mut() {
                    editor.text.push(character);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn accept_text(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let Some(editor) = self.text_editor.take() else {
            return Ok(());
        };
        let transition = self
            .controller
            .set_scalar_text(editor.slot, editor.text)
            .await?;
        self.active_root = Some(
            self.controller
                .inspect_root(editor.slot, self.max_relationship_rows)
                .await?,
        );
        self.mode = BrowserMode::Pool;
        if transition == BuilderTransition::Ready
            && self
                .pending_link
                .as_ref()
                .is_some_and(|link| link.source == ValueAddress::root(editor.slot))
        {
            self.open_pending_link_actions().await?;
        }
        Ok(())
    }

    fn focus_next_builder_input(&mut self) -> Result<(), ObjectBrowserControllerError> {
        let key = self.active_root.as_ref().and_then(|root| {
            let builder = root.builder()?;
            match builder.kind() {
                BuilderKindSnapshot::ShapeUnset => Some(CardRowKey::Shape),
                BuilderKindSnapshot::Enum {
                    selected_variant: None,
                    ..
                } => Some(CardRowKey::Variant),
                BuilderKindSnapshot::Scalar {
                    value_is_set: false,
                } => Some(CardRowKey::Value),
                _ => builder
                    .fields()
                    .iter()
                    .find(|field| field.binding() == &FieldBindingSnapshot::Unset)
                    .map(|field| CardRowKey::Field(field.name().to_owned())),
            }
        });
        self.controller.focus_row(key)
    }

    fn focus_after_builder_transition(
        &mut self,
        _slot: SlotId,
        transition: BuilderTransition,
    ) -> Result<(), ObjectBrowserControllerError> {
        if transition == BuilderTransition::Ready {
            // A completed builder is now a real value. Keep the value card and
            // its shape row selected instead of asking for another builder row
            // that no longer exists.
            self.controller.focus_row(Some(CardRowKey::Shape))
        } else {
            self.focus_next_builder_input()
        }
    }
}

fn resize_card_breadth(current: u16, minimum: u16, main_axis_extent: u16, direction: isize) -> u16 {
    if main_axis_extent == 0 {
        return resize_dimension(current, minimum, direction);
    }
    let visible_cards = (main_axis_extent / current.max(1)).max(1);
    let target_cards = if direction < 0 {
        visible_cards.saturating_add(1)
    } else {
        visible_cards.saturating_sub(1).max(1)
    };
    let snapped = (main_axis_extent / target_cards).max(minimum);
    if snapped == current {
        resize_dimension(current, minimum, direction)
    } else {
        snapped
    }
}

fn resize_dimension(current: u16, minimum: u16, direction: isize) -> u16 {
    if direction < 0 {
        current.saturating_sub(1).max(minimum)
    } else {
        current.saturating_add(1).max(minimum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::ArenaQueryContext;
    use crate::object_explorer::ExplorerEngine;
    use crate::object_explorer::FieldCandidateAction;
    use crate::object_explorer::ProduceJsonRequest;
    use crate::object_explorer::RootLifecycleSnapshot;
    use crate::object_explorer::Tab;
    use arbitrary::Arbitrary;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use cloud_terrastodon_registry::ProductionKind;
    use cloud_terrastodon_registry::functions_from;
    use facet::Facet;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::future::Future;
    use std::future::IntoFuture;
    use std::pin::Pin;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct BrowserProducerRequest {
        marker: u8,
    }

    #[derive(Arbitrary, Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct BrowserProducedValue {
        marker: u8,
    }

    #[derive(Clone, Debug, Eq, Facet, PartialEq)]
    #[repr(C)]
    struct BrowserProductionDestination {
        value: BrowserProducedValue,
    }

    impl IntoFuture for BrowserProducerRequest {
        type Output = eyre::Result<BrowserProducedValue>;
        type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(async move {
                Ok(BrowserProducedValue {
                    marker: self.marker,
                })
            })
        }
    }

    cloud_terrastodon_registry::register_thing!(BrowserProducerRequest);
    cloud_terrastodon_registry::register_thing!(BrowserProducedValue);
    cloud_terrastodon_registry::register_arbitrary!(BrowserProducedValue);
    cloud_terrastodon_registry::register_into_future!(
        BrowserProducerRequest => BrowserProducedValue
    );

    fn browser_producer() -> &'static cloud_terrastodon_registry::Function {
        functions_from(BrowserProducerRequest::SHAPE)
            .into_iter()
            .find(|function| {
                function.production_kind(BrowserProducedValue::SHAPE) == Some(ProductionKind::Exact)
            })
            .unwrap()
    }

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn modified(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    #[tokio::test]
    async fn typing_in_the_pool_starts_nucleo_row_search_on_semantic_rows() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let (destination, _) = app
                .controller
                .create_builder(BrowserProductionDestination::SHAPE)
                .await
                .unwrap();
            app.controller
                .focus(CardAddress::Value(ValueAddress::root(destination)))
                .unwrap();
            app.active_root = Some(
                app.controller
                    .inspect_root(destination, app.max_relationship_rows)
                    .await
                    .unwrap(),
            );

            app.handle_event(&key(KeyCode::Char('v'))).await.unwrap();

            assert_eq!(app.mode, BrowserMode::RowSearch);
            assert_eq!(
                app.row_search.as_ref().and_then(|search| search.selected()),
                Some(&CardRowKey::Field("value".to_owned()))
            );
            assert_eq!(
                app.controller.active_state().unwrap().focused_row(),
                Some(&CardRowKey::Field("value".to_owned()))
            );
            assert_eq!(app.controller.active_state().unwrap().search_query(), "v");

            app.handle_event(&key(KeyCode::Esc)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Pool);
            assert!(
                app.controller
                    .active_state()
                    .unwrap()
                    .search_query()
                    .is_empty()
            );
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn shift_home_focuses_the_first_card() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            app.tick().await.unwrap();
            let first = app
                .controller
                .window()
                .and_then(|window| window.cards().first())
                .map(|card| card.address().clone())
                .expect("the initial query has a first card");

            app.handle_event(&modified(KeyCode::Home, KeyModifiers::SHIFT))
                .await
                .unwrap();

            assert_eq!(app.selected_address(), first);
            assert_eq!(
                app.controller.active_state().unwrap().viewport_anchor(),
                &first
            );
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn alt_plus_and_minus_resize_the_active_card_axis() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let area = ratatui::layout::Rect::new(0, 0, 120, 24);
            app.configure_viewport(area);
            let original = app.max_cards;

            app.handle_event(&modified(KeyCode::Char('+'), KeyModifiers::ALT))
                .await
                .unwrap();
            app.configure_viewport(area);
            assert!(
                app.max_cards < original,
                "wider horizontal cards should reduce the visible card count"
            );

            app.handle_event(&modified(KeyCode::Char('-'), KeyModifiers::ALT))
                .await
                .unwrap();
            app.configure_viewport(area);
            assert_eq!(app.max_cards, original);
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn up_from_the_pool_focuses_existing_breadcrumbs_for_editing() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(8);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            app.controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec!["String".to_owned()],
                }))
                .await
                .unwrap();

            app.handle_event(&key(KeyCode::Up)).await.unwrap();
            assert!(
                app.breadcrumb_focus.is_some_and(|focus| focus.is_add(1)),
                "Up enters the breadcrumb bar at +Add Breadcrumb"
            );
            app.handle_event(&key(KeyCode::Left)).await.unwrap();
            assert_eq!(
                app.breadcrumb_focus.and_then(|focus| focus.operation(1)),
                Some(0)
            );

            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            assert!(
                terminal
                    .backend()
                    .to_string()
                    .contains("Everything > [shape String] > +Add Breadcrumb")
            );

            app.handle_event(&key(KeyCode::Down)).await.unwrap();
            assert!(app.breadcrumb_focus.is_none());
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn entering_a_filtered_child_opens_an_exact_projection_tab_without_scanning() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let source_tab = app.controller.active_tab().unwrap();
            let root = app
                .controller
                .insert_owned(BrowserProductionDestination {
                    value: BrowserProducedValue { marker: 7 },
                })
                .await
                .unwrap();
            let shape =
                cloud_terrastodon_registry::describe_shape(BrowserProductionDestination::SHAPE);
            app.controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec![shape.to_owned()],
                }))
                .await
                .unwrap();
            app.finish_query_change().await.unwrap();
            let root_address = ValueAddress::root(root);
            assert_eq!(
                app.controller.active_state().unwrap().selection(),
                &CardAddress::Value(root_address.clone())
            );
            app.controller
                .focus_row(Some(CardRowKey::Field("value".to_owned())))
                .unwrap();

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();

            let target = root_address.child(crate::object_explorer::ValuePathSegment::Field(
                "value".to_owned(),
            ));
            let projection_tab = app.controller.active_tab().unwrap();
            assert_ne!(projection_tab, source_tab);
            assert_eq!(
                app.controller.active_state().unwrap().selection(),
                &CardAddress::Value(target.clone())
            );
            assert!(!app.controller.is_scanning());
            assert!(app.controller.window().unwrap().cards().iter().any(|card| {
                matches!(
                    card.address(),
                    CardAddress::Value(address) if address == &target
                )
            }));
            assert!(app.status.contains("source tab"));

            assert!(app.controller.switch_tab_previous().await.unwrap());
            assert_eq!(app.controller.active_tab(), Some(source_tab));
            assert_eq!(
                app.controller.active_state().unwrap().selection(),
                &CardAddress::Value(root_address)
            );
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn revisited_building_root_fields_still_activate() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            app.tick().await.unwrap();
            let (destination, transition) = app
                .controller
                .create_builder(BrowserProductionDestination::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            app.controller
                .focus(CardAddress::Value(ValueAddress::root(destination)))
                .unwrap();
            app.controller
                .focus_row(Some(CardRowKey::Field("value".to_owned())))
                .unwrap();
            app.active_root = None;
            app.controller
                .refresh_window(
                    ObjectBrowserApp::FRAME_WORK,
                    app.max_cards,
                    app.max_relationship_rows,
                )
                .await
                .unwrap();

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();

            assert_eq!(app.mode, BrowserMode::Value);
            assert_eq!(app.value_picker.as_ref().unwrap().field_name(), "value");
            assert_eq!(app.active_root.as_ref().unwrap().slot(), destination);
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn generic_default_producer_picker_drives_the_engine_job_to_a_ready_field() {
        let _ = browser_producer();
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let (destination, transition) = app
                .controller
                .create_builder(BrowserProductionDestination::SHAPE)
                .await
                .unwrap();
            assert_eq!(transition, BuilderTransition::Building);
            app.controller
                .focus(CardAddress::Value(ValueAddress::root(destination)))
                .unwrap();
            app.active_root = Some(
                app.controller
                    .inspect_root(destination, app.max_relationship_rows)
                    .await
                    .unwrap(),
            );
            app.focus_next_builder_input().unwrap();

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Value);
            let selected = app.value_picker.as_ref().unwrap().selected().unwrap();
            assert!(
                selected
                    .label()
                    .contains("invoke default BrowserProducerRequest")
            );
            assert!(matches!(
                selected.choice(),
                ValuePickerChoice::InvokeDefaultProducer { function }
                    if std::ptr::eq(*function, browser_producer())
            ));

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Pool);
            assert!(app.production_jobs_active);
            assert_eq!(app.active_root.as_ref().unwrap().slot(), destination);
            assert!(matches!(
                app.active_root.as_ref().unwrap().lifecycle(),
                RootLifecycleSnapshot::Building
            ));

            for _ in 0..64 {
                app.tick().await.unwrap();
                if !app.production_jobs_active {
                    break;
                }
                tokio::task::yield_now().await;
            }
            assert!(!app.production_jobs_active);
            let destination_snapshot = app
                .controller
                .inspect_root(destination, app.max_relationship_rows)
                .await
                .unwrap();
            assert_eq!(
                destination_snapshot.lifecycle(),
                &RootLifecycleSnapshot::Ready
            );
            assert!(app.status.contains("moved output slot"));
            app.close().await.unwrap();
            destination
        };

        let (engine, destination) = tokio::join!(engine.run(inbox), client);
        assert!(engine.arena().ready_value(destination).is_some());
        assert_eq!(engine.active_production_count(), 0);
        assert_eq!(engine.arena().allocated_slot_count(), 5);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn produce_json_uses_generic_owned_field_picker_with_owner_provenance() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            app.tick().await.unwrap();

            app.handle_event(&modified(KeyCode::End, KeyModifiers::SHIFT))
                .await
                .unwrap();
            assert_eq!(app.selected_address(), CardAddress::NewSlot);
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            let request_slot = app.active_root.as_ref().unwrap().slot();
            app.controller
                .set_builder_shape(request_slot, ProduceJsonRequest::SHAPE)
                .await
                .unwrap();
            app.active_root = Some(
                app.controller
                    .inspect_root(request_slot, app.max_relationship_rows)
                    .await
                    .unwrap(),
            );
            app.mode = BrowserMode::Pool;
            app.focus_next_builder_input().unwrap();
            assert_eq!(app.mode, BrowserMode::Pool);
            assert_eq!(
                app.controller.active_state().unwrap().focused_row(),
                Some(&CardRowKey::Field("breadcrumbs".to_owned()))
            );

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Value);
            let breadcrumbs_row = app.value_picker.as_ref().unwrap().selected().unwrap();
            assert!(
                breadcrumbs_row
                    .label()
                    .contains("field breadcrumbs of slot")
            );
            assert!(breadcrumbs_row.label().contains("(Tab)"));
            assert!(matches!(
                breadcrumbs_row.choice(),
                ValuePickerChoice::Candidate { .. }
            ));
            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains("Pick Object for breadcrumbs"));
            assert!(rendered.contains("field breadcrumbs of slot"));
            assert!(rendered.contains("(Tab)"));

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::LinkAction);
            assert_eq!(
                app.link_action_picker
                    .as_ref()
                    .unwrap()
                    .consequences()
                    .iter()
                    .map(|consequence| consequence.action())
                    .collect::<Vec<_>>(),
                [FieldCandidateAction::Clone]
            );
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Pool);
            assert_eq!(app.active_root.as_ref().unwrap().slot(), request_slot);
            assert_eq!(
                app.controller.active_state().unwrap().focused_row(),
                Some(&CardRowKey::Field("filename".to_owned()))
            );

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Value);
            let filename_row = app.value_picker.as_ref().unwrap().selected().unwrap();
            assert_eq!(filename_row.label(), "+ create new owned String");
            assert!(matches!(
                filename_row.choice(),
                ValuePickerChoice::CreateNewOwned { shape }
                    if shape.is_shape(String::SHAPE)
            ));
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Text);

            for character in "admins.json".chars() {
                app.handle_event(&key(KeyCode::Char(character)))
                    .await
                    .unwrap();
            }
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::LinkAction);
            assert_eq!(
                app.link_action_picker
                    .as_ref()
                    .unwrap()
                    .consequences()
                    .iter()
                    .map(|consequence| consequence.action())
                    .collect::<Vec<_>>(),
                [FieldCandidateAction::Move, FieldCandidateAction::Clone]
            );
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();

            assert_eq!(app.mode, BrowserMode::Pool);
            let request = app.active_root.as_ref().unwrap();
            assert_eq!(request.slot(), request_slot);
            assert_eq!(request.lifecycle(), &RootLifecycleSnapshot::Ready);
            assert_eq!(request.card().shape(), "ProduceJsonRequest");
            assert!(request.builder().is_none());
            assert_eq!(
                app.controller.active_state().unwrap().focused_row(),
                Some(&CardRowKey::Shape),
                "finalizing the last field keeps the completed value visibly focused"
            );
            app.close().await.unwrap();
        };

        let (engine, ()) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.arena().allocated_slot_count(), 3);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn breadcrumb_and_tab_controls_update_ordinary_arena_tabs() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let first = app.controller.active_tab().unwrap();
            app.tick().await.unwrap();

            assert_eq!(app.selected_address(), CardAddress::NewSlot);
            app.controller
                .focus(CardAddress::Value(ValueAddress::root(first)))
                .unwrap();
            app.handle_event(&key(KeyCode::Right)).await.unwrap();
            assert!(matches!(
                app.selected_address(),
                CardAddress::Value(ref address)
                    if address
                        .path()
                        .segments()
                        .last()
                        == Some(&crate::object_explorer::ValuePathSegment::Field(
                            "name".to_owned()
                        ))
            ));

            let selected = app.selected_card().and_then(|card| {
                let CardAddress::Value(address) = card.address() else {
                    return None;
                };
                Some((address, card.shape()))
            });
            let picker = BreadcrumbPicker::new(selected, &[], None);
            assert_eq!(picker.selected().unwrap().label(), "filter shapes…");
            let value_filter = picker
                .rows()
                .iter()
                .find(|row| row.label() == "filter name equals …")
                .unwrap()
                .choice()
                .clone();
            app.accept_breadcrumb_choice(value_filter).await.unwrap();
            assert_eq!(app.mode, BrowserMode::BreadcrumbValue);
            for character in "unnamed".chars() {
                app.handle_event(&key(KeyCode::Char(character)))
                    .await
                    .unwrap();
            }
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.mode, BrowserMode::Pool);
            assert_eq!(app.controller.active_tab_header().breadcrumb_count(), 1);
            assert_eq!(
                app.controller.active_tab_header().breadcrumb_labels(),
                ["name = unnamed"]
            );

            app.handle_event(&key(KeyCode::F(2))).await.unwrap();
            assert_eq!(app.mode, BrowserMode::TabName);
            for _ in 0.."unnamed".len() {
                app.handle_event(&key(KeyCode::Backspace)).await.unwrap();
            }
            for character in "admins".chars() {
                app.handle_event(&key(KeyCode::Char(character)))
                    .await
                    .unwrap();
            }
            app.handle_event(&key(KeyCode::Enter)).await.unwrap();
            assert_eq!(app.controller.active_tab_header().name(), "admins");

            let backend = TestBackend::new(120, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(&format!("slot {first} - Tab 1 of 1 - admins")));
            assert!(rendered.contains("Everything > name = unnamed > +Add Breadcrumb"));

            app.handle_event(&modified(KeyCode::Char(']'), KeyModifiers::SHIFT))
                .await
                .unwrap();
            let second = app.controller.active_tab().unwrap();
            assert_ne!(second, first);
            assert_eq!(app.controller.active_tab_header().name(), "unnamed");
            assert_eq!(app.controller.active_tab_header().breadcrumb_count(), 0);

            app.handle_event(&modified(KeyCode::Char('['), KeyModifiers::SHIFT))
                .await
                .unwrap();
            assert_eq!(app.controller.active_tab(), Some(first));
            assert_eq!(app.controller.active_tab_header().name(), "admins");
            assert_eq!(app.controller.active_tab_header().breadcrumb_count(), 1);

            app.handle_event(&modified(KeyCode::Backspace, KeyModifiers::CONTROL))
                .await
                .unwrap();
            assert_eq!(app.controller.active_tab_header().breadcrumb_count(), 0);

            app.close().await.unwrap();
            (first, second)
        };

        let (engine, (first, second)) = tokio::join!(engine.run(inbox), client);
        let clone_tab = |slot| {
            engine
                .arena()
                .ready_value(slot)
                .unwrap()
                .try_clone()
                .unwrap()
                .into_box::<Tab>()
                .unwrap()
                .downcast::<Tab>()
                .unwrap()
        };
        let first_tab = clone_tab(first);
        let second_tab = clone_tab(second);
        assert_eq!(first_tab.name(), "admins");
        assert!(first_tab.breadcrumbs().is_empty());
        assert_eq!(second_tab.name(), "unnamed");
        assert!(second_tab.breadcrumbs().is_empty());
        assert_eq!(engine.arena().allocated_slot_count(), 2);
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn invoke_arbitrary_creates_a_plan_and_focuses_an_output_projection_tab() {
        let _ = browser_producer();
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let request_tab = app.controller.active_tab().unwrap();
            let request = app
                .controller
                .insert_owned(BrowserProducerRequest { marker: 42 })
                .await
                .unwrap();
            app.controller
                .focus(CardAddress::Value(ValueAddress::root(request)))
                .unwrap();
            let snapshot = app
                .controller
                .inspect_root(request, app.max_relationship_rows)
                .await
                .unwrap();
            let action = snapshot
                .card()
                .rows()
                .iter()
                .find(|row| {
                    matches!(
                        row.content(),
                        CardRowContent::RootAction(action)
                            if action.arbitrary_invocation().is_some()
                    )
                })
                .expect("an arbitrary output constructor adds a request action")
                .key()
                .clone();
            app.active_root = Some(snapshot);
            app.controller.focus_row(Some(action)).unwrap();

            app.handle_event(&key(KeyCode::Enter)).await.unwrap();

            let output_tab = app.controller.active_tab().unwrap();
            let output = app.active_root.as_ref().unwrap().slot();
            assert_ne!(output_tab, request_tab);
            assert_ne!(output, request);
            assert_eq!(app.controller.active_tab_header().breadcrumb_count(), 1);
            assert_eq!(
                app.controller.active_state().unwrap().selection(),
                &CardAddress::Value(ValueAddress::root(output))
            );
            assert!(app.status.contains("via arbitrary plan"));
            assert!(app.status.contains("in a new tab"));

            assert!(app.controller.switch_tab_previous().await.unwrap());
            assert_eq!(app.controller.active_tab(), Some(request_tab));
            assert_eq!(
                app.controller.active_state().unwrap().selection(),
                &CardAddress::Value(ValueAddress::root(request)),
                "the request tab retains its logical selection"
            );
            app.close().await.unwrap();
            (request, output)
        };

        let (engine, (request, output)) = tokio::join!(engine.run(inbox), client);
        assert_eq!(engine.invocation_plan_count(), 1);
        assert!(engine.arena().ready_value(request).is_some());
        assert!(engine.arena().ready_value(output).is_some());
        assert_eq!(
            engine
                .arena()
                .ready_slot_ids()
                .filter(|slot| {
                    engine
                        .arena()
                        .ready_value(*slot)
                        .is_some_and(|value| value.shape().is_shape(ArbitraryBytes::SHAPE))
                })
                .count(),
            1,
            "the arbitrary plan retains an explicit byte-source root"
        );
        assert_eq!(engine.json_serialization_count(), 0);
    }

    #[tokio::test]
    async fn escape_pops_breadcrumbs_then_requires_three_presses_to_exit() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            app.controller
                .update_active_tab(TabUpdate::PushBreadcrumb(Breadcrumb::ShapeFilter {
                    included_shapes: vec!["String".to_owned()],
                }))
                .await
                .unwrap();

            app.handle_event(&key(KeyCode::Esc)).await.unwrap();
            assert_eq!(app.controller.active_tab_header().breadcrumb_count(), 0);
            assert!(!app.should_quit);
            assert!(app.recent_escape_presses.is_empty());

            app.handle_event(&key(KeyCode::Esc)).await.unwrap();
            assert!(!app.should_quit);
            assert!(app.status.contains("2 more times"));
            app.handle_event(&key(KeyCode::Esc)).await.unwrap();
            assert!(!app.should_quit);
            assert!(app.status.contains("1 more time"));
            app.handle_event(&key(KeyCode::Esc)).await.unwrap();
            assert!(app.should_quit);
            app.close().await.unwrap();
        };

        let (_engine, ()) = tokio::join!(engine.run(inbox), client);
    }
}
