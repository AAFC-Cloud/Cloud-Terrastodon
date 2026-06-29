use cloud_terrastodon_registry::KnownShapeInfo;
use cloud_terrastodon_registry::ShapeFieldInfo;
use cloud_terrastodon_registry::ShapeVariantInfo;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::known_shapes;
use cloud_terrastodon_registry::shape_fields_for_thing;
use cloud_terrastodon_registry::shape_variants_for_thing;
use crossterm::event::EventStream;
use eyre::Result;
use futures::FutureExt;
use futures::StreamExt;
use nucleo::Matcher;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEvent;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::Color;
use ratatui::prelude::Modifier;
use ratatui::prelude::Style;
use ratatui::prelude::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use serde_json::Map;
use serde_json::Value;
use std::collections::BTreeSet;
use std::ops::Range;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::info;
use tui_textarea::CursorMove;
use tui_textarea::TextArea;

pub async fn ui_main() -> Result<()> {
    info!("Starting object browser");
    let terminal = ratatui::init();
    let app_result = ObjectBrowserApp::default().run(terminal).await;
    ratatui::restore();
    app_result
}

struct ObjectBrowserApp {
    should_quit: bool,
    mode: UiMode,
    shape_picker: ShapePickerState,
    variant_picker: Option<VariantPickerState>,
    field_picker: Option<FieldPickerState>,
    link_action_picker: Option<LinkActionPickerState>,
    rename_slot: Option<RenameSlotState>,
    shape_choices: Vec<KnownShapeInfo>,
    object_slots: Vec<ObjectSlot>,
    active_slot_index: usize,
    active_row_index: usize,
    next_slot_id: usize,
    status_message: String,
}

impl Default for ObjectBrowserApp {
    fn default() -> Self {
        let shape_choices = known_shapes();
        let shape_picker = ShapePickerState::new(&shape_choices);

        Self {
            should_quit: false,
            mode: UiMode::Pool,
            shape_picker,
            variant_picker: None,
            field_picker: None,
            link_action_picker: None,
            rename_slot: None,
            shape_choices,
            object_slots: Vec::new(),
            active_slot_index: 0,
            active_row_index: 0,
            next_slot_id: 1,
            status_message: "Left/Right: slots | Up/Down: rows | Enter/Space: act | q: quit"
                .to_string(),
        }
    }
}

impl ObjectBrowserApp {
    const FRAMES_PER_SECOND: f32 = 60.0;
    const MIN_SLOT_WIDTH: u16 = 34;

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { self.advance_pending_invocations(); terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }

