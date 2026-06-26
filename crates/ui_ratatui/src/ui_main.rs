use cloud_terrastodon_registry::KnownShapeInfo;
use cloud_terrastodon_registry::ShapeFieldInfo;
use cloud_terrastodon_registry::ShapeVariantInfo;
use cloud_terrastodon_registry::known_shapes;
use cloud_terrastodon_registry::shape_fields_for_thing;
use cloud_terrastodon_registry::shape_variants_for_thing;
use crossterm::event::EventStream;
use eyre::Result;
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
use ratatui::widgets::Widget;
use ratatui::widgets::Wrap;
use std::ops::Range;
use std::time::Duration;
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
    const MIN_SLOT_WIDTH: u16 = 30;

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }
        Ok(())
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

        let status = Line::from(self.status_message.as_str());
        frame.render_widget(status, status_area);

        match self.mode {
            UiMode::Pool => {}
            UiMode::ShapePicker => self.draw_shape_picker_popup(frame),
            UiMode::VariantPicker => self.draw_variant_picker_popup(frame),
        }
    }

    fn draw_pool(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Object Pool");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let visible = self.visible_slot_range(inner.width);
        let constraints = vec![Constraint::Fill(1); visible.len()];
        let slot_areas = Layout::horizontal(constraints).split(inner);

        for (offset, slot_index) in visible.enumerate() {
            let Some(slot_area) = slot_areas.get(offset).copied() else {
                break;
            };
            let is_active = slot_index == self.active_slot_index;
            if slot_index == self.pseudo_slot_index() {
                self.draw_new_slot(frame, slot_area, is_active);
            } else if let Some(slot) = self.object_slots.get(slot_index) {
                self.draw_object_slot(frame, slot_area, slot, is_active);
            }
        }
    }

    fn draw_object_slot(&self, frame: &mut Frame, area: Rect, slot: &ObjectSlot, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("slot {}", slot.id))
            .border_style(slot_border_style(is_active));
        let paragraph = Paragraph::new(slot.lines(is_active, self.active_row_index))
            .block(block)
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, area);
    }

    fn draw_new_slot(&self, frame: &mut Frame, area: Rect, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::LightTripleDashed)
            .title("new slot")
            .border_style(slot_border_style(is_active));
        let line =
            selectable_plain_line("+ create object", is_active && self.active_row_index == 0);
        let paragraph = Paragraph::new(vec![line]).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_shape_picker_popup(&mut self, frame: &mut Frame) {
        let preview_lines = self.shape_picker.preview_lines(&self.shape_choices);
        let items = self.shape_picker.list_items();
        draw_picker_popup(
            frame,
            "Pick Shape",
            "Shape Preview",
            &mut self.shape_picker.search,
            items,
            self.shape_picker.labels.len(),
            preview_lines,
        );
    }

    fn draw_variant_picker_popup(&mut self, frame: &mut Frame) {
        let Some(variant_picker) = self.variant_picker.as_mut() else {
            return;
        };
        let preview_lines = variant_picker.preview_lines();
        let items = variant_picker.list_items();
        draw_picker_popup(
            frame,
            "Pick Variant",
            "Variant Preview",
            &mut variant_picker.search,
            items,
            variant_picker.labels.len(),
            preview_lines,
        );
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

        let Some(target) = self
            .current_slot()
            .and_then(|slot| slot.focus_target(self.active_row_index))
        else {
            return;
        };

        match target {
            SlotFocusTarget::SlotName => {
                self.status_message =
                    "Slot naming is the next interaction to add for the focused slot row."
                        .to_string();
            }
            SlotFocusTarget::Shape => self.open_shape_picker(),
            SlotFocusTarget::Variant => self.open_variant_picker(),
            SlotFocusTarget::FieldType(field_index) => {
                self.describe_field_type_actions(field_index)
            }
            SlotFocusTarget::FieldValue(field_index) => self.toggle_field_value(field_index),
        }
    }

    fn append_slot(&mut self) {
        let slot = ObjectSlot::new(self.next_slot_id);
        self.object_slots.push(slot);
        self.next_slot_id += 1;
        self.active_slot_index = self.object_slots.len().saturating_sub(1);
        self.active_row_index = 1;
        self.status_message = format!(
            "Created slot {}. Pick a shape on the highlighted row.",
            self.object_slots[self.active_slot_index].id
        );
    }

    fn open_shape_picker(&mut self) {
        if self.shape_choices.is_empty() {
            self.status_message = "No shapes are registered yet.".to_string();
            return;
        }

        let preferred_index = self.current_slot().and_then(|slot| {
            slot.shape_name.as_ref().and_then(|shape_name| {
                self.shape_choices
                    .iter()
                    .position(|entry| &entry.label == shape_name)
            })
        });
        self.shape_picker.open(preferred_index);
        self.mode = UiMode::ShapePicker;
        self.status_message =
            "Choose a shape. Type to search; PgUp/PgDn scrolls the preview pane.".to_string();
    }

    fn open_variant_picker(&mut self) {
        let Some((shape_name, variants, selected_variant)) = self
            .current_slot()
            .and_then(ObjectSlot::variant_picker_seed)
        else {
            self.status_message =
                "The focused slot does not have variants to choose from.".to_string();
            return;
        };

        self.variant_picker = Some(VariantPickerState::new(
            shape_name,
            variants,
            selected_variant,
        ));
        self.mode = UiMode::VariantPicker;
        self.status_message =
            "Choose a variant. Type to search; PgUp/PgDn scrolls the preview pane.".to_string();
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

        let Some((default_focus_row, status_message)) = self.current_slot_mut().map(|slot| {
            slot.apply_shape_choice(&choice);
            let default_focus_row = slot.default_focus_row();
            let status_message = match &slot.body {
                SlotBody::Unset => format!("Shape set to {}.", choice.label),
                SlotBody::Struct { fields } if fields.is_empty() => format!(
                    "Shape set to {}. This shape has no reflected fields yet.",
                    choice.label
                ),
                SlotBody::Struct { .. } => format!(
                    "Shape set to {}. The slot is ready for field-level construction.",
                    choice.label
                ),
                SlotBody::Enum { .. } => format!(
                    "Shape set to {}. Open the variant row to choose which enum branch to build.",
                    choice.label
                ),
            };
            (default_focus_row, status_message)
        }) else {
            return;
        };

        self.mode = UiMode::Pool;
        self.active_row_index = default_focus_row;
        self.status_message = status_message;
    }
    fn apply_variant_selection(&mut self) {
        let Some((variant_index, shape_name, variant)) =
            self.variant_picker.as_ref().and_then(|picker| {
                let variant_index = picker.selected_index()?;
                let variant = picker.selected_variant()?.clone();
                Some((variant_index, picker.shape_name.clone(), variant))
            })
        else {
            self.status_message = "No variant is selected.".to_string();
            return;
        };

        let Some((next_focus_row, slot_id)) = self.current_slot_mut().and_then(|slot| {
            slot.select_variant(variant_index)
                .map(|(_, slot_id, _, next_focus_row)| (next_focus_row, slot_id))
        }) else {
            return;
        };

        self.variant_picker = None;
        self.mode = UiMode::Pool;
        self.active_row_index = next_focus_row;
        self.status_message = match variant.payload_shape_name {
            Some(payload_shape_name) if variant.payload_fields.is_empty() => format!(
                "Selected {}::{}. This payload is a {} value; general value editing is the next interaction to add.",
                shape_name, variant.variant_name, payload_shape_name
            ),
            Some(_) => format!(
                "Selected {}::{} for slot {}. The payload fields are now visible below the variant row.",
                shape_name, variant.variant_name, slot_id
            ),
            None => format!(
                "Selected {}::{} for slot {}.",
                shape_name, variant.variant_name, slot_id
            ),
        };
    }

    fn describe_field_type_actions(&mut self, field_index: usize) {
        let Some(slot) = self.current_slot() else {
            return;
        };
        let Some(field) = slot.field(field_index) else {
            return;
        };

        self.status_message = format!(
            "{} has type {}. Type-scoped actions like browsing matching objects or producers are the next interaction to add.",
            field.info.field_name, field.info.field_shape_name
        );
    }

    fn toggle_field_value(&mut self, field_index: usize) {
        let Some(status_message) = self.current_slot_mut().and_then(|slot| {
            let slot_id = slot.id;
            let field = slot.field_mut(field_index)?;

            Some(match field.value_state {
                FieldValueState::Defaulted => {
                    field.value_state = FieldValueState::Unset;
                    format!("Cleared {} on slot {}.", field.info.field_name, slot_id)
                }
                FieldValueState::Unset if field.info.has_default => {
                    field.value_state = FieldValueState::Defaulted;
                    format!(
                        "Applied the default value for {} on slot {}.",
                        field.info.field_name, slot_id
                    )
                }
                FieldValueState::Unset => format!(
                    "{} is required; general value editing is the next interaction to add.",
                    field.info.field_name
                ),
            })
        }) else {
            return;
        };

        self.status_message = status_message;
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

    fn current_slot_mut(&mut self) -> Option<&mut ObjectSlot> {
        self.object_slots.get_mut(self.active_slot_index)
    }

    fn active_focusable_rows(&self) -> usize {
        self.current_slot()
            .map_or(1, ObjectSlot::focusable_row_count)
    }

    fn clamp_active_row(&mut self) {
        let max_row = self.active_focusable_rows().saturating_sub(1);
        self.active_row_index = self.active_row_index.min(max_row);
    }

    fn visible_slot_range(&self, width: u16) -> Range<usize> {
        let total = self.total_slot_count();
        let max_visible = usize::from((width / Self::MIN_SLOT_WIDTH).max(1));
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiMode {
    Pool,
    ShapePicker,
    VariantPicker,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotFocusTarget {
    SlotName,
    Shape,
    Variant,
    FieldType(usize),
    FieldValue(usize),
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
        self.search
            .selected_filtered_index()
            .and_then(|index| shape_choices.get(index))
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
            .unwrap_or_else(|| vec![Line::from("No shapes match the current query.")])
    }
}

struct VariantPickerState {
    shape_name: String,
    variants: Vec<ShapeVariantInfo>,
    labels: Vec<String>,
    search: PickerSearchState,
}

impl VariantPickerState {
    fn new(
        shape_name: String,
        variants: Vec<ShapeVariantInfo>,
        preferred_index: Option<usize>,
    ) -> Self {
        let labels = variants
            .iter()
            .map(|variant| variant_label(variant))
            .collect::<Vec<_>>();
        let mut search = PickerSearchState::new();
        search.reset(&labels, preferred_index);
        Self {
            shape_name,
            variants,
            labels,
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

    fn preview_lines(&self) -> Vec<Line<'static>> {
        self.selected_variant()
            .map(|variant| variant_preview_lines(&self.shape_name, variant))
            .unwrap_or_else(|| vec![Line::from("No variants match the current query.")])
    }
}

#[derive(Clone, Debug)]
struct ObjectSlot {
    id: usize,
    name: Option<String>,
    shape_name: Option<String>,
    body: SlotBody,
}

impl ObjectSlot {
    fn new(id: usize) -> Self {
        Self {
            id,
            name: None,
            shape_name: None,
            body: SlotBody::Unset,
        }
    }

    fn focusable_row_count(&self) -> usize {
        match &self.body {
            SlotBody::Unset => 2,
            SlotBody::Struct { fields } => 2 + (fields.len() * 2),
            SlotBody::Enum { fields, .. } => 3 + (fields.len() * 2),
        }
    }

    fn focus_target(&self, row_index: usize) -> Option<SlotFocusTarget> {
        if row_index == 0 {
            return Some(SlotFocusTarget::SlotName);
        }
        if row_index == 1 {
            return Some(SlotFocusTarget::Shape);
        }

        match &self.body {
            SlotBody::Unset => None,
            SlotBody::Struct { fields } => {
                let body_index = row_index.saturating_sub(2);
                focus_target_for_fields(body_index, fields.len())
            }
            SlotBody::Enum { fields, .. } => {
                if row_index == 2 {
                    Some(SlotFocusTarget::Variant)
                } else {
                    let body_index = row_index.saturating_sub(3);
                    focus_target_for_fields(body_index, fields.len())
                }
            }
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

    fn default_focus_row(&self) -> usize {
        match &self.body {
            SlotBody::Unset => 1,
            SlotBody::Struct { fields } => {
                if fields.is_empty() {
                    1
                } else {
                    2
                }
            }
            SlotBody::Enum { .. } => 2,
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

    fn select_variant(
        &mut self,
        variant_index: usize,
    ) -> Option<(String, usize, ShapeVariantInfo, usize)> {
        let shape_name = self.shape_name.clone()?;
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
        let next_focus_row = if fields.is_empty() { 2 } else { 3 };
        Some((shape_name, self.id, variant, next_focus_row))
    }

    fn lines(&self, is_active: bool, active_row: usize) -> Vec<Line<'static>> {
        let active_target = if is_active {
            self.focus_target(active_row)
        } else {
            None
        };

        let mut lines = vec![
            selectable_plain_line(
                format!(
                    "slot {} ({})",
                    self.id,
                    self.name.as_deref().unwrap_or("unnamed")
                ),
                active_target == Some(SlotFocusTarget::SlotName),
            ),
            self.shape_line(active_target == Some(SlotFocusTarget::Shape)),
        ];

        match &self.body {
            SlotBody::Unset => {}
            SlotBody::Struct { fields } => {
                if !fields.is_empty() {
                    lines.push(separator_line("fields"));
                }
                lines.extend(field_lines(fields, active_target));
            }
            SlotBody::Enum {
                selected_variant,
                variants,
                fields,
            } => {
                lines.push(self.variant_line(
                    variants,
                    *selected_variant,
                    active_target == Some(SlotFocusTarget::Variant),
                ));
                if !fields.is_empty() {
                    lines.push(separator_line("fields"));
                }
                lines.extend(field_lines(fields, active_target));
            }
        }

        lines
    }

    fn shape_line(&self, focused: bool) -> Line<'static> {
        if let Some(shape_name) = &self.shape_name {
            return selectable_plain_line(shape_name.clone(), focused);
        }

        selectable_spans_line(
            vec![Span::raw("shape "), Span::styled("unset", unset_style())],
            focused,
        )
    }

    fn variant_line(
        &self,
        variants: &[ObjectVariantState],
        selected_variant: Option<usize>,
        focused: bool,
    ) -> Line<'static> {
        let Some(variant_index) = selected_variant else {
            return selectable_spans_line(
                vec![Span::raw("variant "), Span::styled("unset", unset_style())],
                focused,
            );
        };
        let Some(variant) = variants.get(variant_index) else {
            return selectable_spans_line(
                vec![Span::raw("variant "), Span::styled("unset", unset_style())],
                focused,
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

        selectable_spans_line(spans, focused)
    }
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

    fn type_line(&self, accent: Color, focused: bool) -> Line<'static> {
        selectable_spans_line(
            vec![
                Span::styled(
                    "type ",
                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    self.info.field_shape_name.clone(),
                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                ),
            ],
            focused,
        )
    }

    fn value_line(&self, accent: Color, focused: bool) -> Line<'static> {
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
        }

        selectable_spans_line(spans, focused)
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
}

fn field_lines(
    fields: &[ObjectFieldState],
    active_target: Option<SlotFocusTarget>,
) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(fields.len() * 2);
    for (index, field) in fields.iter().enumerate() {
        let accent = field_group_color(index);
        lines.push(field.type_line(
            accent,
            active_target == Some(SlotFocusTarget::FieldType(index)),
        ));
        lines.push(field.value_line(
            accent,
            active_target == Some(SlotFocusTarget::FieldValue(index)),
        ));
    }
    lines
}

fn focus_target_for_fields(body_index: usize, field_count: usize) -> Option<SlotFocusTarget> {
    let field_index = body_index / 2;
    if field_index >= field_count {
        return None;
    }

    if body_index % 2 == 0 {
        Some(SlotFocusTarget::FieldType(field_index))
    } else {
        Some(SlotFocusTarget::FieldValue(field_index))
    }
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
                cloud_terrastodon_registry::describe_shape(output_shape)
            )));
        }
        let dependencies = choice.thing.input_dependencies();
        if !dependencies.is_empty() {
            lines.push(separator_line("input dependencies"));
            for dependency in dependencies {
                lines.push(Line::from(format!(
                    "  {}: {}",
                    dependency.field_name,
                    cloud_terrastodon_registry::describe_shape(dependency.shape)
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
            lines.push(Line::from(format!("payload: {payload_shape_name}")));
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

fn slot_border_style(is_active: bool) -> Style {
    if is_active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
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
    use super::ObjectBrowserApp;
    use super::ObjectSlot;
    use super::ShapeVariantInfo;
    use super::SlotBody;
    use cloud_terrastodon_registry::known_shapes;

    #[test]
    fn creating_a_slot_focuses_the_shape_row() {
        let mut app = ObjectBrowserApp::default();

        app.activate_current_row();

        assert_eq!(app.object_slots.len(), 1);
        assert_eq!(app.active_slot_index, 0);
        assert_eq!(app.active_row_index, 1);
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

        assert_eq!(app.active_row_index, 2);
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
        let (_, _, variant, next_focus_row) = slot
            .select_variant(variant_index)
            .expect("variant selection should succeed");

        assert_eq!(variant.variant_name, "Id");
        assert_eq!(next_focus_row, 3);
        assert!(slot.field(0).is_some());
    }
}
