use cloud_terrastodon_registry::KnownShapeInfo;
use cloud_terrastodon_registry::ShapeFieldInfo;
use cloud_terrastodon_registry::known_shapes;
use cloud_terrastodon_registry::shape_fields_for_thing;
use crossterm::event::EventStream;
use eyre::Result;
use futures::StreamExt;
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
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
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use std::ops::Range;
use std::time::Duration;
use tracing::info;

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
    shape_picker_state: ListState,
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
        let mut shape_picker_state = ListState::default();
        if shape_choices.is_empty() {
            shape_picker_state.select(None);
        } else {
            shape_picker_state.select(Some(0));
        }

        Self {
            should_quit: false,
            mode: UiMode::Pool,
            shape_picker_state,
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
    const MIN_SLOT_WIDTH: u16 = 28;

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

        if self.mode == UiMode::ShapePicker {
            self.draw_shape_picker(frame);
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
            .title(format!("#{}", slot.id))
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
            .title("new")
            .border_style(slot_border_style(is_active));
        let line = selectable_line("+ create object", is_active && self.active_row_index == 0);
        let paragraph = Paragraph::new(vec![line]).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_shape_picker(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 70, frame.area());
        frame.render_widget(Clear, area);

        if self.shape_choices.is_empty() {
            let paragraph = Paragraph::new(vec![Line::from("No shapes are registered yet.")])
                .block(Block::default().borders(Borders::ALL).title("Pick Shape"));
            frame.render_widget(paragraph, area);
            return;
        }

        let items = self
            .shape_choices
            .iter()
            .map(|shape| ListItem::new(shape.label.clone()))
            .collect::<Vec<_>>();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Pick Shape"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_stateful_widget(list, area, &mut self.shape_picker_state);
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
            UiMode::ShapePicker => self.handle_shape_picker_key(key.code),
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

    fn handle_shape_picker_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = UiMode::Pool;
                self.status_message = "Shape selection cancelled.".to_string();
            }
            KeyCode::Up => self.shape_picker_state.select_previous(),
            KeyCode::Down => self.shape_picker_state.select_next(),
            KeyCode::Enter | KeyCode::Char(' ') => self.apply_shape_selection(),
            _ => {}
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

        match self.active_row_index {
            0 => {
                self.status_message =
                    "Slot naming is the next interaction to add for the focused slot row."
                        .to_string();
            }
            1 => self.open_shape_picker(),
            row => self.toggle_field_value(row.saturating_sub(2)),
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

        if let Some(slot) = self.current_slot()
            && let Some(shape_name) = &slot.shape_name
            && let Some(index) = self
                .shape_choices
                .iter()
                .position(|entry| &entry.label == shape_name)
        {
            self.shape_picker_state.select(Some(index));
        } else if self.shape_picker_state.selected().is_none() {
            self.shape_picker_state.select(Some(0));
        }

        self.mode = UiMode::ShapePicker;
        self.status_message = "Choose a shape for the focused slot.".to_string();
    }

    fn apply_shape_selection(&mut self) {
        let Some(choice) = selected_index(&self.shape_picker_state, self.shape_choices.len())
            .and_then(|index| self.shape_choices.get(index).cloned())
        else {
            self.status_message = "No shape is selected.".to_string();
            return;
        };

        let field_count = if let Some(slot) = self.current_slot_mut() {
            slot.apply_shape_choice(&choice)
        } else {
            0
        };

        self.mode = UiMode::Pool;
        self.active_row_index = if field_count == 0 { 1 } else { 2 };
        self.status_message = format!(
            "Shape set to {}. {}",
            choice.label,
            if field_count == 0 {
                "This shape has no reflected fields."
            } else {
                "Use Up/Down to inspect and toggle defaulted fields."
            }
        );
    }

    fn toggle_field_value(&mut self, field_index: usize) {
        let Some(slot) = self.current_slot_mut() else {
            return;
        };
        let Some(field) = slot.fields.get_mut(field_index) else {
            return;
        };

        match field.value_state {
            FieldValueState::Defaulted => {
                field.value_state = FieldValueState::Unset;
                self.status_message =
                    format!("Cleared {} on slot {}.", field.info.field_name, slot.id);
            }
            FieldValueState::Unset if field.info.has_default => {
                field.value_state = FieldValueState::Defaulted;
                self.status_message = format!(
                    "Applied the default value for {} on slot {}.",
                    field.info.field_name, slot.id
                );
            }
            FieldValueState::Unset => {
                self.status_message = format!(
                    "{} is required; general value editing is the next interaction to add.",
                    field.info.field_name
                );
            }
        }
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
}

#[derive(Clone, Debug)]
struct ObjectSlot {
    id: usize,
    name: Option<String>,
    shape_name: Option<String>,
    fields: Vec<ObjectFieldState>,
}

impl ObjectSlot {
    fn new(id: usize) -> Self {
        Self {
            id,
            name: None,
            shape_name: None,
            fields: Vec::new(),
        }
    }

    fn focusable_row_count(&self) -> usize {
        2 + self.fields.len()
    }

    fn apply_shape_choice(&mut self, choice: &KnownShapeInfo) -> usize {
        self.shape_name = Some(choice.label.clone());
        self.fields = shape_fields_for_thing(choice.thing)
            .into_iter()
            .map(ObjectFieldState::new)
            .collect();
        self.fields.len()
    }

    fn lines(&self, is_active: bool, active_row: usize) -> Vec<Line<'static>> {
        let mut lines = vec![
            selectable_line(
                format!(
                    "slot {} ({})",
                    self.id,
                    self.name.as_deref().unwrap_or("unnamed")
                ),
                is_active && active_row == 0,
            ),
            selectable_line(
                self.shape_name
                    .clone()
                    .unwrap_or_else(|| "shape unset".to_string()),
                is_active && active_row == 1,
            ),
        ];

        if !self.fields.is_empty() {
            lines.push(separator_line());
        }

        for (index, field) in self.fields.iter().enumerate() {
            lines.push(selectable_line(
                field.display_text(),
                is_active && active_row == index + 2,
            ));
        }

        lines
    }
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

    fn display_text(&self) -> String {
        let value = match self.value_state {
            FieldValueState::Defaulted => self
                .info
                .default_value_label
                .as_deref()
                .unwrap_or("<default>"),
            FieldValueState::Unset if self.info.has_default => "unset (default available)",
            FieldValueState::Unset => "unset",
        };
        format!("{}: {}", self.info.field_name, value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldValueState {
    Defaulted,
    Unset,
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

fn selectable_line(label: impl Into<String>, focused: bool) -> Line<'static> {
    let label = label.into();
    if focused {
        return Line::from(vec![
            Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(label, Style::default().add_modifier(Modifier::BOLD)),
        ]);
    }

    Line::from(format!("  {label}"))
}

fn separator_line() -> Line<'static> {
    Line::from(Span::styled(
        "  ────────────────────",
        Style::default().fg(Color::DarkGray),
    ))
}

fn selected_index(state: &ListState, len: usize) -> Option<usize> {
    state.selected().filter(|index| *index < len)
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
}