        Ok(())
    }

    fn advance_pending_invocations(&mut self) {
        let mut updates = Vec::new();
        for slot_index in 0..self.object_slots.len() {
            let is_finished = matches!(
                self.object_slots.get(slot_index).and_then(|slot| slot.runtime_state.as_ref()),
                Some(SlotRuntimeState::Pending(pending)) if pending.join_handle.is_finished()
            );
            if !is_finished {
                continue;
            }

            let Some(slot) = self.object_slots.get_mut(slot_index) else {
                continue;
            };
            let Some(SlotRuntimeState::Pending(pending)) = slot.runtime_state.take() else {
                continue;
            };

            let next_state = match pending
                .join_handle
                .now_or_never()
                .expect("finished join handle should resolve immediately")
            {
                Ok(Ok(output)) => match (pending.output_serialize)(output.as_ref()) {
                    Ok(json) => SlotRuntimeState::ResolvedValue { json },
                    Err(error) => SlotRuntimeState::Failed {
                        message: format!("could not serialize invocation result: {error}"),
                    },
                },
                Ok(Err(error)) => SlotRuntimeState::Failed {
                    message: error.to_string(),
                },
                Err(error) => SlotRuntimeState::Failed {
                    message: format!("task join failed: {error}"),
                },
            };
            updates.push((slot.id, next_state));
        }

        for (slot_id, next_state) in &updates {
            let status_message = match next_state {
                SlotRuntimeState::ResolvedValue { .. } => {
                    format!("Result slot {slot_id} resolved.")
                }
                SlotRuntimeState::Failed { message } => {
                    format!("Result slot {slot_id} failed: {message}")
                }
                SlotRuntimeState::Pending(_) => continue,
            };
            if let Some(slot) = self.slot_by_id_mut(*slot_id) {
                slot.runtime_state = Some(match next_state {
                    SlotRuntimeState::ResolvedValue { json } => {
                        SlotRuntimeState::ResolvedValue { json: json.clone() }
                    }
                    SlotRuntimeState::Failed { message } => {
                        SlotRuntimeState::Failed { message: message.clone() }
                    }
                    SlotRuntimeState::Pending(_) => continue,
                });
            }
            self.status_message = status_message;
        }
        if !updates.is_empty() {
            self.invalidate_all_slot_display_caches();
        }
    }
    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [title_area, body_area, status_area] = vertical.areas(frame.area());

        let title = Line::from("Cloud Terrastodon Object Pool")
            .centered()
            .bold();
        frame.render_widget(title, title_area);

        self.draw_pool(frame, body_area);

        frame.render_widget(Line::from(self.status_message.as_str()), status_area);

        match self.mode {
            UiMode::Pool => {}
            UiMode::ShapePicker => self.draw_shape_picker_popup(frame),
            UiMode::VariantPicker => self.draw_variant_picker_popup(frame),
            UiMode::FieldPicker => self.draw_field_picker_popup(frame),
            UiMode::LinkActionPicker => self.draw_link_action_picker_popup(frame),
            UiMode::RenameSlot => self.draw_rename_slot_popup(frame),
        }
    }

    fn draw_pool(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Object Pool");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let [cards_area, scrollbar_area] = if inner.height > 1 {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(inner)
        } else {
            [inner, Rect::default()]
        };
        if cards_area.width == 0 || cards_area.height == 0 {
            return;
        }

        let visible = self.visible_slot_range(cards_area.width);
        let constraints = vec![Constraint::Fill(1); visible.len()];
        let slot_areas = Layout::horizontal(constraints).split(cards_area);

        for (offset, slot_index) in visible.clone().enumerate() {
            let Some(slot_area) = slot_areas.get(offset).copied() else {
                break;
            };
            let is_active = slot_index == self.active_slot_index;
            if slot_index == self.pseudo_slot_index() {
                self.draw_new_slot(frame, slot_area, is_active);
            } else if let Some(slot) = self.object_slots.get(slot_index) {
                self.draw_object_slot(frame, slot_area, slot.id, is_active);
            }
        }

        let max_visible = self.max_visible_slots(cards_area.width);
        if scrollbar_area.height > 0 && self.total_slot_count() > max_visible {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
            let mut scrollbar_state = ScrollbarState::new(self.total_slot_count())
                .position(visible.start)
                .viewport_content_length(max_visible);
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }
    }

    fn draw_object_slot(&mut self, frame: &mut Frame, area: Rect, slot_id: usize, is_active: bool) {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return;
        };
        let slot_label = slot.name.as_deref().unwrap_or("unnamed");
        let title = match slot.kind {
            SlotKind::Owned => format!("slot {} ({slot_label}) [owned]", slot.id),
            SlotKind::View(_) => format!("slot {} ({slot_label}) [view]", slot.id),
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(self.slot_border_style(slot_id, is_active));
        let paragraph = Paragraph::new(self.slot_lines(slot_id, is_active, self.active_row_index))
            .block(block)
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    fn draw_new_slot(&self, frame: &mut Frame, area: Rect, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::LightTripleDashed)
            .title("new slot")
            .border_style(slot_border_style(Color::DarkGray, is_active));
        let line =
            selectable_plain_line("+ create object", is_active && self.active_row_index == 0);
        frame.render_widget(Paragraph::new(vec![line]).block(block), area);
    }

    fn draw_shape_picker_popup(&mut self, frame: &mut Frame) {
        let preview_lines = self.shape_picker.preview_lines(&self.shape_choices);
        let items = self.shape_picker.list_items();
        let total_count = self.shape_picker.labels.len();
        let search = &mut self.shape_picker.search;
        draw_picker_popup(
            frame,
            "Pick Shape",
            "Shape Preview",
            search,
            items,
            total_count,
            preview_lines,
        );
    }

    fn draw_variant_picker_popup(&mut self, frame: &mut Frame) {
        let Some(preview_lines) = self.variant_picker_preview_lines() else {
            return;
        };
        let Some((items, total_count)) = self
            .variant_picker
            .as_ref()
            .map(|picker| (picker.list_items(), picker.labels.len()))
        else {
            return;
        };
        let search = &mut self.variant_picker.as_mut().expect("picker exists").search;
        draw_picker_popup(
            frame,
            "Pick Variant",
            "Variant Preview",
            search,
            items,
            total_count,
            preview_lines,
        );
    }

    fn draw_field_picker_popup(&mut self, frame: &mut Frame) {
        let Some(preview_lines) = self.field_picker_preview_lines() else {
            return;
        };
        let Some((items, total_count)) = self
            .field_picker
            .as_ref()
            .map(|picker| (picker.list_items(), picker.labels.len()))
        else {
            return;
        };
        let search = &mut self.field_picker.as_mut().expect("picker exists").search;
        draw_picker_popup(
            frame,
            "Pick Object",
            "Object Preview",
            search,
            items,
            total_count,
            preview_lines,
        );
    }

    fn draw_link_action_picker_popup(&mut self, frame: &mut Frame) {
        let Some(link_action_picker) = self.link_action_picker.as_mut() else {
            return;
        };

        let area = centered_rect(58, 42, frame.area());
        frame.render_widget(Clear, area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title("Move or Clone")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = popup_block.inner(area);
        frame.render_widget(popup_block, area);

        let [list_area, preview_area] =
            Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
                .areas(inner);

        let list = List::new(link_action_picker.list_items())
            .block(Block::default().borders(Borders::ALL).title("Action"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Yellow));
        frame.render_stateful_widget(list, list_area, &mut link_action_picker.list_state);

        let preview = Paragraph::new(Text::from(self.link_action_preview_lines()))
            .block(Block::default().borders(Borders::ALL).title("Consequence"))
            .wrap(Wrap { trim: false });
        frame.render_widget(preview, preview_area);
    }

    fn draw_rename_slot_popup(&mut self, frame: &mut Frame) {
        let Some(rename_slot) = self.rename_slot.as_mut() else {
            return;
        };

        let area = centered_rect(52, 28, frame.area());
        frame.render_widget(Clear, area);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .title("Rename Slot")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = popup_block.inner(area);
        frame.render_widget(popup_block, area);

        let [hint_area, editor_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Length(3)]).areas(inner);

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!("slot {}", rename_slot.slot_id)),
                Line::from("Enter: save | Esc: cancel"),
            ]),
            hint_area,
        );

        rename_slot
            .textarea
            .set_block(Block::default().borders(Borders::ALL).title("Name"));
        rename_slot.textarea.render(editor_area, frame.buffer_mut());
    }

    fn handle_event(&mut self, event: &Event) {
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match self.mode {
            UiMode::Pool => self.handle_pool_key(key.code),
            UiMode::ShapePicker => self.handle_shape_picker_key(*key),
            UiMode::VariantPicker => self.handle_variant_picker_key(*key),
            UiMode::FieldPicker => self.handle_field_picker_key(*key),
            UiMode::LinkActionPicker => self.handle_link_action_picker_key(*key),
            UiMode::RenameSlot => self.handle_rename_slot_key(*key),
        }
    }

    fn handle_pool_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Left => self.move_slot_left(),
            KeyCode::Right => self.move_slot_right(),
            KeyCode::Up => self.move_row_up(),
            KeyCode::Down => self.move_row_down(),
            KeyCode::Enter | KeyCode::Char(' ') => self.activate_current_row(),
            _ => {}
        }
    }

    fn handle_shape_picker_key(&mut self, key: KeyEvent) {
        match self
            .shape_picker
            .search
            .handle_key(key, &self.shape_picker.labels)
        {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.mode = UiMode::Pool;
                self.status_message = "Shape selection cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_shape_selection(),
        }
    }

    fn handle_variant_picker_key(&mut self, key: KeyEvent) {
        let Some(variant_picker) = self.variant_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };

        match variant_picker
            .search
            .handle_key(key, &variant_picker.labels)
        {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.variant_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "Variant selection cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_variant_selection(),
        }
    }

    fn handle_field_picker_key(&mut self, key: KeyEvent) {
        let Some(field_picker) = self.field_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };

        match field_picker.search.handle_key(key, &field_picker.labels) {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.field_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "Object selection cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_field_picker_selection(),
        }
    }

    fn handle_link_action_picker_key(&mut self, key: KeyEvent) {
        let Some(link_action_picker) = self.link_action_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.link_action_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "Move/clone selection cancelled.".to_string();
            }
            KeyCode::Up => link_action_picker.list_state.select_previous(),
            KeyCode::Down => link_action_picker.list_state.select_next(),
            KeyCode::Home => link_action_picker.list_state.select(Some(0)),
            KeyCode::End => link_action_picker.list_state.select(Some(1)),
            KeyCode::Enter => self.apply_link_action_selection(),
            _ => {}
        }
    }

    fn handle_rename_slot_key(&mut self, key: KeyEvent) {
        let Some(rename_slot) = self.rename_slot.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };

        match key.code {
            KeyCode::Esc => {
                self.rename_slot = None;
                self.mode = UiMode::Pool;
                self.status_message = "Rename cancelled.".to_string();
            }
            KeyCode::Enter => self.apply_rename_slot(),
            _ => {
                rename_slot.textarea.input(key);
            }
        }
    }
    fn move_slot_left(&mut self) {
        self.active_slot_index = self.active_slot_index.saturating_sub(1);
        self.clamp_active_row();
    }

    fn move_slot_right(&mut self) {
        let max_index = self.total_slot_count().saturating_sub(1);
        self.active_slot_index = (self.active_slot_index + 1).min(max_index);
        self.clamp_active_row();
    }
    fn move_row_up(&mut self) {
        self.active_row_index = self.active_row_index.saturating_sub(1);
    }

    fn move_row_down(&mut self) {
        let max_row = self.active_focusable_rows().saturating_sub(1);
        self.active_row_index = (self.active_row_index + 1).min(max_row);
    }

    fn activate_current_row(&mut self) {
        if self.active_slot_index == self.pseudo_slot_index() {
            self.append_slot();
            return;
        }

        let Some(slot_id) = self.current_slot_id() else {
            return;
        };
        let targets = self.slot_focus_targets(slot_id);
        let Some(target) = targets.get(self.active_row_index).copied() else {
            return;
        };

        match target {
            SlotFocusTarget::Shape => self.open_shape_picker(),
            SlotFocusTarget::ViewPointer => self.jump_to_view_owner(slot_id),
            SlotFocusTarget::Variant => self.open_variant_picker(),
            SlotFocusTarget::FieldType(field_index) => {
                self.describe_field_type_actions(field_index)
            }
            SlotFocusTarget::FieldValue(field_index) => self.activate_field_value(field_index),
            SlotFocusTarget::Inlink(inlink_index) => self.activate_inlink(slot_id, inlink_index),
            SlotFocusTarget::Result(result_index) => self.activate_result(slot_id, result_index),
            SlotFocusTarget::Action(action) => self.activate_slot_action(slot_id, action),
        }
    }

    fn append_slot(&mut self) {
        let slot = ObjectSlot::new(self.next_slot_id);
        self.object_slots.push(slot);
        self.next_slot_id += 1;
        self.active_slot_index = self.object_slots.len().saturating_sub(1);
        self.active_row_index = 0;
        self.status_message = format!(
            "Created slot {}. Pick a shape on the highlighted row.",
            self.object_slots[self.active_slot_index].id
        );
    }

    fn open_shape_picker(&mut self) {
        let Some(slot) = self.current_slot() else {
            return;
        };
        if matches!(slot.kind, SlotKind::View(_)) {
            self.status_message =
                "View slots keep the shape fixed; edit the underlying object through its fields or variant."
                    .to_string();
            return;
        }

        if self.shape_choices.is_empty() {
            self.status_message = "No shapes are registered yet.".to_string();
            return;
        }

        let preferred_index = slot.shape_name.as_ref().and_then(|shape_name| {
            self.shape_choices
                .iter()
                .position(|entry| &entry.label == shape_name)
        });
        self.shape_picker.open(preferred_index);
        self.mode = UiMode::ShapePicker;
        self.status_message =
            "Choose a shape. Type to search; PgUp/PgDn scrolls the preview pane.".to_string();
    }

    fn open_variant_picker(&mut self) {
        let Some(slot_id) = self.current_slot_id() else {
            return;
        };
        let Some((shape_name, variants, selected_variant)) = self.slot_variant_picker_seed(slot_id)
        else {
            self.status_message =
                "The focused slot does not have variants to choose from.".to_string();
            return;
        };

        self.variant_picker = Some(VariantPickerState::new(
            slot_id,
            shape_name,
            variants,
            selected_variant,
        ));
        self.mode = UiMode::VariantPicker;
        self.status_message =
            "Choose a variant. Type to search; PgUp/PgDn scrolls the preview pane.".to_string();
    }

    fn open_field_picker(&mut self, field_index: usize) {
        let Some(owner_slot_id) = self.current_slot_id() else {
            return;
        };
        let Some(field) = self.slot_field(owner_slot_id, field_index).cloned() else {
            return;
        };

        if !self.has_known_shape_label(&field.info.field_shape_name) {
            self.toggle_default_field_value(owner_slot_id, field_index);
            return;
        }

        let mut choices = self
            .matching_slot_ids(&field.info.field_shape_name, owner_slot_id)
            .into_iter()
            .map(|slot_id| FieldPickerChoice::ExistingSlot { slot_id })
            .collect::<Vec<_>>();
        choices.push(FieldPickerChoice::CreateNew);

        let labels = choices
            .iter()
            .map(|choice| self.field_picker_label(*choice, &field.info.field_shape_name))
            .collect::<Vec<_>>();
        let preferred_index = match field.value_state {
            FieldValueState::Linked { slot_id } => choices
                .iter()
                .position(|choice| *choice == FieldPickerChoice::ExistingSlot { slot_id }),
            _ => choices
                .iter()
                .position(|choice| *choice == FieldPickerChoice::CreateNew),
        };

        self.field_picker = Some(FieldPickerState::new(
            owner_slot_id,
            field_index,
            field.info.field_shape_name.clone(),
            choices,
            labels,
            preferred_index,
        ));
        self.mode = UiMode::FieldPicker;
        self.status_message =
            "Choose an object for the field. Type to search; PgUp/PgDn scrolls the preview pane."
                .to_string();
    }

    fn open_link_action_picker(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        selected_slot_id: usize,
    ) {
        self.link_action_picker = Some(LinkActionPickerState::new(
            owner_slot_id,
            field_index,
            selected_slot_id,
        ));
        self.mode = UiMode::LinkActionPicker;
        self.status_message =
            "Choose whether the selected object should move into the field or stay where it is."
                .to_string();
    }

    fn activate_slot_action(&mut self, slot_id: usize, action: SlotAction) {
        match action {
            SlotAction::Rename => self.open_rename_slot(slot_id),
            SlotAction::Delete => self.delete_slot(slot_id),
            SlotAction::Clone => self.clone_slot(slot_id),
            SlotAction::Take => self.take_slot(slot_id),
            SlotAction::Invoke => self.invoke_slot(slot_id),
        }
    }

    fn open_rename_slot(&mut self, slot_id: usize) {
        let existing_name = self.slot_by_id(slot_id).and_then(|slot| slot.name.clone());
        self.rename_slot = Some(RenameSlotState::new(slot_id, existing_name));
        self.mode = UiMode::RenameSlot;
        self.status_message = "Rename the slot and press Enter to save.".to_string();
    }

    fn apply_rename_slot(&mut self) {
        let Some(rename_slot) = self.rename_slot.take() else {
            return;
        };
        let slot_id = rename_slot.slot_id;
        let name = rename_slot.text_value();

        if let Some(slot) = self.slot_by_id_mut(slot_id) {
            slot.name = name.clone();
        }

        self.mode = UiMode::Pool;
        self.invalidate_all_slot_display_caches();
        self.status_message = match name {
            Some(name) => format!("Renamed slot {} to {}.", slot_id, name),
            None => format!("Cleared the name for slot {}.", slot_id),
        };
    }

    fn delete_slot(&mut self, slot_id: usize) {
        let Some(slot_kind) = self.slot_by_id(slot_id).map(|slot| slot.kind.clone()) else {
            return;
        };

        if let SlotKind::View(info) = &slot_kind {
            self.clear_owner_field_link(info.owner_slot_id, info.field_index, slot_id);
        }

        self.remove_slots_cascade(slot_id);
        self.mode = UiMode::Pool;
        self.status_message = match slot_kind {
            SlotKind::Owned => format!("Deleted owned slot {}.", slot_id),
            SlotKind::View(_) => format!("Deleted view slot {} and unset its field.", slot_id),
        };
    }

    fn clone_slot(&mut self, slot_id: usize) {
        let Some(snapshot) = self.slot_snapshot(slot_id) else {
            return;
        };
        let new_slot_id = self.next_slot_id;
        self.next_slot_id += 1;
        self.object_slots
            .push(ObjectSlot::from_snapshot(new_slot_id, snapshot));
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot(new_slot_id);
        self.status_message = format!(
            "Cloned slot {} into new owned slot {}.",
            slot_id, new_slot_id
        );
    }

    fn take_slot(&mut self, slot_id: usize) {
        let Some(slot_kind) = self.slot_by_id(slot_id).map(|slot| slot.kind.clone()) else {
            return;
        };

        let SlotKind::View(info) = slot_kind else {
            self.status_message = format!("Slot {} is already owned.", slot_id);
            return;
        };

        let Some(snapshot) = self.slot_snapshot(slot_id) else {
            return;
        };
        self.clear_owner_field_link(info.owner_slot_id, info.field_index, slot_id);
        if let Some(slot) = self.slot_by_id_mut(slot_id) {
            slot.name = snapshot.name;
            slot.kind = SlotKind::Owned;
            slot.shape_name = snapshot.shape_name;
            slot.body = snapshot.body;
            slot.runtime_state = snapshot
                .value_json
                .map(|json| SlotRuntimeState::ResolvedValue { json });
        }
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot(slot_id);
        self.status_message = format!(
            "Took slot {} out of slot {}.{} and made it owned.",
            slot_id, info.owner_slot_id, info.field_name
        );
    }

    fn activate_result(&mut self, slot_id: usize, result_index: usize) {
        let Some(result_slot_id) = self
            .slot_by_id(slot_id)
            .and_then(|slot| slot.result_slot_ids.get(result_index).copied())
        else {
            return;
        };
        self.jump_to_slot(result_slot_id);
        self.status_message = format!("Jumped to result slot {}.", result_slot_id);
    }
    fn invoke_slot(&mut self, slot_id: usize) {
        let Some(shape_name) = self.slot_shape_name(slot_id).map(str::to_string) else {
            self.status_message = "Pick a shape before invoking.".to_string();
            return;
        };
        let Some(thing) = self.thing_for_shape_name(&shape_name) else {
            self.status_message = format!("{shape_name} is not a registered invokable thing.");
            return;
        };
        let Some(invocation) = thing.invocation else {
            self.status_message = format!("{shape_name} is not invokable.");
            return;
        };
        let json = match self.slot_json_string(slot_id) {
            Ok(json) => json,
            Err(error) => {
                self.status_message = format!("Could not invoke slot {}: {error}", slot_id);
                return;
            }
        };
        let request = match thing.parse_boxed(&json) {
            Ok(request) => request,
            Err(error) => {
                self.status_message = format!("Could not build {shape_name} request: {error}");
                return;
            }
        };
        let future = match thing.invoke_boxed(request) {
            Ok(future) => future,
            Err(error) => {
                self.status_message = format!("Could not invoke {shape_name}: {error}");
                return;
            }
        };

        let result_slot_id = self.next_slot_id;
        self.next_slot_id += 1;
        let output_shape_name = describe_shape(invocation.output_shape);
        let pending = PendingInvocationState {
            join_handle: tokio::spawn(future),
            output_serialize: invocation.output_serialize,
        };
        self.object_slots.push(ObjectSlot::new_result(
            result_slot_id,
            output_shape_name.clone(),
            pending,
        ));
        if let Some(slot) = self.slot_by_id_mut(slot_id) {
            slot.result_slot_ids.push(result_slot_id);
        }
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot(result_slot_id);
        self.status_message = format!(
            "Invoked slot {} and started result slot {} for {}.",
            slot_id, result_slot_id, output_shape_name
        );
    }

    fn apply_shape_selection(&mut self) {
        let Some(choice) = self
            .shape_picker
            .selected_choice(&self.shape_choices)
            .cloned()
        else {
            self.status_message = "No shape is selected.".to_string();
            return;
        };
        let Some(slot_id) = self.current_slot_id() else {
            return;
        };

        let Some(default_focus_target) = self.slot_by_id_mut(slot_id).map(|slot| {
            slot.apply_shape_choice(&choice);
            slot.default_focus_target()
        }) else {
            return;
        };

        self.mode = UiMode::Pool;
        self.invalidate_all_slot_display_caches();
        self.active_row_index = self
            .focus_row_for_slot_target(slot_id, default_focus_target)
            .unwrap_or(0);
        self.status_message = match self.slot_body(slot_id) {
            Some(SlotBody::Unset) => format!("Shape set to {}.", choice.label),
            Some(SlotBody::Struct { fields }) if fields.is_empty() => format!(
                "Shape set to {}. This shape has no reflected fields yet.",
                choice.label
            ),
            Some(SlotBody::Struct { .. }) => format!(
                "Shape set to {}. The slot is ready for field-level construction.",
                choice.label
            ),
            Some(SlotBody::Enum { .. }) => format!(
                "Shape set to {}. Open the variant row to choose which enum branch to build.",
                choice.label
            ),
            None => format!("Shape set to {}.", choice.label),
        };
    }

    fn apply_variant_selection(&mut self) {
        let Some((source_slot_id, variant_index, shape_name, variant)) =
            self.variant_picker.as_ref().and_then(|picker| {
                Some((
                    picker.source_slot_id,
                    picker.selected_index()?,
                    picker.shape_name.clone(),
                    picker.selected_variant()?.clone(),
                ))
            })
        else {
            self.status_message = "No variant is selected.".to_string();
            return;
        };

        let Some(next_focus_target) = self.data_slot_mut_for(source_slot_id).and_then(|slot| {
            slot.select_variant(variant_index)?;
            Some(slot.default_focus_target())
        }) else {
            return;
        };

        self.variant_picker = None;
        self.mode = UiMode::Pool;
        self.invalidate_all_slot_display_caches();
        self.active_row_index = self
            .focus_row_for_slot_target(source_slot_id, next_focus_target)
            .unwrap_or(0);
        self.status_message = match variant.payload_shape_name {
            Some(payload_shape_name) if variant.payload_fields.is_empty() => format!(
                "Selected {}::{}. This payload is a {} value; general value editing is the next interaction to add.",
                shape_name, variant.variant_name, payload_shape_name
            ),
            Some(_) => format!(
                "Selected {}::{}. The payload fields are now visible below the variant row.",
                shape_name, variant.variant_name
            ),
            None => format!("Selected {}::{}.", shape_name, variant.variant_name),
        };
    }

    fn apply_field_picker_selection(&mut self) {
        let Some((owner_slot_id, field_index, required_shape_name, choice)) =
            self.field_picker.as_ref().and_then(|picker| {
                Some((
                    picker.owner_slot_id,
                    picker.field_index,
                    picker.required_shape_name.clone(),
                    picker.selected_choice()?,
                ))
            })
        else {
            self.status_message = "No object is selected.".to_string();
            return;
        };

        self.field_picker = None;
        self.mode = UiMode::Pool;

        match choice {
            FieldPickerChoice::CreateNew => {
                self.create_field_object(owner_slot_id, field_index, &required_shape_name)
            }
            FieldPickerChoice::ExistingSlot { slot_id } => {
                self.open_link_action_picker(owner_slot_id, field_index, slot_id)
            }
        }
    }

    fn apply_link_action_selection(&mut self) {
        let Some((owner_slot_id, field_index, selected_slot_id, action)) =
            self.link_action_picker.as_ref().map(|picker| {
                (
                    picker.owner_slot_id,
                    picker.field_index,
                    picker.selected_slot_id,
                    picker.selected_action(),
                )
            })
        else {
            return;
        };

        self.link_action_picker = None;
        self.mode = UiMode::Pool;

        match action {
            LinkAction::Move => {
                self.move_slot_into_field(owner_slot_id, field_index, selected_slot_id)
            }
            LinkAction::Clone => {
                self.clone_slot_into_field(owner_slot_id, field_index, selected_slot_id)
            }
        }
    }

    fn create_field_object(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        required_shape_name: &str,
    ) {
        let Some(field_name) = self
            .slot_field(owner_slot_id, field_index)
            .map(|field| field.info.field_name)
        else {
            return;
        };
        let Some(choice) = self
            .shape_choices
            .iter()
            .find(|shape| shape.label == required_shape_name)
            .cloned()
        else {
            self.status_message = format!(
                "{} is not currently registered as a constructible shape.",
                required_shape_name
            );
            return;
        };

        let slot_id = self.next_slot_id;
        self.next_slot_id += 1;

        let mut slot = ObjectSlot::new(slot_id);
        slot.apply_shape_choice(&choice);
        let focus_target = slot.default_focus_target();
        slot.kind = SlotKind::View(ViewInfo {
            source_slot_id: slot_id,
            owner_slot_id,
            field_index,
            field_name,
        });

        self.object_slots.push(slot);
        self.set_field_link(owner_slot_id, field_index, slot_id);
        self.jump_to_slot_target(slot_id, focus_target);
        self.status_message = format!(
            "Created slot {} as a {} object and linked it into slot {}.{}.",
            slot_id, required_shape_name, owner_slot_id, field_name
        );
    }

    fn move_slot_into_field(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        selected_slot_id: usize,
    ) {
        if owner_slot_id == selected_slot_id {
            self.status_message =
                "Moving a slot into one of its own fields is not supported yet.".to_string();
            return;
        }

        let Some(field_name) = self
            .slot_field(owner_slot_id, field_index)
            .map(|field| field.info.field_name)
        else {
            return;
        };
        let Some(source_slot_id) = self.data_slot_id_for(selected_slot_id) else {
            return;
        };

        self.clear_links_to_slot(selected_slot_id);
        if let Some(slot) = self.slot_by_id_mut(selected_slot_id) {
            slot.kind = SlotKind::View(ViewInfo {
                source_slot_id,
                owner_slot_id,
                field_index,
                field_name,
            });
        }
        self.set_field_link(owner_slot_id, field_index, selected_slot_id);
        self.jump_to_slot(selected_slot_id);
        self.status_message = format!(
            "Moved slot {} into slot {}.{}.",
            selected_slot_id, owner_slot_id, field_name
        );
    }

    fn clone_slot_into_field(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        selected_slot_id: usize,
    ) {
        if owner_slot_id == selected_slot_id {
            self.status_message =
                "Cloning a slot into one of its own fields is not supported yet.".to_string();
            return;
        }

        let Some(field_name) = self
            .slot_field(owner_slot_id, field_index)
            .map(|field| field.info.field_name)
        else {
            return;
        };
        let Some(source_slot_id) = self.data_slot_id_for(selected_slot_id) else {
            return;
        };

        let slot_id = self.next_slot_id;
        self.next_slot_id += 1;
        self.object_slots.push(ObjectSlot::new_view(
            slot_id,
            source_slot_id,
            owner_slot_id,
            field_index,
            field_name,
        ));
        self.set_field_link(owner_slot_id, field_index, slot_id);
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot(slot_id);
        self.status_message = format!(
            "Cloned slot {} into slot {}.{} as view slot {}.",
            selected_slot_id, owner_slot_id, field_name, slot_id
        );
    }

    fn activate_field_value(&mut self, field_index: usize) {
        let Some(owner_slot_id) = self.current_slot_id() else {
            return;
        };
        let Some(field) = self.slot_field(owner_slot_id, field_index).cloned() else {
            return;
        };

        match field.value_state {
            FieldValueState::Linked { slot_id } => {
                self.jump_to_slot(slot_id);
                self.status_message = format!(
                    "Jumped to slot {} for {} on slot {}.",
                    slot_id, field.info.field_name, owner_slot_id
                );
            }
            FieldValueState::Defaulted | FieldValueState::Unset
                if self.has_known_shape_label(&field.info.field_shape_name) =>
            {
                self.open_field_picker(field_index)
            }
            _ => self.toggle_default_field_value(owner_slot_id, field_index),
        }
    }
    fn toggle_default_field_value(&mut self, owner_slot_id: usize, field_index: usize) {
        let Some(status_message) = self
            .slot_field_mut(owner_slot_id, field_index)
            .map(|field| match field.value_state {
                FieldValueState::Defaulted => {
                    field.value_state = FieldValueState::Unset;
                    format!("Cleared {}.", field.info.field_name)
                }
                FieldValueState::Unset if field.info.has_default => {
                    field.value_state = FieldValueState::Defaulted;
                    format!("Applied the default value for {}.", field.info.field_name)
                }
                FieldValueState::Unset => format!(
                    "{} is required; general value editing is the next interaction to add.",
                    field.info.field_name
                ),
                FieldValueState::Linked { slot_id } => {
                    format!(
                        "{} currently points at slot {}.",
                        field.info.field_name, slot_id
                    )
                }
            })
        else {
            return;
        };

        self.status_message = status_message;
        self.invalidate_all_slot_display_caches();
    }
    fn describe_field_type_actions(&mut self, field_index: usize) {
        let Some(slot_id) = self.current_slot_id() else {
            return;
        };
        let Some(field) = self.slot_field(slot_id, field_index) else {
            return;
        };

        self.status_message = format!(
            "{} has type {}. Type-scoped actions like browsing matching objects or producers are the next interaction to add.",
            field.info.field_name, field.info.field_shape_name
        );
    }
    fn set_field_link(&mut self, owner_slot_id: usize, field_index: usize, linked_slot_id: usize) {
        if let Some(field) = self.slot_field_mut(owner_slot_id, field_index) {
            field.value_state = FieldValueState::Linked {
                slot_id: linked_slot_id,
            };
        }
        self.invalidate_all_slot_display_caches();
    }

    fn clear_links_to_slot(&mut self, slot_id: usize) {
        for slot in &mut self.object_slots {
            match &mut slot.body {
                SlotBody::Unset => {}
                SlotBody::Struct { fields } | SlotBody::Enum { fields, .. } => {
                    for field in fields {
                        if matches!(
                            field.value_state,
                            FieldValueState::Linked {
                                slot_id: linked_slot_id,
                            } if linked_slot_id == slot_id
                        ) {
                            reset_field_value(field);
                        }
                    }
                }
            }
        }
        self.invalidate_all_slot_display_caches();
    }

    fn clear_owner_field_link(&mut self, owner_slot_id: usize, field_index: usize, slot_id: usize) {
        if let Some(field) = self.slot_field_mut(owner_slot_id, field_index) {
            if matches!(
                field.value_state,
                FieldValueState::Linked {
                    slot_id: linked_slot_id,
                } if linked_slot_id == slot_id
            ) {
                reset_field_value(field);
            }
        }
        self.invalidate_all_slot_display_caches();
    }

    fn slot_snapshot(&self, slot_id: usize) -> Option<SlotSnapshot> {
        let data_slot = self.slot_by_id(self.data_slot_id_for(slot_id)?)?;
        let display_slot = self.slot_by_id(slot_id)?;
        if matches!(
            data_slot.runtime_state,
            Some(SlotRuntimeState::Pending(_)) | Some(SlotRuntimeState::Failed { .. })
        ) {
            return None;
        }
        Some(SlotSnapshot {
            name: display_slot.name.clone().or_else(|| data_slot.name.clone()),
            shape_name: data_slot.shape_name.clone(),
            body: data_slot.body.clone(),
            value_json: match &data_slot.runtime_state {
                Some(SlotRuntimeState::ResolvedValue { json }) => Some(json.clone()),
                _ => None,
            },
        })
    }
    fn remove_slots_cascade(&mut self, initial_slot_id: usize) {
        let mut to_remove = BTreeSet::from([initial_slot_id]);
        let mut changed = true;
        while changed {
            changed = false;
            for slot in &self.object_slots {
                if to_remove.contains(&slot.id) {
                    for result_slot_id in &slot.result_slot_ids {
                        changed |= to_remove.insert(*result_slot_id);
                    }
                }

                let SlotKind::View(info) = &slot.kind else {
                    continue;
                };
                if to_remove.contains(&slot.id)
                    || to_remove.contains(&info.source_slot_id)
                    || to_remove.contains(&info.owner_slot_id)
                {
                    changed |= to_remove.insert(slot.id);
                }
            }
        }

        let removed_ids = to_remove.iter().copied().collect::<Vec<_>>();
        for slot_id in removed_ids {
            self.clear_links_to_slot(slot_id);
        }
        for slot in &mut self.object_slots {
            if to_remove.contains(&slot.id) {
                if let Some(SlotRuntimeState::Pending(pending)) = &slot.runtime_state {
                    pending.join_handle.abort();
                }
            }
        }
        self.object_slots
            .retain(|slot| !to_remove.contains(&slot.id));
        for slot in &mut self.object_slots {
            slot.result_slot_ids
                .retain(|slot_id| !to_remove.contains(slot_id));
        }

        self.invalidate_all_slot_display_caches();
        let max_index = self.total_slot_count().saturating_sub(1);
        self.active_slot_index = self.active_slot_index.min(max_index);
        self.clamp_active_row();
    }
    fn total_slot_count(&self) -> usize {
        self.object_slots.len() + 1
    }

    fn pseudo_slot_index(&self) -> usize {
        self.object_slots.len()
    }

    fn current_slot(&self) -> Option<&ObjectSlot> {
        self.object_slots.get(self.active_slot_index)
    }

    fn current_slot_id(&self) -> Option<usize> {
        self.current_slot().map(|slot| slot.id)
    }

    fn active_focusable_rows(&self) -> usize {
        self.current_slot_id()
            .map(|slot_id| self.slot_focus_targets(slot_id).len())
            .unwrap_or(1)
    }

    fn clamp_active_row(&mut self) {
        let max_row = self.active_focusable_rows().saturating_sub(1);
        self.active_row_index = self.active_row_index.min(max_row);
    }

    fn max_visible_slots(&self, width: u16) -> usize {
        usize::from((width / Self::MIN_SLOT_WIDTH).max(1))
    }

    fn visible_slot_range(&self, width: u16) -> Range<usize> {
        let total = self.total_slot_count();
        let max_visible = self.max_visible_slots(width);
        if total <= max_visible {
            return 0..total;
        }

        let half = max_visible / 2;
        let mut start = self.active_slot_index.saturating_sub(half);
        if start + max_visible > total {
            start = total.saturating_sub(max_visible);
        }
        start..(start + max_visible)
    }

    fn slot_index_by_id(&self, slot_id: usize) -> Option<usize> {
        self.object_slots.iter().position(|slot| slot.id == slot_id)
    }

    fn slot_by_id(&self, slot_id: usize) -> Option<&ObjectSlot> {
        self.slot_index_by_id(slot_id)
            .and_then(|index| self.object_slots.get(index))
    }

    fn slot_by_id_mut(&mut self, slot_id: usize) -> Option<&mut ObjectSlot> {
        let index = self.slot_index_by_id(slot_id)?;
        self.object_slots.get_mut(index)
    }

    fn data_slot_id_for(&self, slot_id: usize) -> Option<usize> {
        let slot = self.slot_by_id(slot_id)?;
        match slot.kind {
            SlotKind::Owned => Some(slot.id),
            SlotKind::View(ref info) => Some(info.source_slot_id),
        }
    }

    fn data_slot_mut_for(&mut self, slot_id: usize) -> Option<&mut ObjectSlot> {
        let data_slot_id = self.data_slot_id_for(slot_id)?;
        self.slot_by_id_mut(data_slot_id)
    }

    fn slot_body(&self, slot_id: usize) -> Option<&SlotBody> {
        self.slot_by_id(self.data_slot_id_for(slot_id)?)
            .map(|slot| &slot.body)
    }

    fn slot_shape_name(&self, slot_id: usize) -> Option<&str> {
        self.slot_by_id(self.data_slot_id_for(slot_id)?)
            .and_then(|slot| slot.shape_name.as_deref())
    }

    fn slot_field(&self, slot_id: usize, field_index: usize) -> Option<&ObjectFieldState> {
        self.slot_by_id(self.data_slot_id_for(slot_id)?)
            .and_then(|slot| slot.field(field_index))
    }

    fn slot_field_mut(
        &mut self,
        slot_id: usize,
        field_index: usize,
    ) -> Option<&mut ObjectFieldState> {
        self.data_slot_mut_for(slot_id)
            .and_then(|slot| slot.field_mut(field_index))
    }

    fn slot_variant_picker_seed(
        &self,
        slot_id: usize,
    ) -> Option<(String, Vec<ShapeVariantInfo>, Option<usize>)> {
        self.slot_by_id(self.data_slot_id_for(slot_id)?)
            .and_then(ObjectSlot::variant_picker_seed)
    }

    fn thing_for_shape_name(
        &self,
        shape_name: &str,
    ) -> Option<&'static cloud_terrastodon_registry::Thing> {
        self.shape_choices
            .iter()
            .find(|shape| shape.label == shape_name)
            .map(|shape| shape.thing)
    }

    fn slot_json_string(&self, slot_id: usize) -> Result<String> {
        let value = self.slot_json_value(slot_id)?;
        Ok(serde_json::to_string(&value)?)
    }

    fn slot_json_value(&self, slot_id: usize) -> Result<Value> {
        let data_slot_id = self
            .data_slot_id_for(slot_id)
            .ok_or_else(|| eyre::eyre!("slot {slot_id} has no backing value"))?;
        let slot = self
            .slot_by_id(data_slot_id)
            .ok_or_else(|| eyre::eyre!("slot {data_slot_id} is missing"))?;

        match &slot.runtime_state {
            Some(SlotRuntimeState::Pending(_)) => {
                eyre::bail!("slot {} is still pending", slot.id);
            }
            Some(SlotRuntimeState::Failed { message }) => {
                eyre::bail!("slot {} failed: {}", slot.id, message);
            }
            Some(SlotRuntimeState::ResolvedValue { json }) => {
                return Ok(serde_json::from_str(json)?);
            }
            None => {}
        }

        match &slot.body {
            SlotBody::Unset => eyre::bail!("slot {} is still unset", slot.id),
            SlotBody::Struct { fields } => {
                let mut object = Map::new();
                for field in fields {
                    object.insert(
                        field.info.field_name.to_string(),
                        self.field_json_value(field)?,
                    );
                }
                Ok(Value::Object(object))
            }
            SlotBody::Enum {
                variants,
                selected_variant,
                fields,
            } => {
                let Some(variant_index) = selected_variant else {
                    eyre::bail!("slot {} does not have a selected variant", slot.id);
                };
                let Some(variant) = variants.get(*variant_index) else {
                    eyre::bail!("slot {} selected an invalid variant", slot.id);
                };

                if variant.info.payload_shape_name.is_none() {
                    return Ok(Value::String(variant.info.variant_name.to_string()));
                }
                if fields.is_empty() {
                    eyre::bail!(
                        "{} requires payload value editing, which is not implemented yet",
                        variant.info.variant_name
                    );
                }

                let payload = if fields.len() == 1 {
                    self.field_json_value(&fields[0])?
                } else {
                    let mut payload = Map::new();
                    for field in fields {
                        payload.insert(
                            field.info.field_name.to_string(),
                            self.field_json_value(field)?,
                        );
                    }
                    Value::Object(payload)
                };

                let mut object = Map::new();
                object.insert(variant.info.variant_name.to_string(), payload);
                Ok(Value::Object(object))
            }
        }
    }

    fn field_json_value(&self, field: &ObjectFieldState) -> Result<Value> {
        match field.value_state {
            FieldValueState::Linked { slot_id } => self.slot_json_value(slot_id),
            FieldValueState::Defaulted => self.default_json_value(
                &field.info.field_shape_name,
                field.info.default_value_label.as_deref(),
            ),
            FieldValueState::Unset => {
                eyre::bail!("{} is unset", field.info.field_name);
            }
        }
    }

    fn default_json_value(
        &self,
        field_shape_name: &str,
        default_value_label: Option<&str>,
    ) -> Result<Value> {
        let Some(default_value_label) = default_value_label else {
            eyre::bail!(
                "default value metadata is unavailable for {}",
                field_shape_name
            );
        };
        if let Some(variant_name) = default_value_label
            .strip_prefix(field_shape_name)
            .and_then(|rest| rest.strip_prefix("::"))
        {
            return Ok(Value::String(variant_name.to_string()));
        }
        eyre::bail!(
            "generic default serialization is not implemented for {} ({})",
            field_shape_name,
            default_value_label
        );
    }
    fn slot_display_rows(&mut self, slot_id: usize) -> Vec<SlotDisplayRow> {
        let Some(slot_index) = self.slot_index_by_id(slot_id) else {
            return Vec::new();
        };
        let needs_rebuild = self.object_slots[slot_index].display_cache.is_none();
        if needs_rebuild {
            let rows = self.build_slot_display_rows(slot_id);
            self.object_slots[slot_index].display_cache = Some(rows);
        }
        self.object_slots[slot_index]
            .display_cache
            .clone()
            .unwrap_or_default()
    }

    fn build_slot_display_rows(&self, slot_id: usize) -> Vec<SlotDisplayRow> {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return Vec::new();
        };

        let mut rows = Vec::new();

        if let SlotKind::View(info) = &slot.kind {
            rows.push(focusable_spans_row(
                SlotFocusTarget::ViewPointer,
                vec![
                    Span::styled(
                        "pointer ",
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                    Span::styled(
                        format!("slot {}.{}", info.owner_slot_id, info.field_name),
                        Style::default().fg(Color::Cyan),
                    ),
                ],
            ));
        }

        rows.push(shape_row(self.slot_shape_name(slot_id)));

        match self.slot_body(slot_id) {
            Some(SlotBody::Unset) | None => {}
            Some(SlotBody::Struct { fields }) => {
                if !fields.is_empty() {
                    rows.push(SlotDisplayRow::Static(separator_line("fields")));
                }
                rows.extend(field_rows(fields));
            }
            Some(SlotBody::Enum {
                variants,
                selected_variant,
                fields,
            }) => {
                rows.push(variant_row(variants, *selected_variant));
                if !fields.is_empty() {
                    rows.push(SlotDisplayRow::Static(separator_line("fields")));
                }
                rows.extend(field_rows(fields));
            }
        }

        let inlinks = self.slot_inlinks(slot_id);
        if !inlinks.is_empty() {
            rows.push(SlotDisplayRow::Static(separator_line("inlinks")));
            for (index, inlink) in inlinks.iter().enumerate() {
                rows.push(focusable_plain_row(
                    SlotFocusTarget::Inlink(index),
                    format!("slot {}.{}", inlink.owner_slot_id, inlink.field_name),
                ));
            }
        }

        if let Some(runtime_state) = self.slot_runtime_state(slot_id) {
            rows.push(SlotDisplayRow::Static(separator_line("status")));
            rows.extend(runtime_state_rows(runtime_state));
        }

        if !slot.result_slot_ids.is_empty() {
            rows.push(SlotDisplayRow::Static(separator_line("results")));
            for (index, result_slot_id) in slot.result_slot_ids.iter().copied().enumerate() {
                rows.push(focusable_plain_row(
                    SlotFocusTarget::Result(index),
                    self.result_slot_label(result_slot_id),
                ));
            }
        }

        rows.push(SlotDisplayRow::Static(separator_line("actions")));
        for action in [
            SlotAction::Rename,
            SlotAction::Delete,
            SlotAction::Clone,
            SlotAction::Take,
            SlotAction::Invoke,
        ] {
            if action == SlotAction::Invoke
                && !self
                    .slot_shape_name(slot_id)
                    .and_then(|shape_name| self.thing_for_shape_name(shape_name))
                    .is_some_and(|thing| thing.is_invokable())
            {
                continue;
            }
            rows.push(focusable_plain_row(
                SlotFocusTarget::Action(action),
                self.slot_action_label(slot_id, action),
            ));
        }

        rows
    }

    fn invalidate_all_slot_display_caches(&mut self) {
        for slot in &mut self.object_slots {
            slot.display_cache = None;
        }
    }
    fn slot_focus_targets(&self, slot_id: usize) -> Vec<SlotFocusTarget> {
        let mut targets = Vec::new();
        if matches!(
            self.slot_by_id(slot_id).map(|slot| &slot.kind),
            Some(SlotKind::View(_))
        ) {
            targets.push(SlotFocusTarget::ViewPointer);
        }
        targets.push(SlotFocusTarget::Shape);

        if let Some(body) = self.slot_body(slot_id) {
            match body {
                SlotBody::Unset => {}
                SlotBody::Struct { fields } => {
                    for index in 0..fields.len() {
                        targets.push(SlotFocusTarget::FieldType(index));
                        targets.push(SlotFocusTarget::FieldValue(index));
                    }
                }
                SlotBody::Enum { fields, .. } => {
                    targets.push(SlotFocusTarget::Variant);
                    for index in 0..fields.len() {
                        targets.push(SlotFocusTarget::FieldType(index));
                        targets.push(SlotFocusTarget::FieldValue(index));
                    }
                }
            }
        }

        for (index, _) in self.slot_inlinks(slot_id).iter().enumerate() {
            targets.push(SlotFocusTarget::Inlink(index));
        }

        if let Some(slot) = self.slot_by_id(slot_id) {
            for (index, _) in slot.result_slot_ids.iter().enumerate() {
                targets.push(SlotFocusTarget::Result(index));
            }
        }

        targets.extend([
            SlotFocusTarget::Action(SlotAction::Rename),
            SlotFocusTarget::Action(SlotAction::Delete),
            SlotFocusTarget::Action(SlotAction::Clone),
            SlotFocusTarget::Action(SlotAction::Take),
        ]);
        if self
            .slot_shape_name(slot_id)
            .and_then(|shape_name| self.thing_for_shape_name(shape_name))
            .is_some_and(|thing| thing.is_invokable())
        {
            targets.push(SlotFocusTarget::Action(SlotAction::Invoke));
        }

        targets
    }
    fn slot_default_focus_target(&self, slot_id: usize) -> SlotFocusTarget {
        self.slot_by_id(self.data_slot_id_for(slot_id).unwrap_or(slot_id))
            .map(ObjectSlot::default_focus_target)
            .unwrap_or(SlotFocusTarget::Shape)
    }

    fn focus_row_for_slot_target(&self, slot_id: usize, target: SlotFocusTarget) -> Option<usize> {
        self.slot_focus_targets(slot_id)
            .iter()
            .position(|candidate| *candidate == target)
    }

    fn jump_to_slot(&mut self, slot_id: usize) {
        let target = self.slot_default_focus_target(slot_id);
        self.jump_to_slot_target(slot_id, target);
    }

    fn jump_to_slot_target(&mut self, slot_id: usize, target: SlotFocusTarget) {
        let Some(slot_index) = self.slot_index_by_id(slot_id) else {
            return;
        };
        self.active_slot_index = slot_index;
        self.active_row_index = self.focus_row_for_slot_target(slot_id, target).unwrap_or(0);
    }

    fn jump_to_view_owner(&mut self, slot_id: usize) {
        let Some(SlotKind::View(info)) = self.slot_by_id(slot_id).map(|slot| slot.kind.clone())
        else {
            return;
        };
        self.jump_to_slot_target(
            info.owner_slot_id,
            SlotFocusTarget::FieldValue(info.field_index),
        );
        self.status_message = format!("Jumped to slot {}.{}.", info.owner_slot_id, info.field_name);
    }

    fn activate_inlink(&mut self, slot_id: usize, inlink_index: usize) {
        let Some(inlink) = self.slot_inlinks(slot_id).get(inlink_index).cloned() else {
            return;
        };
        self.jump_to_slot_target(
            inlink.owner_slot_id,
            SlotFocusTarget::FieldValue(inlink.field_index),
        );
        self.status_message = format!(
            "Jumped to slot {}.{}.",
            inlink.owner_slot_id, inlink.field_name
        );
    }

    fn slot_inlinks(&self, slot_id: usize) -> Vec<SlotInlink> {
        let mut inlinks = Vec::new();
        for slot in &self.object_slots {
            let fields = match &slot.body {
                SlotBody::Unset => continue,
                SlotBody::Struct { fields } | SlotBody::Enum { fields, .. } => fields,
            };
            for (field_index, field) in fields.iter().enumerate() {
                if matches!(
                    field.value_state,
                    FieldValueState::Linked {
                        slot_id: linked_slot_id,
                    } if linked_slot_id == slot_id
                ) {
                    inlinks.push(SlotInlink {
                        owner_slot_id: slot.id,
                        field_index,
                        field_name: field.info.field_name,
                    });
                }
            }
        }
        inlinks
    }

    fn has_known_shape_label(&self, shape_name: &str) -> bool {
        self.shape_choices
            .iter()
            .any(|shape| shape.label == shape_name)
    }

    fn matching_slot_ids(&self, shape_name: &str, owner_slot_id: usize) -> Vec<usize> {
        self.object_slots
            .iter()
            .filter(|slot| slot.id != owner_slot_id)
            .filter(|slot| self.slot_shape_name(slot.id) == Some(shape_name))
            .map(|slot| slot.id)
            .collect()
    }

    fn field_picker_label(&self, choice: FieldPickerChoice, required_shape_name: &str) -> String {
        match choice {
            FieldPickerChoice::ExistingSlot { slot_id } => self.slot_picker_label(slot_id),
            FieldPickerChoice::CreateNew => format!("+ create new {required_shape_name}"),
        }
    }

    fn slot_picker_label(&self, slot_id: usize) -> String {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return format!("slot {slot_id}");
        };
        let kind = match slot.kind {
            SlotKind::Owned => "owned",
            SlotKind::View(_) => "view",
        };
        let shape_name = self.slot_shape_name(slot_id).unwrap_or("unset");
        format!("slot {} [{}] - {}", slot.id, kind, shape_name)
    }

    fn slot_action_label(&self, slot_id: usize, action: SlotAction) -> String {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return match action {
                SlotAction::Rename => "rename".to_string(),
                SlotAction::Delete => "delete".to_string(),
                SlotAction::Clone => "clone".to_string(),
                SlotAction::Take => "take".to_string(),
                SlotAction::Invoke => "invoke".to_string(),
            };
        };

        match action {
            SlotAction::Rename => "rename".to_string(),
            SlotAction::Delete => match slot.kind {
                SlotKind::Owned => "delete".to_string(),
                SlotKind::View(_) => "delete (unset field)".to_string(),
            },
            SlotAction::Clone => "clone".to_string(),
            SlotAction::Take => match slot.kind {
                SlotKind::Owned => "take (already owned)".to_string(),
                SlotKind::View(_) => "take".to_string(),
            },
            SlotAction::Invoke => "invoke".to_string(),
        }
    }
    fn variant_picker_preview_lines(&mut self) -> Option<Vec<Line<'static>>> {
        let picker = self.variant_picker.as_ref()?;
        let variant = picker.selected_variant()?;
        Some(variant_preview_lines(&picker.shape_name, variant))
    }

    fn field_picker_preview_lines(&mut self) -> Option<Vec<Line<'static>>> {
        let (choice, required_shape_name) = {
            let picker = self.field_picker.as_ref()?;
            (picker.selected_choice()?, picker.required_shape_name.clone())
        };
        match choice {
            FieldPickerChoice::ExistingSlot { slot_id } => Some(self.slot_preview_lines(slot_id)),
            FieldPickerChoice::CreateNew => self
                .shape_choices
                .iter()
                .find(|shape| shape.label == required_shape_name)
                .map(shape_preview_lines),
        }
    }

    fn link_action_preview_lines(&self) -> Vec<Line<'static>> {
        let Some(picker) = self.link_action_picker.as_ref() else {
            return Vec::new();
        };
        let action = picker.selected_action();
        let slot_label = self.slot_picker_label(picker.selected_slot_id);
        let field_label = self
            .slot_field(picker.owner_slot_id, picker.field_index)
            .map(|field| format!("slot {}.{}", picker.owner_slot_id, field.info.field_name))
            .unwrap_or_else(|| {
                format!("slot {}.field{}", picker.owner_slot_id, picker.field_index)
            });

        match action {
            LinkAction::Move => vec![
                Line::from(format!("{slot_label}")),
                Line::from(format!("will move into {field_label}.")),
                Line::from(""),
                Line::from("The old parent link will be cleared, leaving a hole there if needed."),
                Line::from("This keeps the same slot card, but repoints it at the new field."),
            ],
            LinkAction::Clone => vec![
                Line::from(format!("{slot_label}")),
                Line::from(format!(
                    "will stay put and {field_label} gets a new view slot."
                )),
                Line::from(""),
                Line::from("Use this when the current top-level card should remain where it is."),
                Line::from("The new field gets its own slot card for navigation."),
            ],
        }
    }

    fn slot_lines(&mut self, slot_id: usize, is_active: bool, active_row: usize) -> Vec<Line<'static>> {
        let active_target = if is_active {
            self.slot_focus_targets(slot_id).get(active_row).copied()
        } else {
            None
        };
        let rows = self.slot_display_rows(slot_id);
        render_slot_display_rows(&rows, active_target)
    }

    fn slot_preview_lines(&mut self, slot_id: usize) -> Vec<Line<'static>> {
        let rows = self.slot_display_rows(slot_id);
        render_slot_display_rows(&rows, None)
    }
    fn slot_border_style(&self, slot_id: usize, is_active: bool) -> Style {
        if let Some(runtime_state) = self.slot_runtime_state(slot_id) {
            let color = match runtime_state {
                SlotRuntimeState::Pending(_) => Color::Yellow,
                SlotRuntimeState::ResolvedValue { .. } => Color::Green,
                SlotRuntimeState::Failed { .. } => Color::Red,
            };
            return slot_border_style(color, is_active);
        }

        let color = match (
            self.slot_by_id(slot_id).map(|slot| &slot.kind),
            self.slot_completion(slot_id),
        ) {
            (_, SlotCompletion::Unset) => Color::Red,
            (Some(SlotKind::Owned), SlotCompletion::Partial) => Color::Yellow,
            (Some(SlotKind::Owned), SlotCompletion::Complete) => Color::Green,
            (Some(SlotKind::View(_)), SlotCompletion::Partial) => Color::Cyan,
            (Some(SlotKind::View(_)), SlotCompletion::Complete) => Color::Magenta,
            _ => Color::DarkGray,
        };
        slot_border_style(color, is_active)
    }

    fn slot_completion(&self, slot_id: usize) -> SlotCompletion {
        if let Some(runtime_state) = self.slot_runtime_state(slot_id) {
            return match runtime_state {
                SlotRuntimeState::Pending(_) => SlotCompletion::Partial,
                SlotRuntimeState::ResolvedValue { .. } => SlotCompletion::Complete,
                SlotRuntimeState::Failed { .. } => SlotCompletion::Unset,
            };
        }
        let Some(shape_name) = self.slot_shape_name(slot_id) else {
            return SlotCompletion::Unset;
        };
        if shape_name.is_empty() {
            return SlotCompletion::Unset;
        }
        let Some(body) = self.slot_body(slot_id) else {
            return SlotCompletion::Unset;
        };
        match body {
            SlotBody::Unset => SlotCompletion::Unset,
            SlotBody::Struct { fields } => {
                if fields.iter().all(field_is_resolved) {
                    SlotCompletion::Complete
                } else {
                    SlotCompletion::Partial
                }
            }
            SlotBody::Enum {
                selected_variant,
                fields,
                ..
            } => {
                if selected_variant.is_none() {
                    SlotCompletion::Partial
                } else if fields.iter().all(field_is_resolved) {
                    SlotCompletion::Complete
                } else {
                    SlotCompletion::Partial
                }
            }
        }
    }

    fn slot_runtime_state(&self, slot_id: usize) -> Option<&SlotRuntimeState> {
        let data_slot_id = self.data_slot_id_for(slot_id)?;
        self.slot_by_id(data_slot_id)
            .and_then(|slot| slot.runtime_state.as_ref())
    }

    fn result_slot_label(&self, slot_id: usize) -> String {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return format!("slot {slot_id}");
        };
        let shape_name = slot.shape_name.as_deref().unwrap_or("unset");
        let status = match slot.runtime_state.as_ref() {
            Some(SlotRuntimeState::Pending(_)) => "pending",
            Some(SlotRuntimeState::ResolvedValue { .. }) => "resolved",
            Some(SlotRuntimeState::Failed { .. }) => "failed",
            None => "ready",
        };
        format!("slot {} [{}] - {}", slot.id, status, shape_name)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiMode {
    Pool,
    ShapePicker,
    VariantPicker,
    FieldPicker,
    LinkActionPicker,
    RenameSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotFocusTarget {
    Shape,
    ViewPointer,
    Variant,
    FieldType(usize),
    FieldValue(usize),
    Inlink(usize),
    Result(usize),
    Action(SlotAction),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotAction {
    Rename,
    Delete,
    Clone,
    Take,
    Invoke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotCompletion {
    Unset,
    Partial,
    Complete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SlotInlink {
    owner_slot_id: usize,
    field_index: usize,
    field_name: &'static str,
}

#[derive(Debug, Eq, PartialEq)]
enum PickerSearchAction {
    None,
    Cancel,
    Submit,
}

struct PickerSearchState {
    list_state: ListState,
    query: TextArea<'static>,
    filtered_indices: Vec<usize>,
    preview_scroll: usize,
}

impl PickerSearchState {
    fn new() -> Self {
        Self {
            list_state: ListState::default(),
            query: build_text_area(""),
            filtered_indices: Vec::new(),
            preview_scroll: 0,
        }
    }

    fn reset(&mut self, labels: &[String], preferred_index: Option<usize>) {
        self.list_state = ListState::default();
        self.query = build_text_area("");
        self.preview_scroll = 0;
        self.filtered_indices = filter_indices("", labels);
        self.select_preferred(preferred_index);
    }

    fn selected_filtered_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .and_then(|position| self.filtered_indices.get(position).copied())
    }

    fn handle_key(&mut self, key: KeyEvent, labels: &[String]) -> PickerSearchAction {
        match key.code {
            KeyCode::Esc => return PickerSearchAction::Cancel,
            KeyCode::Enter => return PickerSearchAction::Submit,
            KeyCode::Up => {
                self.list_state.select_previous();
                return PickerSearchAction::None;
            }
            KeyCode::Down => {
                self.list_state.select_next();
                return PickerSearchAction::None;
            }
            KeyCode::Home => {
                if !self.filtered_indices.is_empty() {
                    self.list_state.select(Some(0));
                }
                return PickerSearchAction::None;
            }
            KeyCode::End => {
                if !self.filtered_indices.is_empty() {
                    self.list_state
                        .select(Some(self.filtered_indices.len().saturating_sub(1)));
                }
                return PickerSearchAction::None;
            }
            KeyCode::PageUp => {
                self.preview_scroll = self.preview_scroll.saturating_sub(8);
                return PickerSearchAction::None;
            }
            KeyCode::PageDown => {
                self.preview_scroll = self.preview_scroll.saturating_add(8);
                return PickerSearchAction::None;
            }
            _ => {}
        }

        let previous_selection = self.selected_filtered_index();
        if self.query.input(key) {
            let query = self.query.lines().join("\n");
            self.filtered_indices = filter_indices(&query, labels);
            self.select_preferred(previous_selection);
            self.preview_scroll = 0;
        }

        PickerSearchAction::None
    }

    fn select_preferred(&mut self, preferred_index: Option<usize>) {
        let preferred_position = preferred_index.and_then(|index| {
            self.filtered_indices
                .iter()
                .position(|filtered_index| *filtered_index == index)
        });
        self.list_state.select(
            preferred_position.or_else(|| (!self.filtered_indices.is_empty()).then_some(0)),
        );
    }
}
struct ShapePickerState {
    labels: Vec<String>,
    search: PickerSearchState,
}

impl ShapePickerState {
    fn new(shape_choices: &[KnownShapeInfo]) -> Self {
        let labels = shape_choices
            .iter()
            .map(|shape| shape.label.clone())
            .collect::<Vec<_>>();
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        Self { labels, search }
    }

    fn open(&mut self, preferred_index: Option<usize>) {
        self.search.reset(&self.labels, preferred_index);
    }

    fn selected_choice<'a>(
        &self,
        shape_choices: &'a [KnownShapeInfo],
    ) -> Option<&'a KnownShapeInfo> {
        let index = self.search.selected_filtered_index()?;
        shape_choices.get(index)
    }

    fn list_items(&self) -> Vec<ListItem<'static>> {
        self.search
            .filtered_indices
            .iter()
            .filter_map(|index| self.labels.get(*index))
            .map(|label| ListItem::new(label.clone()))
            .collect()
    }

    fn preview_lines(&self, shape_choices: &[KnownShapeInfo]) -> Vec<Line<'static>> {
        self.selected_choice(shape_choices)
            .map(shape_preview_lines)
            .unwrap_or_else(|| vec![Line::from("No shape is selected.")])
    }
}

struct VariantPickerState {
    source_slot_id: usize,
    shape_name: String,
    labels: Vec<String>,
    variants: Vec<ShapeVariantInfo>,
    search: PickerSearchState,
}

impl VariantPickerState {
    fn new(
        source_slot_id: usize,
        shape_name: String,
        variants: Vec<ShapeVariantInfo>,
        selected_variant: Option<usize>,
    ) -> Self {
        let labels = variants.iter().map(variant_label).collect::<Vec<_>>();
        let mut search = PickerSearchState::new();
        search.reset(&labels, selected_variant);
        Self {
            source_slot_id,
            shape_name,
            labels,
            variants,
            search,
        }
    }

    fn selected_index(&self) -> Option<usize> {
        self.search.selected_filtered_index()
    }

    fn selected_variant(&self) -> Option<&ShapeVariantInfo> {
        self.selected_index()
            .and_then(|index| self.variants.get(index))
    }

    fn list_items(&self) -> Vec<ListItem<'static>> {
        self.search
            .filtered_indices
            .iter()
            .filter_map(|index| self.labels.get(*index))
            .map(|label| ListItem::new(label.clone()))
            .collect()
    }
}

struct FieldPickerState {
    owner_slot_id: usize,
    field_index: usize,
    required_shape_name: String,
    labels: Vec<String>,
    choices: Vec<FieldPickerChoice>,
    search: PickerSearchState,
}

impl FieldPickerState {
    fn new(
        owner_slot_id: usize,
        field_index: usize,
        required_shape_name: String,
        choices: Vec<FieldPickerChoice>,
        labels: Vec<String>,
        preferred_index: Option<usize>,
    ) -> Self {
        let mut search = PickerSearchState::new();
        search.reset(&labels, preferred_index);
        Self {
            owner_slot_id,
            field_index,
            required_shape_name,
            labels,
            choices,
            search,
        }
    }

    fn selected_choice(&self) -> Option<FieldPickerChoice> {
        let index = self.search.selected_filtered_index()?;
        self.choices.get(index).copied()
    }

    fn list_items(&self) -> Vec<ListItem<'static>> {
        self.search
            .filtered_indices
            .iter()
            .filter_map(|index| self.labels.get(*index))
            .map(|label| ListItem::new(label.clone()))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldPickerChoice {
    ExistingSlot { slot_id: usize },
    CreateNew,
}

struct RenameSlotState {
    slot_id: usize,
    textarea: TextArea<'static>,
}

impl RenameSlotState {
    fn new(slot_id: usize, existing_name: Option<String>) -> Self {
        let mut textarea = build_text_area(existing_name.as_deref().unwrap_or(""));
        if existing_name.is_some() {
            textarea.select_all();
        }
        Self { slot_id, textarea }
    }

    fn text_value(&self) -> Option<String> {
        let text = self.textarea.lines().join("\n").trim().to_string();
        (!text.is_empty()).then_some(text)
    }
}

struct LinkActionPickerState {
    owner_slot_id: usize,
    field_index: usize,
    selected_slot_id: usize,
    list_state: ListState,
}

impl LinkActionPickerState {
    fn new(owner_slot_id: usize, field_index: usize, selected_slot_id: usize) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            owner_slot_id,
            field_index,
            selected_slot_id,
            list_state,
        }
    }

    fn selected_action(&self) -> LinkAction {
        match self.list_state.selected().unwrap_or(0) {
            1 => LinkAction::Clone,
            _ => LinkAction::Move,
        }
    }

    fn list_items(&self) -> Vec<ListItem<'static>> {
        vec![ListItem::new("Move"), ListItem::new("Clone")]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LinkAction {
    Move,
    Clone,
}

#[derive(Clone, Debug)]
enum SlotDisplayRow {
    Static(Line<'static>),
    Focusable {
        target: SlotFocusTarget,
        spans: Vec<Span<'static>>,
    },
}
#[derive(Clone, Debug)]
struct SlotSnapshot {
    name: Option<String>,
    shape_name: Option<String>,
    body: SlotBody,
    value_json: Option<String>,
}

#[derive(Debug)]
enum SlotRuntimeState {
    Pending(PendingInvocationState),
    ResolvedValue { json: String },
    Failed { message: String },
}

#[derive(Debug)]
struct PendingInvocationState {
    join_handle: JoinHandle<Result<Box<dyn std::any::Any + Send>>>,
    output_serialize: cloud_terrastodon_registry::SerializeFn,
}

#[derive(Debug)]
struct ObjectSlot {
    id: usize,
    name: Option<String>,
    kind: SlotKind,
    shape_name: Option<String>,
    body: SlotBody,
    result_slot_ids: Vec<usize>,
    runtime_state: Option<SlotRuntimeState>,
    display_cache: Option<Vec<SlotDisplayRow>>,
}

impl ObjectSlot {
    fn from_snapshot(id: usize, snapshot: SlotSnapshot) -> Self {
        Self {
            id,
            name: snapshot.name,
            kind: SlotKind::Owned,
            shape_name: snapshot.shape_name,
            body: snapshot.body,
            result_slot_ids: Vec::new(),
            runtime_state: snapshot
                .value_json
                .map(|json| SlotRuntimeState::ResolvedValue { json }),
            display_cache: None,
        }
    }

    fn new(id: usize) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::Owned,
            shape_name: None,
            body: SlotBody::Unset,
            result_slot_ids: Vec::new(),
            runtime_state: None,
            display_cache: None,
        }
    }

    fn new_result(id: usize, shape_name: String, pending: PendingInvocationState) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::Owned,
            shape_name: Some(shape_name),
            body: SlotBody::Unset,
            result_slot_ids: Vec::new(),
            runtime_state: Some(SlotRuntimeState::Pending(pending)),
            display_cache: None,
        }
    }

    fn new_view(
        id: usize,
        source_slot_id: usize,
        owner_slot_id: usize,
        field_index: usize,
        field_name: &'static str,
    ) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::View(ViewInfo {
                source_slot_id,
                owner_slot_id,
                field_index,
                field_name,
            }),
            shape_name: None,
            body: SlotBody::Unset,
            result_slot_ids: Vec::new(),
            runtime_state: None,
            display_cache: None,
        }
    }

    fn apply_shape_choice(&mut self, choice: &KnownShapeInfo) {
        self.shape_name = Some(choice.label.clone());

        let variants = shape_variants_for_thing(choice.thing)
            .into_iter()
            .map(ObjectVariantState::new)
            .collect::<Vec<_>>();
        if !variants.is_empty() {
            self.body = SlotBody::Enum {
                variants,
                selected_variant: None,
                fields: Vec::new(),
            };
            return;
        }

        let fields = shape_fields_for_thing(choice.thing)
            .into_iter()
            .map(ObjectFieldState::new)
            .collect::<Vec<_>>();
        self.body = SlotBody::Struct { fields };
    }

    fn variant_picker_seed(&self) -> Option<(String, Vec<ShapeVariantInfo>, Option<usize>)> {
        let shape_name = self.shape_name.clone()?;
        let SlotBody::Enum {
            variants,
            selected_variant,
            ..
        } = &self.body
        else {
            return None;
        };

        Some((
            shape_name,
            variants
                .iter()
                .map(|variant| variant.info.clone())
                .collect(),
            *selected_variant,
        ))
    }

    fn select_variant(&mut self, variant_index: usize) -> Option<ShapeVariantInfo> {
        let SlotBody::Enum {
            variants,
            selected_variant,
            fields,
        } = &mut self.body
        else {
            return None;
        };
        let variant = variants.get(variant_index)?.info.clone();
        *selected_variant = Some(variant_index);
        *fields = variant
            .payload_fields
            .clone()
            .into_iter()
            .map(ObjectFieldState::new)
            .collect::<Vec<_>>();
        Some(variant)
    }

    fn default_focus_target(&self) -> SlotFocusTarget {
        match &self.body {
            SlotBody::Unset => SlotFocusTarget::Shape,
            SlotBody::Struct { fields } if fields.is_empty() => SlotFocusTarget::Shape,
            SlotBody::Struct { .. } => SlotFocusTarget::FieldValue(0),
            SlotBody::Enum {
                selected_variant: None,
                ..
            } => SlotFocusTarget::Variant,
            SlotBody::Enum { fields, .. } if fields.is_empty() => SlotFocusTarget::Variant,
            SlotBody::Enum { .. } => SlotFocusTarget::FieldValue(0),
        }
    }

    fn field(&self, field_index: usize) -> Option<&ObjectFieldState> {
        match &self.body {
            SlotBody::Struct { fields } | SlotBody::Enum { fields, .. } => fields.get(field_index),
            SlotBody::Unset => None,
        }
    }

    fn field_mut(&mut self, field_index: usize) -> Option<&mut ObjectFieldState> {
        match &mut self.body {
            SlotBody::Struct { fields } | SlotBody::Enum { fields, .. } => {
                fields.get_mut(field_index)
            }
            SlotBody::Unset => None,
        }
    }
}

#[derive(Clone, Debug)]
enum SlotKind {
    Owned,
    View(ViewInfo),
}

#[derive(Clone, Debug)]
struct ViewInfo {
    source_slot_id: usize,
    owner_slot_id: usize,
    field_index: usize,
    field_name: &'static str,
}

#[derive(Clone, Debug)]
enum SlotBody {
    Unset,
    Struct {
        fields: Vec<ObjectFieldState>,
    },
    Enum {
        variants: Vec<ObjectVariantState>,
        selected_variant: Option<usize>,
        fields: Vec<ObjectFieldState>,
    },
}

#[derive(Clone, Debug)]
struct ObjectFieldState {
    info: ShapeFieldInfo,
    value_state: FieldValueState,
}

impl ObjectFieldState {
    fn new(info: ShapeFieldInfo) -> Self {
        let value_state = if info.has_default {
            FieldValueState::Defaulted
        } else {
            FieldValueState::Unset
        };
        Self { info, value_state }
    }

    fn type_spans(&self, accent: Color) -> Vec<Span<'static>> {
        vec![
            Span::styled(
                "type ",
                Style::default().fg(accent).add_modifier(Modifier::DIM),
            ),
            Span::styled(
                self.info.field_shape_name.clone(),
                Style::default().fg(accent).add_modifier(Modifier::DIM),
            ),
        ]
    }

    fn value_spans(&self, accent: Color) -> Vec<Span<'static>> {
        let mut spans = vec![Span::styled(
            format!("{}: ", self.info.field_name),
            Style::default().fg(accent),
        )];

        match self.value_state {
            FieldValueState::Defaulted => spans.push(Span::styled(
                self.info
                    .default_value_label
                    .clone()
                    .unwrap_or_else(|| "<default>".to_string()),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
            FieldValueState::Unset if self.info.has_default => {
                spans.push(Span::styled("unset", unset_style()));
                spans.push(Span::styled(
                    " (default available)",
                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                ));
            }
            FieldValueState::Unset => spans.push(Span::styled("unset", unset_style())),
            FieldValueState::Linked { slot_id } => spans.push(Span::styled(
                format!("slot {slot_id}"),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
        }

        spans
    }

}

#[derive(Clone, Debug)]
struct ObjectVariantState {
    info: ShapeVariantInfo,
}

impl ObjectVariantState {
    fn new(info: ShapeVariantInfo) -> Self {
        Self { info }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldValueState {
    Defaulted,
    Unset,
    Linked { slot_id: usize },
}

fn field_rows(fields: &[ObjectFieldState]) -> Vec<SlotDisplayRow> {
    let mut rows = Vec::with_capacity(fields.len() * 2);
    for (index, field) in fields.iter().enumerate() {
        let accent = field_group_color(index);
        rows.push(focusable_spans_row(
            SlotFocusTarget::FieldType(index),
            field.type_spans(accent),
        ));
        rows.push(focusable_spans_row(
            SlotFocusTarget::FieldValue(index),
            field.value_spans(accent),
        ));
    }
    rows
}

fn shape_row(shape_name: Option<&str>) -> SlotDisplayRow {
    match shape_name {
        Some(shape_name) => focusable_plain_row(SlotFocusTarget::Shape, shape_name.to_string()),
        None => focusable_spans_row(
            SlotFocusTarget::Shape,
            vec![Span::raw("shape "), Span::styled("unset", unset_style())],
        ),
    }
}

fn variant_row(
    variants: &[ObjectVariantState],
    selected_variant: Option<usize>,
) -> SlotDisplayRow {
    let Some(variant_index) = selected_variant else {
        return focusable_spans_row(
            SlotFocusTarget::Variant,
            vec![Span::raw("variant "), Span::styled("unset", unset_style())],
        );
    };
    let Some(variant) = variants.get(variant_index) else {
        return focusable_spans_row(
            SlotFocusTarget::Variant,
            vec![Span::raw("variant "), Span::styled("unset", unset_style())],
        );
    };

    let mut spans = vec![
        Span::styled(
            "variant ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(
            variant.info.variant_name.to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ];
    if let Some(payload_shape_name) = &variant.info.payload_shape_name {
        spans.push(Span::styled(
            format!(": {payload_shape_name}"),
            Style::default().fg(Color::DarkGray),
        ));
    }

    focusable_spans_row(SlotFocusTarget::Variant, spans)
}

fn runtime_state_rows(runtime_state: &SlotRuntimeState) -> Vec<SlotDisplayRow> {
    match runtime_state {
        SlotRuntimeState::Pending(_) => {
            vec![SlotDisplayRow::Static(Line::from("  pending invocation..."))]
        }
        SlotRuntimeState::Failed { message } => vec![SlotDisplayRow::Static(Line::from(vec![
            Span::raw("  "),
            Span::styled("failed", unset_style()),
            Span::raw(format!(": {message}")),
        ]))],
        SlotRuntimeState::ResolvedValue { json } => {
            let mut rows = vec![SlotDisplayRow::Static(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "resolved",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))];
            rows.extend(pretty_json_lines(json).into_iter().map(SlotDisplayRow::Static));
            rows
        }
    }
}

fn focusable_plain_row(target: SlotFocusTarget, label: impl Into<String>) -> SlotDisplayRow {
    focusable_spans_row(target, vec![Span::raw(label.into())])
}

fn focusable_spans_row(target: SlotFocusTarget, spans: Vec<Span<'static>>) -> SlotDisplayRow {
    SlotDisplayRow::Focusable { target, spans }
}

fn render_slot_display_rows(
    rows: &[SlotDisplayRow],
    active_target: Option<SlotFocusTarget>,
) -> Vec<Line<'static>> {
    rows.iter()
        .map(|row| match row {
            SlotDisplayRow::Static(line) => line.clone(),
            SlotDisplayRow::Focusable { target, spans } => {
                selectable_spans_line(spans.clone(), active_target == Some(*target))
            }
        })
        .collect()
}
fn field_is_resolved(field: &ObjectFieldState) -> bool {
    !matches!(field.value_state, FieldValueState::Unset)
}

fn reset_field_value(field: &mut ObjectFieldState) {
    field.value_state = if field.info.has_default {
        FieldValueState::Defaulted
    } else {
        FieldValueState::Unset
    };
}

fn draw_picker_popup(
    frame: &mut Frame,
    popup_title: &str,
    preview_title: &str,
    search: &mut PickerSearchState,
    items: Vec<ListItem<'static>>,
    total_count: usize,
    preview_lines: Vec<Line<'static>>,
) {
    let area = centered_rect(82, 80, frame.area());
    frame.render_widget(Clear, area);

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .title(popup_title)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = popup_block.inner(area);
    frame.render_widget(popup_block, area);

    let [left_area, right_area] =
        Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)]).areas(inner);
    let [list_area, search_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(left_area);

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "{} matches / {} total",
            search.filtered_indices.len(),
            total_count
        )))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::Yellow));
    frame.render_stateful_widget(list, list_area, &mut search.list_state);

    search
        .query
        .set_block(Block::default().borders(Borders::ALL).title("Search"));
    search.query.render(search_area, frame.buffer_mut());

    let preview_block = Block::default().borders(Borders::ALL).title(preview_title);
    let preview_inner = preview_block.inner(right_area);
    frame.render_widget(preview_block, right_area);
    if preview_inner.width == 0 || preview_inner.height == 0 {
        return;
    }

    let max_scroll = preview_lines
        .len()
        .saturating_sub(preview_inner.height.max(1) as usize);
    search.preview_scroll = search.preview_scroll.min(max_scroll);
    let visible_preview = preview_lines
        .into_iter()
        .skip(search.preview_scroll)
        .take(preview_inner.height as usize)
        .collect::<Vec<_>>();
    let preview = Paragraph::new(Text::from(visible_preview)).wrap(Wrap { trim: false });
    frame.render_widget(preview, preview_inner);
}

fn shape_preview_lines(choice: &KnownShapeInfo) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(Span::styled(
        choice.label.clone(),
        Style::default().add_modifier(Modifier::BOLD),
    ))];

    if choice.thing.is_invokable() {
        lines.push(Line::from("invokable request"));
        if let Some(output_shape) = choice.thing.output_shape() {
            lines.push(Line::from(format!(
                "produces: {}",
                describe_shape(output_shape)
            )));
        }
        let dependencies = choice.thing.input_dependencies();
        if !dependencies.is_empty() {
            lines.push(separator_line("input dependencies"));
            for dependency in dependencies {
                lines.push(Line::from(format!(
                    "  {}: {}",
                    dependency.field_name,
                    describe_shape(dependency.shape)
                )));
            }
        }
    }

    let variants = shape_variants_for_thing(choice.thing);
    if !variants.is_empty() {
        lines.push(separator_line("variants"));
        for variant in variants {
            lines.push(Line::from(format!("  {}", variant_label(&variant))));
            if !variant.payload_fields.is_empty() {
                for field in &variant.payload_fields {
                    lines.extend(field_preview_lines(field, 4));
                }
            }
        }
        return lines;
    }

    let fields = shape_fields_for_thing(choice.thing);
    if fields.is_empty() {
        lines.push(Line::from("No reflected fields."));
    } else {
        lines.push(separator_line("fields"));
        for field in &fields {
            lines.extend(field_preview_lines(field, 2));
        }
    }

    lines
}

fn variant_preview_lines(shape_name: &str, variant: &ShapeVariantInfo) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![
        Span::styled(
            shape_name.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("::"),
        Span::styled(
            variant.variant_name.to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    if variant.is_default {
        lines.push(Line::from("This is the default variant."));
    }

    match &variant.payload_shape_name {
        Some(payload_shape_name) => {
            lines.push(Line::from(format!("payload: {payload_shape_name}")))
        }
        None => lines.push(Line::from("unit variant")),
    }

    if variant.payload_fields.is_empty() {
        if variant.payload_shape_name.is_some() {
            lines.push(Line::from("General payload value editing comes next."));
        }
    } else {
        lines.push(separator_line("payload fields"));
        for field in &variant.payload_fields {
            lines.extend(field_preview_lines(field, 2));
        }
    }

    lines
}

fn field_preview_lines(field: &ShapeFieldInfo, indent: usize) -> Vec<Line<'static>> {
    let prefix = " ".repeat(indent);
    let mut lines = vec![Line::from(format!(
        "{prefix}{}: {}",
        field.field_name, field.field_shape_name
    ))];
    if let Some(default_value_label) = &field.default_value_label {
        lines.push(Line::from(format!(
            "{prefix}default: {default_value_label}"
        )));
    } else if !field.has_default {
        lines.push(Line::from(format!("{prefix}required")));
    }
    lines
}

fn variant_label(variant: &ShapeVariantInfo) -> String {
    match &variant.payload_shape_name {
        Some(payload_shape_name) => format!("{}: {}", variant.variant_name, payload_shape_name),
        None => variant.variant_name.to_string(),
    }
}

fn filter_indices(query: &str, labels: &[String]) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..labels.len()).collect::<Vec<_>>();
    }

    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(nucleo::Config::DEFAULT);
    pattern
        .match_list(labels, &mut matcher)
        .into_iter()
        .filter_map(|(matched_label, _score)| {
            labels.iter().position(|label| label == matched_label)
        })
        .collect()
}

fn build_text_area(query: &str) -> TextArea<'static> {
    let mut text_area = TextArea::new(vec![query.to_string()]);
    text_area.move_cursor(CursorMove::End);
    text_area
}

fn slot_border_style(color: Color, is_active: bool) -> Style {
    if is_active {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(color)
    }
}

fn field_group_color(index: usize) -> Color {
    match index % 4 {
        0 => Color::Blue,
        1 => Color::Green,
        2 => Color::Yellow,
        _ => Color::Magenta,
    }
}

fn unset_style() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

fn selectable_plain_line(label: impl Into<String>, focused: bool) -> Line<'static> {
    selectable_spans_line(vec![Span::raw(label.into())], focused)
}

fn selectable_spans_line(spans: Vec<Span<'static>>, focused: bool) -> Line<'static> {
    let mut line_spans = vec![focus_prefix(focused)];
    line_spans.extend(spans);
    Line::from(line_spans)
}

fn focus_prefix(focused: bool) -> Span<'static> {
    if focused {
        Span::styled(
            "> ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("  ")
    }
}

fn separator_line(label: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled("--- ", Style::default().fg(Color::DarkGray)),
        Span::styled(label.to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(" ---", Style::default().fg(Color::DarkGray)),
    ])
}

fn pretty_json_lines(json: &str) -> Vec<Line<'static>> {
    let pretty = serde_json::from_str::<Value>(json)
        .and_then(|value| serde_json::to_string_pretty(&value))
        .unwrap_or_else(|_| json.to_string());
    pretty
        .lines()
        .map(|line| Line::from(format!("  {line}")))
        .collect()
}
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ]);
    let [_, center, _] = vertical.areas(area);
    let horizontal = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]);
    let [_, middle, _] = horizontal.areas(center);
    middle
}

#[cfg(test)]
mod tests {
    use super::FieldPickerChoice;
    use super::ObjectBrowserApp;
    use super::ObjectSlot;
    use super::ShapeVariantInfo;
    use super::SlotBody;
    use super::SlotKind;
    use cloud_terrastodon_registry::known_shapes;
    use facet::Facet;
    use std::future::Future;
    use std::future::IntoFuture;

    #[derive(Debug, Clone, Facet)]
    #[repr(C)]
    struct DummyInvokeOutput {
        message: String,
    }

    #[derive(Debug, Clone, Facet)]
    #[repr(C)]
    struct DummyInvokeRequest {}

    impl IntoFuture for DummyInvokeRequest {
        type Output = eyre::Result<DummyInvokeOutput>;
        type IntoFuture = std::pin::Pin<Box<dyn Future<Output = Self::Output> + Send>>;

        fn into_future(self) -> Self::IntoFuture {
            Box::pin(async {
                Ok(DummyInvokeOutput {
                    message: "done".to_string(),
                })
            })
        }
    }

    cloud_terrastodon_registry::register_thing!(DummyInvokeOutput);
    cloud_terrastodon_registry::register_thing!(DummyInvokeRequest => DummyInvokeOutput);
    #[test]
    fn creating_a_slot_focuses_the_shape_row() {
        let mut app = ObjectBrowserApp::default();

        app.activate_current_row();

        assert_eq!(app.object_slots.len(), 1);
        assert_eq!(app.active_slot_index, 0);
        assert_eq!(app.active_row_index, 0);
    }

    #[test]
    fn moving_to_the_pseudo_slot_clamps_to_its_single_row() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();
        app.move_row_down();
        app.move_slot_right();

        assert_eq!(app.active_slot_index, 1);
        assert_eq!(app.active_row_index, 0);
    }

    #[test]
    fn enum_shape_focuses_the_variant_row() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let shape_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered for the UI tests");
        app.shape_picker.open(Some(shape_index));
        app.shape_picker.search.list_state.select(Some(shape_index));
        app.apply_shape_selection();

        assert_eq!(app.active_row_index, 1);
        assert!(matches!(app.object_slots[0].body, SlotBody::Enum { .. }));
    }

    #[test]
    fn selecting_a_payload_variant_populates_fields() {
        let shape_choice = known_shapes()
            .into_iter()
            .find(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered for the UI tests");
        let mut slot = ObjectSlot::new(3);
        slot.apply_shape_choice(&shape_choice);

        let (variants, selected_variant) = slot
            .variant_picker_seed()
            .map(|(_, variants, selected_variant)| (variants, selected_variant))
            .expect("enum slot should expose variant picker data");
        assert_eq!(selected_variant, None);

        let variant_index = variants
            .iter()
            .position(|variant: &ShapeVariantInfo| variant.variant_name == "Id")
            .expect("AzureTenantArgument::Id should be reflected");
        let variant = slot
            .select_variant(variant_index)
            .expect("variant selection should succeed");

        assert_eq!(variant.variant_name, "Id");
        assert!(slot.field(0).is_some());
    }

    #[test]
    fn field_picker_filters_matching_slots_and_offers_create_new() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_argument_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered");
        app.shape_picker.open(Some(tenant_argument_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_argument_index));
        app.apply_shape_selection();

        app.active_slot_index = 1;
        app.activate_current_row();
        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.active_row_index = 2;
        app.activate_current_row();

        let picker = app.field_picker.as_ref().expect("field picker should open");
        assert!(
            picker
                .choices
                .contains(&FieldPickerChoice::ExistingSlot { slot_id: 1 })
        );
        assert!(picker.choices.contains(&FieldPickerChoice::CreateNew));
    }

    #[test]
    fn deleting_a_view_slot_unsets_the_owner_field() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_argument_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered");
        app.shape_picker.open(Some(tenant_argument_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_argument_index));
        app.apply_shape_selection();

        app.active_slot_index = 1;
        app.activate_current_row();
        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.clone_slot_into_field(2, 0, 1);
        app.delete_slot(3);

        assert!(app.slot_by_id(3).is_none());
        assert!(matches!(
            app.slot_field(2, 0).map(|field| field.value_state),
            Some(super::FieldValueState::Unset)
        ));
    }

    #[test]
    fn taking_a_view_slot_makes_it_owned_and_unsets_the_owner_field() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_argument_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered");
        app.shape_picker.open(Some(tenant_argument_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_argument_index));
        app.apply_shape_selection();

        app.active_slot_index = 1;
        app.activate_current_row();
        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.clone_slot_into_field(2, 0, 1);
        app.take_slot(3);

        assert!(matches!(
            app.slot_by_id(3).map(|slot| &slot.kind),
            Some(SlotKind::Owned)
        ));
        assert!(matches!(
            app.slot_field(2, 0).map(|field| field.value_state),
            Some(super::FieldValueState::Unset)
        ));
    }

    #[test]
    fn moving_an_existing_slot_turns_it_into_a_view() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_argument_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered");
        app.shape_picker.open(Some(tenant_argument_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_argument_index));
        app.apply_shape_selection();

        app.active_slot_index = 1;
        app.activate_current_row();
        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.move_slot_left();
        app.move_slot_right();
        app.active_row_index = 2;
        app.open_link_action_picker(2, 0, 1);
        app.apply_link_action_selection();

        assert!(matches!(app.object_slots[0].kind, SlotKind::View(_)));
    }

    #[test]
    fn rename_action_updates_the_slot_name() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        app.open_rename_slot(1);
        if let Some(rename_slot) = app.rename_slot.as_mut() {
            rename_slot.textarea = super::build_text_area("tenant source");
        }
        app.apply_rename_slot();

        assert_eq!(
            app.slot_by_id(1).and_then(|slot| slot.name.as_deref()),
            Some("tenant source")
        );
    }

    #[tokio::test]
    async fn invoke_action_creates_and_resolves_a_result_slot() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label == "DummyInvokeRequest")
            .expect("DummyInvokeRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.invoke_slot(1);
        let result_slot_id = app
            .slot_by_id(1)
            .and_then(|slot| slot.result_slot_ids.first().copied())
            .expect("invocation should create a result slot");
        assert!(matches!(
            app.slot_by_id(result_slot_id)
                .and_then(|slot| slot.runtime_state.as_ref()),
            Some(super::SlotRuntimeState::Pending(_))
        ));

        tokio::task::yield_now().await;
        app.advance_pending_invocations();

        let resolved_json = app
            .slot_by_id(result_slot_id)
            .and_then(|slot| slot.runtime_state.as_ref())
            .and_then(|runtime| match runtime {
                super::SlotRuntimeState::ResolvedValue { json } => Some(json.clone()),
                _ => None,
            })
            .expect("result slot should resolve");
        assert!(resolved_json.contains("\"message\":\"done\""));
    }
}

