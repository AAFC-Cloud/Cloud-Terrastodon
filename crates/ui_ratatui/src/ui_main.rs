use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionInvocation;
use cloud_terrastodon_registry::KnownShapeInfo;
use cloud_terrastodon_registry::ProductionKind;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::ShapeFieldInfo;
use cloud_terrastodon_registry::ShapeVariantInfo;
use cloud_terrastodon_registry::describe_function;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::functions_from;
use cloud_terrastodon_registry::functions_to;
use cloud_terrastodon_registry::known_shapes;
use cloud_terrastodon_registry::map_value_shape as registry_map_value_shape;
use cloud_terrastodon_registry::sequence_element_shape;
use cloud_terrastodon_registry::shape_field_shape;
use cloud_terrastodon_registry::shape_fields_for_thing;
use cloud_terrastodon_registry::shape_variants_for_thing;
use crossterm::event::EventStream;
use eyre::Result;
use futures::FutureExt;
use futures::StreamExt;
use nucleo::Matcher;
use nucleo::Utf32Str;
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
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ops::Range;
use std::time::Duration;
use std::time::Instant;
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
    pool_surface: PoolSurface,
    active_breadcrumb_index: usize,
    projection_stack: Vec<JsonProjectionView>,
    recent_escape_presses: Vec<Instant>,
    shape_picker: ShapePickerState,
    variant_picker: Option<VariantPickerState>,
    field_picker: Option<FieldPickerState>,
    function_picker: Option<FunctionPickerState>,
    link_action_picker: Option<LinkActionPickerState>,
    rename_slot: Option<RenameSlotState>,
    slot_search: Option<SlotSearchState>,
    slot_axis: SlotAxis,
    focused_slot_fill: bool,
    show_hotkey_help: bool,
    slot_width: u16,
    slot_height: u16,
    shape_choices: Vec<KnownShapeInfo>,
    object_slots: Vec<ObjectSlot>,
    projection_cache: RefCell<ProjectionCache>,
    active_slot_index: usize,
    active_row_index: usize,
    slot_view_offset: usize,
    row_view_offset: usize,
    last_visible_slot_count: usize,
    last_visible_row_count: usize,
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
            pool_surface: PoolSurface::Slots,
            active_breadcrumb_index: 0,
            projection_stack: Vec::new(),
            recent_escape_presses: Vec::new(),
            shape_picker,
            variant_picker: None,
            field_picker: None,
            function_picker: None,
            link_action_picker: None,
            rename_slot: None,
            slot_search: None,
            slot_axis: SlotAxis::Horizontal,
            focused_slot_fill: false,
            show_hotkey_help: false,
            slot_width: Self::MIN_SLOT_WIDTH,
            slot_height: Self::MIN_SLOT_HEIGHT,
            shape_choices,
            object_slots: Vec::new(),
            projection_cache: RefCell::new(ProjectionCache::default()),
            active_slot_index: 0,
            active_row_index: 0,
            slot_view_offset: 0,
            row_view_offset: 0,
            last_visible_slot_count: 1,
            last_visible_row_count: 1,
            next_slot_id: 1,
            status_message: "Left/Right: slots | Up/Down: rows | Type to jump | PageUp/PageDown: page | Enter/Space: act | Esc: back/exit"
                .to_string(),
        }
    }
}

impl ObjectBrowserApp {
    const FRAMES_PER_SECOND: f32 = 60.0;
    const MIN_SLOT_WIDTH: u16 = 34;
    const MIN_SLOT_HEIGHT: u16 = 7;

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
                    Ok(json) => match serde_json::from_str::<Value>(&json) {
                        Ok(value) => SlotRuntimeState::ResolvedValue { json, value },
                        Err(error) => SlotRuntimeState::Failed {
                            message: format!("could not parse invocation result json: {error}"),
                        },
                    },
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
                    SlotRuntimeState::ResolvedValue { json, value } => {
                        SlotRuntimeState::ResolvedValue {
                            json: json.clone(),
                            value: value.clone(),
                        }
                    }
                    SlotRuntimeState::Failed { message } => SlotRuntimeState::Failed {
                        message: message.clone(),
                    },
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
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [title_area, breadcrumb_area, body_area, status_area] = vertical.areas(frame.area());

        let title = Line::from("Cloud Terrastodon Object Pool")
            .centered()
            .bold();
        frame.render_widget(title, title_area);
        frame.render_widget(self.breadcrumbs_line(), breadcrumb_area);

        self.draw_pool(frame, body_area);

        frame.render_widget(Line::from(self.status_message.as_str()), status_area);

        match self.mode {
            UiMode::Pool | UiMode::SlotSearch => {}
            UiMode::ShapePicker => self.draw_shape_picker_popup(frame),
            UiMode::VariantPicker => self.draw_variant_picker_popup(frame),
            UiMode::FieldPicker => self.draw_field_picker_popup(frame),
            UiMode::FunctionPicker => self.draw_function_picker_popup(frame),
            UiMode::LinkActionPicker => self.draw_link_action_picker_popup(frame),
            UiMode::RenameSlot => self.draw_rename_slot_popup(frame),
        }

        if self.show_hotkey_help {
            self.draw_hotkey_help_popup(frame);
        }
    }

    fn draw_pool(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Object Pool");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let [top_margin_area, cards_area, bottom_margin_area] = if inner.height >= 5 {
            Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .areas(inner)
        } else {
            [Rect::default(), inner, Rect::default()]
        };
        if cards_area.width == 0 || cards_area.height == 0 {
            return;
        }

        let visible = self.visible_slot_range_for_area(cards_area);
        let constraints = vec![Constraint::Fill(1); visible.len()];
        let [left_marker_column, slot_layout_area, right_marker_column] =
            if self.slot_axis == SlotAxis::Vertical && cards_area.width >= 8 {
                Layout::horizontal([
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                ])
                .areas(cards_area)
            } else {
                [Rect::default(), cards_area, Rect::default()]
            };
        let slot_areas = match self.slot_axis {
            SlotAxis::Horizontal => Layout::horizontal(constraints.clone()).split(slot_layout_area),
            SlotAxis::Vertical => Layout::vertical(constraints.clone()).split(slot_layout_area),
        };
        let top_marker_areas =
            if self.slot_axis == SlotAxis::Horizontal && top_margin_area.height > 0 {
                Some(Layout::horizontal(constraints.clone()).split(top_margin_area))
            } else {
                None
            };
        let bottom_marker_areas =
            if self.slot_axis == SlotAxis::Horizontal && bottom_margin_area.height > 0 {
                Some(Layout::horizontal(constraints.clone()).split(bottom_margin_area))
            } else {
                None
            };
        let left_marker_areas =
            if self.slot_axis == SlotAxis::Vertical && left_marker_column.width > 0 {
                Some(Layout::vertical(constraints.clone()).split(left_marker_column))
            } else {
                None
            };
        let right_marker_areas =
            if self.slot_axis == SlotAxis::Vertical && right_marker_column.width > 0 {
                Some(Layout::vertical(constraints).split(right_marker_column))
            } else {
                None
            };

        for (offset, slot_index) in visible.clone().enumerate() {
            let Some(slot_area) = slot_areas.get(offset).copied() else {
                break;
            };
            let is_active = slot_index == self.active_slot_index;
            match self.pool_entry_at(slot_index) {
                Some(PoolEntry::NewSlot) => self.draw_new_slot(frame, slot_area, is_active),
                Some(PoolEntry::RealSlot(slot_id)) => {
                    self.draw_object_slot(frame, slot_area, slot_id, is_active);
                }
                Some(PoolEntry::Projection(projection)) => {
                    self.draw_projection_slot(frame, slot_area, &projection, is_active);
                }
                None => {}
            }

            if !is_active {
                continue;
            }

            let marker_style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);
            match self.slot_axis {
                SlotAxis::Horizontal => {
                    if let Some(areas) = &top_marker_areas
                        && let Some(marker_area) = areas.get(offset).copied()
                    {
                        frame.render_widget(
                            Paragraph::new(Line::from(Span::styled("vvvvv", marker_style)))
                                .alignment(Alignment::Center),
                            marker_area,
                        );
                    }
                    if let Some(areas) = &bottom_marker_areas
                        && let Some(marker_area) = areas.get(offset).copied()
                    {
                        frame.render_widget(
                            Paragraph::new(Line::from(Span::styled("^^^^^", marker_style)))
                                .alignment(Alignment::Center),
                            marker_area,
                        );
                    }
                }
                SlotAxis::Vertical => {
                    if let Some(areas) = &left_marker_areas
                        && let Some(marker_area) = areas.get(offset).copied()
                    {
                        frame.render_widget(
                            vertical_marker_paragraph(">", marker_area.height, marker_style),
                            marker_area,
                        );
                    }
                    if let Some(areas) = &right_marker_areas
                        && let Some(marker_area) = areas.get(offset).copied()
                    {
                        frame.render_widget(
                            vertical_marker_paragraph("<", marker_area.height, marker_style),
                            marker_area,
                        );
                    }
                }
            }
        }

        let max_visible = self.max_visible_slots_for_area(cards_area);
        if self.total_slot_count() > max_visible {
            let scrollbar_area = horizontal_scrollbar_overlay_area(area);
            if scrollbar_area.width > 0 && scrollbar_area.height > 0 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom);
                let mut scrollbar_state = ScrollbarState::new(self.total_slot_count())
                    .position(visible.start)
                    .viewport_content_length(max_visible);
                frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
            }
        }
    }
    fn draw_object_slot(&mut self, frame: &mut Frame, area: Rect, slot_id: usize, is_active: bool) {
        let Some(title) = self.slot_by_id(slot_id).map(|slot| {
            let slot_label = slot.name.as_deref().unwrap_or("unnamed");
            match slot.kind {
                SlotKind::Owned => format!("slot {} ({slot_label}) [owned]", slot.id),
                SlotKind::View(_) => format!("slot {} ({slot_label}) [view]", slot.id),
            }
        }) else {
            return;
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(self.slot_border_style(slot_id, is_active));
        let inner = block.inner(area);
        if is_active {
            self.last_visible_row_count = usize::from(inner.height).max(1);
            self.clamp_row_view_offset();
            self.ensure_active_row_visible();
        }
        let scroll_offset = if is_active { self.row_view_offset } else { 0 };
        let lines = self.slot_lines(slot_id, is_active, self.active_row_index);
        let paragraph = Paragraph::new(lines.clone())
            .block(block)
            .alignment(Alignment::Left)
            .scroll((scroll_offset.min(u16::MAX as usize) as u16, 0));
        frame.render_widget(paragraph, area);

        if is_active && usize::from(inner.height) > 0 && lines.len() > usize::from(inner.height) {
            let scrollbar_area = vertical_scrollbar_overlay_area(area);
            if scrollbar_area.width > 0 && scrollbar_area.height > 0 {
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                let mut scrollbar_state = ScrollbarState::new(lines.len())
                    .position(scroll_offset)
                    .viewport_content_length(usize::from(inner.height));
                frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
            }
        }
    }

    fn draw_new_slot(&mut self, frame: &mut Frame, area: Rect, is_active: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::LightTripleDashed)
            .title("new slot")
            .border_style(slot_border_style(Color::DarkGray, is_active));
        let inner = block.inner(area);
        if is_active {
            self.last_visible_row_count = usize::from(inner.height).max(1);
            self.clamp_row_view_offset();
        }
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
    fn draw_function_picker_popup(&mut self, frame: &mut Frame) {
        let Some(preview_lines) = self.function_picker_preview_lines() else {
            return;
        };
        let Some((items, total_count)) = self
            .function_picker
            .as_ref()
            .map(|picker| (picker.list_items(), picker.labels.len()))
        else {
            return;
        };
        let search = &mut self.function_picker.as_mut().expect("picker exists").search;
        draw_picker_popup(
            frame,
            "Pick Function",
            "Function Preview",
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

    fn draw_hotkey_help_popup(&self, frame: &mut Frame) {
        let area = centered_rect(48, 44, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Hotkeys")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines = vec![
            Line::from("Alt+/: toggle this help"),
            Line::from("Ctrl+T: transpose slot axis"),
            Line::from("Alt+F: focused slot fill"),
            Line::from("Alt++/Alt+-: resize slots"),
            Line::from("Left/Right: previous/next slot"),
            Line::from("Up/Down: previous/next row"),
            Line::from("PageUp/PageDown: page rows"),
            Line::from("Shift+PageUp/PageDown: page slots"),
            Line::from("Ctrl+Arrows: scroll viewport"),
            Line::from("Enter/Space: activate focused row"),
            Line::from("Type: search rows in active slot"),
            Line::from("Esc: back / close projection / exit"),
        ];
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
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
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('/') {
            self.show_hotkey_help = !self.show_hotkey_help;
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('-') {
            self.show_hotkey_help = false;
            self.resize_slots(-1);
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT)
            && matches!(key.code, KeyCode::Char('+') | KeyCode::Char('='))
        {
            self.show_hotkey_help = false;
            self.resize_slots(1);
            return;
        }
        if key.modifiers.contains(KeyModifiers::ALT)
            && matches!(key.code, KeyCode::Char('f') | KeyCode::Char('F'))
        {
            self.show_hotkey_help = false;
            self.toggle_focused_slot_fill();
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('t') | KeyCode::Char('T'))
        {
            self.show_hotkey_help = false;
            self.toggle_slot_axis();
            return;
        }

        match self.mode {
            UiMode::Pool => self.handle_pool_key(*key),
            UiMode::SlotSearch => self.handle_slot_search_key(*key),
            UiMode::ShapePicker => self.handle_shape_picker_key(*key),
            UiMode::VariantPicker => self.handle_variant_picker_key(*key),
            UiMode::FieldPicker => self.handle_field_picker_key(*key),
            UiMode::FunctionPicker => self.handle_function_picker_key(*key),
            UiMode::LinkActionPicker => self.handle_link_action_picker_key(*key),
            UiMode::RenameSlot => self.handle_rename_slot_key(*key),
        }
    }

    fn handle_pool_key(&mut self, key: KeyEvent) {
        if self.mode == UiMode::SlotSearch {
            self.handle_slot_search_key(key);
            return;
        }

        match self.pool_surface {
            PoolSurface::Breadcrumbs => match key.code {
                KeyCode::Esc => {
                    self.show_hotkey_help = false;
                    self.handle_escape();
                }
                KeyCode::Left => self.move_breadcrumb_left(),
                KeyCode::Right => self.move_breadcrumb_right(),
                KeyCode::Home => self.active_breadcrumb_index = 0,
                KeyCode::End => {
                    self.active_breadcrumb_index = self.breadcrumb_count().saturating_sub(1)
                }
                KeyCode::Down => self.pool_surface = PoolSurface::Slots,
                KeyCode::Enter | KeyCode::Char(' ') => self.activate_current_breadcrumb(),
                _ => {}
            },
            PoolSurface::Slots => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                match (ctrl, shift, key.code) {
                    (_, _, KeyCode::Esc) => {
                        self.show_hotkey_help = false;
                        self.handle_escape();
                    }
                    (true, _, KeyCode::Left) => self.shift_slot_view_left(1),
                    (true, _, KeyCode::Right) => self.shift_slot_view_right(1),
                    (true, _, KeyCode::Up) => self.shift_row_view_up(1),
                    (true, _, KeyCode::Down) => self.shift_row_view_down(1),
                    (_, true, KeyCode::Home) => self.move_slot_home(),
                    (_, true, KeyCode::End) => self.move_slot_end(),
                    (_, true, KeyCode::PageUp) => self.page_slots_left(),
                    (_, true, KeyCode::PageDown) => self.page_slots_right(),
                    (_, false, KeyCode::Home) => self.move_row_home(),
                    (_, false, KeyCode::End) => self.move_row_end(),
                    (_, _, KeyCode::Left) => self.move_slot_left(),
                    (_, _, KeyCode::Right) => self.move_slot_right(),
                    (_, _, KeyCode::Up) => self.move_row_up(),
                    (_, _, KeyCode::Down) => self.move_row_down(),
                    (_, false, KeyCode::PageUp) => self.page_rows_up(),
                    (_, false, KeyCode::PageDown) => self.page_rows_down(),
                    (_, _, KeyCode::Enter | KeyCode::Char(' ')) => self.activate_current_row(),
                    _ if self.is_slot_search_key(key) => self.start_slot_search(key),
                    _ => {}
                }
            }
        }
    }

    fn handle_slot_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.cancel_slot_search(),
            KeyCode::Enter => self.submit_slot_search(),
            KeyCode::Up => self.move_slot_search_selection(-1),
            KeyCode::Down => self.move_slot_search_selection(1),
            KeyCode::Home => self.move_slot_search_to_edge(true),
            KeyCode::End => self.move_slot_search_to_edge(false),
            KeyCode::PageUp => self.page_slot_search(-1),
            KeyCode::PageDown => self.page_slot_search(1),
            _ => {
                let Some(slot_search) = self.slot_search.as_mut() else {
                    self.mode = UiMode::Pool;
                    return;
                };
                if slot_search.query.input(key) {
                    slot_search.query.cancel_selection();
                    slot_search.query.move_cursor(CursorMove::End);
                    self.refresh_slot_search(false);
                }
            }
        }
    }

    fn handle_escape(&mut self) {
        if !self.projection_stack.is_empty() {
            self.pop_projection_breadcrumb();
            return;
        }

        let now = Instant::now();
        self.recent_escape_presses
            .retain(|pressed_at| now.duration_since(*pressed_at) <= Duration::from_secs(5));
        self.recent_escape_presses.push(now);
        self.pool_surface = PoolSurface::Slots;

        let remaining = 3usize.saturating_sub(self.recent_escape_presses.len());
        if remaining == 0 {
            self.should_quit = true;
            return;
        }

        self.status_message = format!(
            "Hit Esc {} more time{} within 5 seconds to exit.",
            remaining,
            if remaining == 1 { "" } else { "s" }
        );
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

    fn handle_function_picker_key(&mut self, key: KeyEvent) {
        let Some(function_picker) = self.function_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };

        match function_picker
            .search
            .handle_key(key, &function_picker.labels)
        {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.function_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "Function selection cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_function_picker_selection(),
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

    fn is_slot_search_key(&self, key: KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char(character) if character != ' ' && !character.is_control())
            && !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    }

    fn start_slot_search(&mut self, key: KeyEvent) {
        let Some(slot_id) = self.current_slot_id() else {
            return;
        };

        let mut query = build_text_area("");
        if !query.input(key) {
            return;
        }
        query.cancel_selection();
        query.move_cursor(CursorMove::End);

        self.slot_search = Some(SlotSearchState {
            slot_id,
            query,
            filtered_matches: Vec::new(),
            selected_match_index: 0,
        });
        self.mode = UiMode::SlotSearch;
        self.refresh_slot_search(false);
    }

    fn cancel_slot_search(&mut self) {
        self.slot_search = None;
        self.mode = UiMode::Pool;
        self.status_message = "Row search cancelled.".to_string();
    }

    fn submit_slot_search(&mut self) {
        let has_match = self.slot_search_current_target().is_some();
        self.slot_search = None;
        self.mode = UiMode::Pool;
        if has_match {
            self.activate_current_row();
        } else {
            self.status_message = "No matching rows in the active slot.".to_string();
        }
    }

    fn move_slot_search_selection(&mut self, direction: isize) {
        let Some(slot_search) = self.slot_search.as_mut() else {
            return;
        };
        if slot_search.filtered_matches.is_empty() {
            return;
        }

        let max_index = slot_search.filtered_matches.len().saturating_sub(1) as isize;
        let next_index =
            (slot_search.selected_match_index as isize + direction).clamp(0, max_index);
        slot_search.selected_match_index = next_index as usize;
        self.sync_slot_search_selection();
    }

    fn move_slot_search_to_edge(&mut self, to_start: bool) {
        let Some(slot_search) = self.slot_search.as_mut() else {
            return;
        };
        if slot_search.filtered_matches.is_empty() {
            return;
        }

        slot_search.selected_match_index = if to_start {
            0
        } else {
            slot_search.filtered_matches.len().saturating_sub(1)
        };
        self.sync_slot_search_selection();
    }

    fn page_slot_search(&mut self, direction: isize) {
        let Some(slot_search) = self.slot_search.as_mut() else {
            return;
        };
        if slot_search.filtered_matches.is_empty() {
            return;
        }

        let step = self.last_visible_row_count.saturating_sub(1).max(1) as isize;
        let max_index = slot_search.filtered_matches.len().saturating_sub(1) as isize;
        let next_index =
            (slot_search.selected_match_index as isize + direction * step).clamp(0, max_index);
        slot_search.selected_match_index = next_index as usize;
        self.sync_slot_search_selection();
    }

    fn refresh_slot_search(&mut self, preserve_current_selection: bool) {
        let Some((slot_id, query, preferred_target)) =
            self.slot_search.as_ref().map(|slot_search| {
                (
                    slot_search.slot_id,
                    slot_search.query.lines().join("\n"),
                    preserve_current_selection
                        .then(|| {
                            slot_search
                                .filtered_matches
                                .get(slot_search.selected_match_index)
                                .map(|matched| matched.target)
                        })
                        .flatten(),
                )
            })
        else {
            return;
        };

        let filtered_matches = self.slot_search_matches(slot_id, &query);
        let selected_match_index = preferred_target
            .and_then(|target| {
                filtered_matches
                    .iter()
                    .position(|matched| matched.target == target)
            })
            .unwrap_or(0);

        if let Some(slot_search) = self.slot_search.as_mut() {
            slot_search.filtered_matches = filtered_matches;
            slot_search.selected_match_index =
                selected_match_index.min(slot_search.filtered_matches.len().saturating_sub(1));
        }

        self.sync_slot_search_selection();
    }

    fn sync_slot_search_selection(&mut self) {
        if let Some(target) = self.slot_search_current_target()
            && let Some(slot_id) = self
                .slot_search
                .as_ref()
                .map(|slot_search| slot_search.slot_id)
        {
            self.active_row_index = self
                .focus_row_for_slot_target(slot_id, target)
                .unwrap_or(self.active_row_index);
        }
        self.ensure_active_row_visible();
        self.update_slot_search_status();
    }

    fn slot_search_current_target(&self) -> Option<SlotFocusTarget> {
        let slot_search = self.slot_search.as_ref()?;
        slot_search
            .filtered_matches
            .get(slot_search.selected_match_index)
            .map(|matched| matched.target)
    }

    fn update_slot_search_status(&mut self) {
        let Some(slot_search) = self.slot_search.as_ref() else {
            return;
        };
        let query = slot_search.query.lines().join("\n");
        let match_count = slot_search.filtered_matches.len();
        self.status_message = format!(
            "Search slot {}: {} ({} match{}) | Up/Down/PgUp/PgDn: navigate | Enter: activate | Esc: cancel",
            slot_search.slot_id,
            if query.is_empty() {
                "<empty>"
            } else {
                query.as_str()
            },
            match_count,
            if match_count == 1 { "" } else { "es" },
        );
    }

    fn allocate_slot_id(&mut self) -> usize {
        let next_available = self
            .object_slots
            .iter()
            .map(|slot| slot.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let slot_id = self.next_slot_id.max(next_available);
        self.next_slot_id = slot_id.saturating_add(1);
        slot_id
    }

    fn move_slot_left(&mut self) {
        self.active_slot_index = self.active_slot_index.saturating_sub(1);
        self.clamp_active_row();
        self.sync_selection_viewports();
    }

    fn move_slot_right(&mut self) {
        let max_index = self.total_slot_count().saturating_sub(1);
        self.active_slot_index = (self.active_slot_index + 1).min(max_index);
        self.clamp_active_row();
        self.sync_selection_viewports();
    }

    fn move_row_up(&mut self) {
        if self.active_row_index == 0 {
            self.pool_surface = PoolSurface::Breadcrumbs;
            self.active_breadcrumb_index = self
                .projection_stack
                .len()
                .min(self.breadcrumb_count().saturating_sub(1));
            return;
        }
        self.active_row_index = self.active_row_index.saturating_sub(1);
        self.ensure_active_row_visible();
    }

    fn move_row_down(&mut self) {
        let max_row = self.active_focusable_rows().saturating_sub(1);
        self.active_row_index = (self.active_row_index + 1).min(max_row);
        self.ensure_active_row_visible();
    }

    fn move_row_home(&mut self) {
        self.active_row_index = 0;
        self.ensure_active_row_visible();
    }

    fn move_row_end(&mut self) {
        self.active_row_index = self.active_focusable_rows().saturating_sub(1);
        self.ensure_active_row_visible();
    }

    fn move_slot_home(&mut self) {
        self.active_slot_index = 0;
        self.clamp_active_row();
        self.sync_selection_viewports();
    }

    fn move_slot_end(&mut self) {
        self.active_slot_index = self.total_slot_count().saturating_sub(1);
        self.clamp_active_row();
        self.sync_selection_viewports();
    }

    fn page_rows_up(&mut self) {
        let step = self.last_visible_row_count.saturating_sub(1).max(1);
        self.active_row_index = self.active_row_index.saturating_sub(step);
        self.ensure_active_row_visible();
    }

    fn page_rows_down(&mut self) {
        let step = self.last_visible_row_count.saturating_sub(1).max(1);
        let max_row = self.active_focusable_rows().saturating_sub(1);
        self.active_row_index = (self.active_row_index + step).min(max_row);
        self.ensure_active_row_visible();
    }

    fn page_slots_left(&mut self) {
        let step = self.last_visible_slot_count.saturating_sub(1).max(1);
        self.active_slot_index = self.active_slot_index.saturating_sub(step);
        self.clamp_active_row();
        self.sync_selection_viewports();
    }

    fn page_slots_right(&mut self) {
        let step = self.last_visible_slot_count.saturating_sub(1).max(1);
        let max_index = self.total_slot_count().saturating_sub(1);
        self.active_slot_index = (self.active_slot_index + step).min(max_index);
        self.clamp_active_row();
        self.sync_selection_viewports();
    }

    fn shift_slot_view_left(&mut self, amount: usize) {
        self.slot_view_offset = self.slot_view_offset.saturating_sub(amount);
        self.clamp_slot_view_offset();
    }

    fn shift_slot_view_right(&mut self, amount: usize) {
        let visible = self.last_visible_slot_count.max(1);
        let max_start = self.total_slot_count().saturating_sub(visible);
        self.slot_view_offset = (self.slot_view_offset + amount).min(max_start);
    }

    fn shift_row_view_up(&mut self, amount: usize) {
        self.row_view_offset = self.row_view_offset.saturating_sub(amount);
        self.clamp_row_view_offset();
    }

    fn shift_row_view_down(&mut self, amount: usize) {
        let visible = self.last_visible_row_count.max(1);
        let max_start = self.active_rendered_line_count().saturating_sub(visible);
        self.row_view_offset = (self.row_view_offset + amount).min(max_start);
    }

    fn toggle_slot_axis(&mut self) {
        self.slot_axis = match self.slot_axis {
            SlotAxis::Horizontal => SlotAxis::Vertical,
            SlotAxis::Vertical => SlotAxis::Horizontal,
        };
        self.sync_selection_viewports();
        self.status_message = format!("Slot axis: {}.", self.slot_axis.label());
    }

    fn toggle_focused_slot_fill(&mut self) {
        self.focused_slot_fill = !self.focused_slot_fill;
        self.sync_selection_viewports();
        self.status_message = if self.focused_slot_fill {
            "Focused slot fill enabled.".to_string()
        } else {
            "Focused slot fill disabled.".to_string()
        };
    }

    fn resize_slots(&mut self, direction: isize) {
        match self.slot_axis {
            SlotAxis::Horizontal => {
                self.slot_width =
                    resize_dimension(self.slot_width, Self::MIN_SLOT_WIDTH, 8, direction);
                self.status_message = format!("Slot width: {}.", self.slot_width);
            }
            SlotAxis::Vertical => {
                self.slot_height =
                    resize_dimension(self.slot_height, Self::MIN_SLOT_HEIGHT, 2, direction);
                self.status_message = format!("Slot height: {}.", self.slot_height);
            }
        }
        self.sync_selection_viewports();
    }

    fn sync_selection_viewports(&mut self) {
        self.ensure_active_slot_visible();
        self.ensure_active_row_visible();
    }

    fn clamp_slot_view_offset(&mut self) {
        let visible = self.last_visible_slot_count.max(1);
        let max_start = self.total_slot_count().saturating_sub(visible);
        self.slot_view_offset = self.slot_view_offset.min(max_start);
    }

    fn clamp_row_view_offset(&mut self) {
        let visible = self.last_visible_row_count.max(1);
        let max_start = self.active_rendered_line_count().saturating_sub(visible);
        self.row_view_offset = self.row_view_offset.min(max_start);
    }

    fn ensure_active_slot_visible(&mut self) {
        let visible = self.last_visible_slot_count.max(1);
        if self.active_slot_index < self.slot_view_offset {
            self.slot_view_offset = self.active_slot_index;
        } else if self.active_slot_index >= self.slot_view_offset + visible {
            self.slot_view_offset = self.active_slot_index + 1 - visible;
        }
        self.clamp_slot_view_offset();
    }

    fn ensure_active_row_visible(&mut self) {
        let visible = self.last_visible_row_count.max(1);
        let active_line_index = self.active_line_index();
        if active_line_index < self.row_view_offset {
            self.row_view_offset = active_line_index;
        } else if active_line_index >= self.row_view_offset + visible {
            self.row_view_offset = active_line_index + 1 - visible;
        }
        self.clamp_row_view_offset();
    }

    fn active_rendered_line_count(&mut self) -> usize {
        match self.current_pool_entry() {
            Some(PoolEntry::RealSlot(slot_id)) => self
                .slot_search
                .as_ref()
                .filter(|slot_search| slot_search.slot_id == slot_id)
                .map(|slot_search| slot_search.filtered_matches.len().max(1))
                .unwrap_or_else(|| self.slot_display_rows(slot_id).len()),
            Some(PoolEntry::Projection(projection)) => {
                self.projection_rendered_line_count(&projection)
            }
            Some(PoolEntry::NewSlot) | None => 1,
        }
    }

    fn active_line_index(&mut self) -> usize {
        match self.current_pool_entry() {
            Some(PoolEntry::RealSlot(slot_id)) => {
                if let Some(slot_search) = self
                    .slot_search
                    .as_ref()
                    .filter(|slot_search| slot_search.slot_id == slot_id)
                {
                    return if slot_search.filtered_matches.is_empty() {
                        0
                    } else {
                        slot_search.selected_match_index
                    };
                }
                self.slot_line_index_for_row(slot_id, self.active_row_index)
            }
            Some(PoolEntry::Projection(projection)) => {
                self.projection_line_index(&projection, self.active_row_index)
            }
            Some(PoolEntry::NewSlot) | None => 0,
        }
    }

    fn slot_line_index_for_row(&mut self, slot_id: usize, active_row: usize) -> usize {
        let Some(active_target) = self.slot_focus_targets(slot_id).get(active_row).copied() else {
            return 0;
        };
        self.slot_display_rows(slot_id)
            .iter()
            .position(|row| {
                matches!(
                    row,
                    SlotDisplayRow::Focusable { target, .. } if *target == active_target
                )
            })
            .unwrap_or(0)
    }

    fn projection_rendered_line_count(&self, projection: &ProjectionSlot) -> usize {
        match self.projection_value(projection) {
            Some(Value::Object(object)) if !object.is_empty() => 2 + object.len() * 2,
            Some(_) => 3,
            None => 1,
        }
    }

    fn projection_line_index(&self, projection: &ProjectionSlot, active_row: usize) -> usize {
        match self.projection_value(projection) {
            Some(Value::Object(object)) if !object.is_empty() && active_row > 0 => (active_row + 1)
                .min(
                    self.projection_rendered_line_count(projection)
                        .saturating_sub(1),
                ),
            Some(_) | None => active_row.min(
                self.projection_rendered_line_count(projection)
                    .saturating_sub(1),
            ),
        }
    }

    fn activate_current_row(&mut self) {
        let Some(entry) = self.current_pool_entry() else {
            return;
        };

        match entry {
            PoolEntry::NewSlot => self.append_slot(),
            PoolEntry::Projection(projection) => {
                self.activate_projection_slot_row(&projection, self.active_row_index);
            }
            PoolEntry::RealSlot(slot_id) => {
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
                    SlotFocusTarget::FieldValue(field_index) => {
                        self.activate_field_value(field_index)
                    }
                    SlotFocusTarget::Inlink(inlink_index) => {
                        self.activate_inlink(slot_id, inlink_index)
                    }
                    SlotFocusTarget::CreatedFor => self.activate_created_for(slot_id),
                    SlotFocusTarget::ProducedBy => self.activate_produced_by(slot_id),
                    SlotFocusTarget::RuntimeValue => self.activate_runtime_value(slot_id),
                    SlotFocusTarget::Result(result_index) => {
                        self.activate_result(slot_id, result_index)
                    }
                    SlotFocusTarget::Action(action) => self.activate_slot_action(slot_id, action),
                }
            }
        }
    }
    fn append_slot(&mut self) {
        let slot_id = self.allocate_slot_id();
        let slot = ObjectSlot::new(slot_id);
        self.object_slots.push(slot);
        self.active_slot_index = self.total_slot_count().saturating_sub(2);
        self.active_row_index = 0;
        self.sync_selection_viewports();
        self.status_message = format!(
            "Created slot {}. Pick a shape on the highlighted row.",
            slot_id
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

        let required_shape_name = field.info.field_shape_name.clone();
        let mut choices = self
            .matching_slot_ids(&required_shape_name, owner_slot_id)
            .into_iter()
            .map(|slot_id| FieldPickerChoice::ExistingSlot { slot_id })
            .collect::<Vec<_>>();
        choices.extend(
            self.existing_producer_slot_ids(&required_shape_name, owner_slot_id)
                .into_iter()
                .map(|slot_id| FieldPickerChoice::ExistingProducerSlot { slot_id }),
        );
        let (arbitrary_producer_choices, regular_producer_choices): (Vec<_>, Vec<_>) = self
            .producer_function_choices_for(&required_shape_name)
            .into_iter()
            .partition(field_picker_choice_is_arbitrary_producer);
        choices.extend(regular_producer_choices);
        choices.push(FieldPickerChoice::CreateNew);
        choices.extend(arbitrary_producer_choices);

        let labels = choices
            .iter()
            .map(|choice| self.field_picker_label(choice, &required_shape_name))
            .collect::<Vec<_>>();
        let preferred_index = match field.value_state {
            FieldValueState::Linked { slot_id } => choices
                .iter()
                .position(|choice| choice == &FieldPickerChoice::ExistingSlot { slot_id }),
            _ => choices
                .iter()
                .position(|choice| matches!(choice, FieldPickerChoice::ExistingSlot { .. }))
                .or_else(|| {
                    choices.iter().position(|choice| {
                        matches!(choice, FieldPickerChoice::ExistingProducerSlot { .. })
                    })
                })
                .or_else(|| {
                    choices.iter().position(|choice| {
                        matches!(choice, FieldPickerChoice::CreateProducer { .. })
                            && !field_picker_choice_is_arbitrary_producer(choice)
                    })
                })
                .or_else(|| {
                    choices
                        .iter()
                        .position(|choice| choice == &FieldPickerChoice::CreateNew)
                })
                .or_else(|| {
                    choices.iter().position(|choice| {
                        matches!(choice, FieldPickerChoice::CreateProducer { .. })
                            && field_picker_choice_is_arbitrary_producer(choice)
                    })
                }),
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
            "Choose an object for the field, or a request that can produce one. Type to search; PgUp/PgDn scrolls the preview pane."
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
            SlotAction::InvokeArbitrary => self.invoke_arbitrary_slot(slot_id),
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
        let new_slot_id = self.allocate_slot_id();
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
            slot.runtime_state = snapshot.value_json.and_then(|json| {
                serde_json::from_str::<Value>(&json)
                    .ok()
                    .map(|value| SlotRuntimeState::ResolvedValue { json, value })
            });
        }
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot(slot_id);
        self.status_message = format!(
            "Took slot {} out of slot {}.{} and made it owned.",
            slot_id, info.owner_slot_id, info.field_name
        );
    }

    fn activate_created_for(&mut self, slot_id: usize) {
        let Some(created_for) = self.slot_by_id(slot_id).and_then(|slot| slot.created_for) else {
            return;
        };
        self.jump_to_slot_target(
            created_for.owner_slot_id,
            SlotFocusTarget::FieldValue(created_for.field_index),
        );
        self.status_message = format!(
            "Jumped to slot {}.{}.",
            created_for.owner_slot_id, created_for.field_name
        );
    }

    fn activate_produced_by(&mut self, slot_id: usize) {
        let Some(produced_by_slot_id) = self
            .slot_by_id(slot_id)
            .and_then(|slot| slot.produced_by_slot_id)
        else {
            return;
        };
        self.jump_to_slot(produced_by_slot_id);
        self.status_message = format!("Jumped to producer slot {}.", produced_by_slot_id);
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
        let functions = self.applicable_functions_for_slot(slot_id);
        match functions.as_slice() {
            [] => {
                self.status_message =
                    "No registered functions are available for this slot.".to_string()
            }
            [function] => self.invoke_registered_function(slot_id, function),
            _ => {
                let labels = functions
                    .iter()
                    .map(|function| self.function_picker_label(function))
                    .collect();
                self.function_picker = Some(FunctionPickerState::new(
                    FunctionPickerTarget::InvokeSlot(slot_id),
                    functions,
                    labels,
                ));
                self.mode = UiMode::FunctionPicker;
                self.status_message = "Select a function to invoke.".to_string();
            }
        }
    }

    fn invoke_arbitrary_slot(&mut self, slot_id: usize) {
        let functions = self.applicable_arbitrary_functions_for_slot(slot_id);
        match functions.as_slice() {
            [] => {
                self.status_message =
                    "No fake response generators are available for this slot.".to_string()
            }
            [function] => self.invoke_arbitrary_registered_function(slot_id, function),
            _ => {
                let labels = functions
                    .iter()
                    .map(|function| self.function_picker_label(function))
                    .collect();
                self.function_picker = Some(FunctionPickerState::new(
                    FunctionPickerTarget::InvokeArbitrarySlot(slot_id),
                    functions,
                    labels,
                ));
                self.mode = UiMode::FunctionPicker;
                self.status_message = "Select a request to fake-invoke.".to_string();
            }
        }
    }

    fn applicable_functions_for_slot(&self, slot_id: usize) -> Vec<&'static Function> {
        let Some(shape_name) = self.slot_shape_name(slot_id) else {
            return Vec::new();
        };
        let Some(thing) = self.thing_for_shape_name(shape_name) else {
            return Vec::new();
        };
        let slot_is_owned = matches!(
            self.slot_by_id(slot_id).map(|slot| &slot.kind),
            Some(SlotKind::Owned)
        );
        functions_from(thing.shape)
            .into_iter()
            .filter(|function| function.supports_slot_kind(slot_is_owned))
            .collect()
    }

    fn applicable_arbitrary_functions_for_slot(&self, slot_id: usize) -> Vec<&'static Function> {
        let Some(shape_name) = self.slot_shape_name(slot_id) else {
            return Vec::new();
        };
        let Some(thing) = self.thing_for_shape_name(shape_name) else {
            return Vec::new();
        };
        let slot_is_owned = matches!(
            self.slot_by_id(slot_id).map(|slot| &slot.kind),
            Some(SlotKind::Owned)
        );
        functions_from(thing.shape)
            .into_iter()
            .filter(|function| function.supports_slot_kind(slot_is_owned))
            .filter(|function| {
                functions_to(function.output_shape)
                    .into_iter()
                    .any(|candidate| describe_shape(candidate.input_shape) == "ArbitraryBytes")
            })
            .collect()
    }

    fn apply_function_picker_selection(&mut self) {
        let Some((target, function)) = self
            .function_picker
            .as_ref()
            .and_then(|picker| Some((picker.target, picker.selected_function()?)))
        else {
            self.status_message = "No function is selected.".to_string();
            return;
        };
        self.function_picker = None;
        self.mode = UiMode::Pool;
        match target {
            FunctionPickerTarget::InvokeSlot(slot_id) => {
                self.invoke_registered_function(slot_id, function)
            }
            FunctionPickerTarget::InvokeArbitrarySlot(slot_id) => {
                self.invoke_arbitrary_registered_function(slot_id, function)
            }
        }
    }

    fn invoke_registered_function(&mut self, slot_id: usize, function: &'static Function) {
        let Some(shape_name) = self.slot_shape_name(slot_id).map(str::to_string) else {
            self.status_message = "Pick a shape before invoking.".to_string();
            return;
        };
        let Some(thing) = self.thing_for_shape_name(&shape_name) else {
            self.status_message = format!("{shape_name} is not a registered input shape.");
            return;
        };
        let json = match self.slot_json_string(slot_id) {
            Ok(json) => json,
            Err(error) => {
                self.status_message = format!("Could not invoke slot {}: {error}", slot_id);
                return;
            }
        };
        let mut input = match thing.parse_boxed(&json) {
            Ok(input) => input,
            Err(error) => {
                self.status_message = format!("Could not build {shape_name} input: {error}");
                return;
            }
        };

        let invocation = match function.receiver_mode {
            ReceiverMode::ByValue => function.invoke_value_boxed(input),
            ReceiverMode::ByRef => function
                .invoke_ref_boxed(input.as_ref())
                .map(FunctionInvocation::Ready),
            ReceiverMode::ByMut => {
                if !matches!(
                    self.slot_by_id(slot_id).map(|slot| &slot.kind),
                    Some(SlotKind::Owned)
                ) {
                    self.status_message = "Mutable functions require an owned slot.".to_string();
                    return;
                }
                let output = match function.invoke_mut_boxed(input.as_mut()) {
                    Ok(output) => output,
                    Err(error) => {
                        self.status_message =
                            format!("Could not invoke {}: {error}", describe_function(function));
                        return;
                    }
                };
                if let Err(error) =
                    self.update_slot_runtime_from_typed(slot_id, thing, input.as_ref())
                {
                    self.status_message = format!(
                        "Function updated the input but it could not be re-serialized: {error}"
                    );
                    return;
                }
                Ok(FunctionInvocation::Ready(output))
            }
        };

        let invocation = match invocation {
            Ok(invocation) => invocation,
            Err(error) => {
                self.status_message =
                    format!("Could not invoke {}: {error}", describe_function(function));
                return;
            }
        };

        match invocation {
            FunctionInvocation::Pending(future) => {
                let result_slot_id = self.allocate_slot_id();
                let output_shape_name = describe_shape(function.output_shape);
                let pending = PendingInvocationState {
                    join_handle: tokio::spawn(future),
                    output_serialize: function.output_serialize,
                };
                let mut result_slot =
                    ObjectSlot::new_result(result_slot_id, output_shape_name.clone(), pending);
                result_slot.produced_by_slot_id = Some(slot_id);
                self.object_slots.push(result_slot);
                if let Some(slot) = self.slot_by_id_mut(slot_id) {
                    slot.result_slot_ids.push(result_slot_id);
                }
                self.invalidate_all_slot_display_caches();
                self.jump_to_slot(result_slot_id);
                self.status_message = format!(
                    "Invoked {} and started result slot {}.",
                    describe_function(function),
                    result_slot_id
                );
            }
            FunctionInvocation::Ready(output) => {
                self.finish_ready_function_output(slot_id, function, output)
            }
        }
    }

    fn update_slot_runtime_from_typed(
        &mut self,
        slot_id: usize,
        thing: &'static cloud_terrastodon_registry::Thing,
        value: &(dyn std::any::Any + Send),
    ) -> Result<()> {
        let json = thing.serialize_boxed(value)?;
        let value = serde_json::from_str::<Value>(&json)?;
        if let Some(slot) = self.slot_by_id_mut(slot_id) {
            slot.runtime_state = Some(SlotRuntimeState::ResolvedValue { json, value });
        }
        Ok(())
    }

    fn finish_ready_function_output(
        &mut self,
        slot_id: usize,
        function: &'static Function,
        output: Box<dyn std::any::Any + Send>,
    ) {
        let json = match (function.output_serialize)(output.as_ref()) {
            Ok(json) => json,
            Err(error) => {
                self.status_message = format!(
                    "Could not serialize {} output: {error}",
                    describe_function(function)
                );
                return;
            }
        };
        let value = match serde_json::from_str::<Value>(&json) {
            Ok(value) => value,
            Err(error) => {
                self.status_message = format!(
                    "Could not parse {} output json: {error}",
                    describe_function(function)
                );
                return;
            }
        };
        let result_slot_id = self.allocate_slot_id();
        let output_shape_name = describe_shape(function.output_shape);
        let mut result_slot =
            ObjectSlot::new_resolved_result(result_slot_id, output_shape_name.clone(), json, value);
        result_slot.produced_by_slot_id = Some(slot_id);
        self.object_slots.push(result_slot);
        if let Some(slot) = self.slot_by_id_mut(slot_id) {
            slot.result_slot_ids.push(result_slot_id);
        }
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot(result_slot_id);
        self.status_message = format!(
            "Invoked {} into result slot {}.",
            describe_function(function),
            result_slot_id
        );
    }

    fn invoke_arbitrary_registered_function(
        &mut self,
        slot_id: usize,
        function: &'static Function,
    ) {
        let constructors = functions_to(function.output_shape)
            .into_iter()
            .filter(|candidate| describe_shape(candidate.input_shape) == "ArbitraryBytes")
            .collect::<Vec<_>>();
        let Some(constructor) = constructors.first().copied() else {
            self.status_message = format!(
                "No fake response generator is registered for {}.",
                describe_shape(function.output_shape)
            );
            return;
        };

        let mut input: Box<dyn std::any::Any + Send> = Box::new(ArbitraryBytes::new(vec![0; 4096]));
        let FunctionInvocation::Ready(output) = (match constructor.invoke_mut_boxed(input.as_mut())
        {
            Ok(output) => FunctionInvocation::Ready(output),
            Err(error) => {
                self.status_message = format!(
                    "Could not fake-invoke {}: {error}",
                    describe_function(function)
                );
                return;
            }
        }) else {
            unreachable!("arbitrary constructors are synchronous")
        };

        self.finish_ready_function_output(slot_id, constructor, output);
        self.status_message = format!(
            "Fake-invoked {} into a {} result.",
            describe_function(function),
            describe_shape(function.output_shape)
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
        self.ensure_active_row_visible();
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
        self.ensure_active_row_visible();
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
            FieldPickerChoice::ExistingSlot { slot_id } => {
                self.open_link_action_picker(owner_slot_id, field_index, slot_id)
            }
            FieldPickerChoice::ExistingProducerSlot { slot_id } => {
                self.jump_to_existing_producer_slot(slot_id, &required_shape_name)
            }
            FieldPickerChoice::CreateProducer {
                input_shape_name, ..
            } => self.create_producer_request_for_field(
                owner_slot_id,
                field_index,
                &input_shape_name,
                &required_shape_name,
            ),
            FieldPickerChoice::CreateNew => {
                self.create_field_object(owner_slot_id, field_index, &required_shape_name)
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

    fn jump_to_existing_producer_slot(&mut self, slot_id: usize, required_shape_name: &str) {
        self.jump_to_slot(slot_id);
        self.status_message = format!(
            "Jumped to source slot {}. It can produce {}.",
            slot_id, required_shape_name
        );
    }

    fn create_producer_request_for_field(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        input_shape_name: &str,
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
            .find(|shape| shape.label == input_shape_name)
            .cloned()
        else {
            self.status_message = format!(
                "{} is not currently registered as a constructible source shape.",
                input_shape_name
            );
            return;
        };

        let slot_id = self.allocate_slot_id();

        let mut slot = ObjectSlot::new(slot_id);
        slot.apply_shape_choice(&choice);
        slot.created_for = Some(SlotCreatedFor {
            owner_slot_id,
            field_index,
            field_name,
        });
        let focus_target = slot.default_focus_target();

        self.object_slots.push(slot);
        self.invalidate_all_slot_display_caches();
        self.jump_to_slot_target(slot_id, focus_target);
        self.status_message = format!(
            "Created source slot {} ({}) to produce {} for slot {}.{}.",
            slot_id, input_shape_name, required_shape_name, owner_slot_id, field_name
        );
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

        let slot_id = self.allocate_slot_id();

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

        let slot_id = self.allocate_slot_id();
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
        if let Some(field) = self.slot_field_mut(owner_slot_id, field_index)
            && matches!(
                field.value_state,
                FieldValueState::Linked {
                    slot_id: linked_slot_id,
                } if linked_slot_id == slot_id
            )
        {
            reset_field_value(field);
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
                Some(SlotRuntimeState::ResolvedValue { json, .. }) => Some(json.clone()),
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
            if to_remove.contains(&slot.id)
                && let Some(SlotRuntimeState::Pending(pending)) = &slot.runtime_state
            {
                pending.join_handle.abort();
            }
        }
        self.object_slots
            .retain(|slot| !to_remove.contains(&slot.id));
        for slot in &mut self.object_slots {
            slot.result_slot_ids
                .retain(|slot_id| !to_remove.contains(slot_id));
            if slot
                .produced_by_slot_id
                .is_some_and(|produced_by_slot_id| to_remove.contains(&produced_by_slot_id))
            {
                slot.produced_by_slot_id = None;
            }
        }

        self.invalidate_all_slot_display_caches();
        let max_index = self.total_slot_count().saturating_sub(1);
        self.active_slot_index = self.active_slot_index.min(max_index);
        self.clamp_active_row();
        self.sync_selection_viewports();
    }
    fn total_slot_count(&self) -> usize {
        self.current_projection_view()
            .map(|view| self.projection_view_slot_count(view))
            .unwrap_or_else(|| {
                self.object_slots
                    .iter()
                    .map(|slot| 1 + self.top_level_projection_child_count(slot.id))
                    .sum::<usize>()
                    + 1
            })
    }
    fn current_pool_entry(&self) -> Option<PoolEntry> {
        self.pool_entry_at(self.active_slot_index)
    }

    fn current_slot(&self) -> Option<&ObjectSlot> {
        self.current_slot_id()
            .and_then(|slot_id| self.slot_by_id(slot_id))
    }

    fn current_slot_id(&self) -> Option<usize> {
        match self.current_pool_entry()? {
            PoolEntry::RealSlot(slot_id) => Some(slot_id),
            PoolEntry::NewSlot | PoolEntry::Projection(_) => None,
        }
    }

    fn active_focusable_rows(&self) -> usize {
        match self.current_pool_entry() {
            Some(PoolEntry::RealSlot(slot_id)) => self.slot_focus_targets(slot_id).len(),
            Some(PoolEntry::NewSlot) => 1,
            Some(PoolEntry::Projection(projection)) => self.projection_focusable_rows(&projection),
            None => 1,
        }
    }

    fn clamp_active_row(&mut self) {
        let max_row = self.active_focusable_rows().saturating_sub(1);
        self.active_row_index = self.active_row_index.min(max_row);
    }

    fn max_visible_slots(&self, width: u16) -> usize {
        usize::from((width / self.slot_width.max(1)).max(1))
    }

    fn max_visible_slots_for_area(&self, area: Rect) -> usize {
        if self.focused_slot_fill {
            return 1;
        }
        match self.slot_axis {
            SlotAxis::Horizontal => self.max_visible_slots(area.width),
            SlotAxis::Vertical => usize::from((area.height / self.slot_height.max(1)).max(1)),
        }
    }

    fn visible_slot_range_for_area(&mut self, area: Rect) -> Range<usize> {
        let total = self.total_slot_count();
        let max_visible = self.max_visible_slots_for_area(area);
        self.last_visible_slot_count = max_visible.max(1);
        if self.focused_slot_fill {
            self.slot_view_offset = self.active_slot_index.min(total.saturating_sub(1));
            return self.slot_view_offset..(self.slot_view_offset + 1).min(total);
        }
        if total <= max_visible {
            self.slot_view_offset = 0;
            return 0..total;
        }

        self.clamp_slot_view_offset();
        let start = self.slot_view_offset.min(total.saturating_sub(max_visible));
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

    fn top_level_projection_child_count(&self, slot_id: usize) -> usize {
        self.projection_child_count(&JsonProjectionView {
            root_slot_id: slot_id,
            path: Vec::new(),
        })
    }

    fn top_level_pool_index_for_slot(&self, slot_id: usize) -> Option<usize> {
        let mut slot_index = 0;
        for slot in &self.object_slots {
            if slot.id == slot_id {
                return Some(slot_index);
            }
            slot_index += 1 + self.top_level_projection_child_count(slot.id);
        }
        None
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
            Some(SlotRuntimeState::ResolvedValue { value, .. }) => {
                return Ok(value.clone());
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
        let has_activity = slot.created_for.is_some()
            || slot.produced_by_slot_id.is_some()
            || !inlinks.is_empty()
            || !slot.result_slot_ids.is_empty();
        if has_activity {
            rows.push(SlotDisplayRow::Static(separator_line("activity")));
            if let Some(created_for) = slot.created_for {
                rows.push(focusable_plain_row(
                    SlotFocusTarget::CreatedFor,
                    format!(
                        "created for slot {}.{}",
                        created_for.owner_slot_id, created_for.field_name
                    ),
                ));
            }
            if let Some(produced_by_slot_id) = slot.produced_by_slot_id {
                rows.push(focusable_plain_row(
                    SlotFocusTarget::ProducedBy,
                    format!("produced by slot {}", produced_by_slot_id),
                ));
            }
            for (index, inlink) in inlinks.iter().enumerate() {
                rows.push(focusable_plain_row(
                    SlotFocusTarget::Inlink(index),
                    format!(
                        "used by slot {}.{}",
                        inlink.owner_slot_id, inlink.field_name
                    ),
                ));
            }
            for (index, result_slot_id) in slot.result_slot_ids.iter().copied().enumerate() {
                rows.push(focusable_plain_row(
                    SlotFocusTarget::Result(index),
                    format!("produced {}", self.result_slot_label(result_slot_id)),
                ));
            }
        }

        if let Some(runtime_state) = self.slot_runtime_state(slot_id) {
            rows.push(SlotDisplayRow::Static(separator_line("status")));
            rows.extend(runtime_state_rows(runtime_state));
        }

        rows.push(SlotDisplayRow::Static(separator_line("actions")));
        for action in [
            SlotAction::Rename,
            SlotAction::Delete,
            SlotAction::Clone,
            SlotAction::Take,
            SlotAction::Invoke,
            SlotAction::InvokeArbitrary,
        ] {
            if action == SlotAction::Invoke
                && self.applicable_functions_for_slot(slot_id).is_empty()
            {
                continue;
            }
            if action == SlotAction::InvokeArbitrary
                && self
                    .applicable_arbitrary_functions_for_slot(slot_id)
                    .is_empty()
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
        self.projection_cache.borrow_mut().clear();
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

        if self
            .slot_by_id(slot_id)
            .and_then(|slot| slot.created_for)
            .is_some()
        {
            targets.push(SlotFocusTarget::CreatedFor);
        }

        if self
            .slot_by_id(slot_id)
            .and_then(|slot| slot.produced_by_slot_id)
            .is_some()
        {
            targets.push(SlotFocusTarget::ProducedBy);
        }

        for (index, _) in self.slot_inlinks(slot_id).iter().enumerate() {
            targets.push(SlotFocusTarget::Inlink(index));
        }

        if let Some(slot) = self.slot_by_id(slot_id) {
            for (index, _) in slot.result_slot_ids.iter().enumerate() {
                targets.push(SlotFocusTarget::Result(index));
            }
        }

        if matches!(
            self.slot_runtime_state(slot_id),
            Some(SlotRuntimeState::ResolvedValue { .. })
        ) {
            targets.push(SlotFocusTarget::RuntimeValue);
        }

        targets.extend([
            SlotFocusTarget::Action(SlotAction::Rename),
            SlotFocusTarget::Action(SlotAction::Delete),
            SlotFocusTarget::Action(SlotAction::Clone),
            SlotFocusTarget::Action(SlotAction::Take),
        ]);
        if !self.applicable_functions_for_slot(slot_id).is_empty() {
            targets.push(SlotFocusTarget::Action(SlotAction::Invoke));
        }
        if !self
            .applicable_arbitrary_functions_for_slot(slot_id)
            .is_empty()
        {
            targets.push(SlotFocusTarget::Action(SlotAction::InvokeArbitrary));
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
        let Some(slot_index) = self.top_level_pool_index_for_slot(slot_id) else {
            return;
        };
        self.projection_stack.clear();
        self.pool_surface = PoolSurface::Slots;
        self.active_slot_index = slot_index;
        self.active_row_index = self.focus_row_for_slot_target(slot_id, target).unwrap_or(0);
        self.sync_selection_viewports();
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

    fn existing_producer_slot_ids(
        &self,
        required_shape_name: &str,
        owner_slot_id: usize,
    ) -> Vec<usize> {
        let Some(required_thing) = self.thing_for_shape_name(required_shape_name) else {
            return Vec::new();
        };

        self.object_slots
            .iter()
            .filter(|slot| slot.id != owner_slot_id)
            .filter_map(|slot| {
                let shape_name = self.slot_shape_name(slot.id)?;
                let thing = self.thing_for_shape_name(shape_name)?;
                functions_from(thing.shape)
                    .into_iter()
                    .any(|function| {
                        function.production_kind(required_thing.shape)
                            == Some(ProductionKind::Exact)
                    })
                    .then_some(slot.id)
            })
            .collect()
    }

    fn producer_function_choices_for(&self, required_shape_name: &str) -> Vec<FieldPickerChoice> {
        let Some(required_thing) = self.thing_for_shape_name(required_shape_name) else {
            return Vec::new();
        };

        let mut seen = BTreeSet::new();
        let mut choices = Vec::new();
        for function in functions_to(required_thing.shape) {
            if function.production_kind(required_thing.shape) != Some(ProductionKind::Exact) {
                continue;
            }
            let input_shape_name = describe_shape(function.input_shape);
            let function_label = self.function_picker_label(function);
            if !self.has_known_shape_label(&input_shape_name)
                || !seen.insert(function_label.clone())
            {
                continue;
            }
            choices.push(FieldPickerChoice::CreateProducer {
                input_shape_name,
                function_label,
            });
        }
        choices
    }

    fn field_picker_label(&self, choice: &FieldPickerChoice, required_shape_name: &str) -> String {
        match choice {
            FieldPickerChoice::ExistingSlot { slot_id } => self.slot_picker_label(*slot_id),
            FieldPickerChoice::ExistingProducerSlot { slot_id } => {
                format!(
                    "{} [produces {}]",
                    self.slot_picker_label(*slot_id),
                    required_shape_name
                )
            }
            FieldPickerChoice::CreateProducer {
                input_shape_name,
                function_label,
            } => {
                format!(
                    "+ create {} for {} via {}",
                    input_shape_name, required_shape_name, function_label
                )
            }
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
    fn function_picker_label(&self, function: &Function) -> String {
        format!(
            "{} [{} | {:?}]",
            describe_function(function),
            match function.receiver_mode {
                ReceiverMode::ByValue => "ByValue",
                ReceiverMode::ByRef => "ByRef",
                ReceiverMode::ByMut => "ByMut",
            },
            function.kind,
        )
    }

    fn slot_action_label(&self, slot_id: usize, action: SlotAction) -> String {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return match action {
                SlotAction::Rename => "rename".to_string(),
                SlotAction::Delete => "delete".to_string(),
                SlotAction::Clone => "clone".to_string(),
                SlotAction::Take => "take".to_string(),
                SlotAction::Invoke => "invoke".to_string(),
                SlotAction::InvokeArbitrary => "invoke arbitrary".to_string(),
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
            SlotAction::Invoke => match self.applicable_functions_for_slot(slot_id).len() {
                0 => "invoke".to_string(),
                1 => "invoke".to_string(),
                count => format!("invoke ({count} functions)"),
            },
            SlotAction::InvokeArbitrary => {
                match self.applicable_arbitrary_functions_for_slot(slot_id).len() {
                    0 => "invoke arbitrary".to_string(),
                    1 => "invoke arbitrary".to_string(),
                    count => format!("invoke arbitrary ({count} functions)"),
                }
            }
        }
    }
    fn breadcrumbs_line(&self) -> Line<'static> {
        let mut spans = Vec::new();
        let labels = std::iter::once("Everything".to_string())
            .chain(
                self.projection_stack
                    .iter()
                    .map(|view| self.projection_view_label(view)),
            )
            .collect::<Vec<_>>();
        let add_index = labels.len();

        for (index, label) in labels.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled(" > ", Style::default().fg(Color::DarkGray)));
            }
            let style = if self.pool_surface == PoolSurface::Breadcrumbs
                && self.active_breadcrumb_index == index
            {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if index == 0 || index + 1 < labels.len() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Yellow)
            };
            spans.push(Span::styled(label.clone(), style));
        }

        spans.push(Span::styled(" > ", Style::default().fg(Color::DarkGray)));
        let add_style = if self.pool_surface == PoolSurface::Breadcrumbs
            && self.active_breadcrumb_index == add_index
        {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM)
        };
        spans.push(Span::styled("+Add Breadcrumb", add_style));
        Line::from(spans)
    }

    fn breadcrumb_count(&self) -> usize {
        2 + self.projection_stack.len()
    }

    fn move_breadcrumb_left(&mut self) {
        self.active_breadcrumb_index = self.active_breadcrumb_index.saturating_sub(1);
    }

    fn move_breadcrumb_right(&mut self) {
        let max_index = self.breadcrumb_count().saturating_sub(1);
        self.active_breadcrumb_index = (self.active_breadcrumb_index + 1).min(max_index);
    }

    fn activate_current_breadcrumb(&mut self) {
        let add_index = self.projection_stack.len() + 1;
        if self.active_breadcrumb_index == add_index {
            self.pool_surface = PoolSurface::Slots;
            self.status_message =
                "Shape-filter breadcrumbs are the next breadcrumb feature to wire up.".to_string();
            return;
        }

        if self.active_breadcrumb_index == 0 {
            self.projection_stack.clear();
            self.active_slot_index = self
                .active_slot_index
                .min(self.total_slot_count().saturating_sub(1));
            self.active_row_index = 0;
            self.pool_surface = PoolSurface::Slots;
            self.sync_selection_viewports();
            self.status_message = "Returned to the full object pool.".to_string();
            return;
        }

        self.projection_stack.truncate(self.active_breadcrumb_index);
        self.active_slot_index = 0;
        self.active_row_index = 0;
        self.pool_surface = PoolSurface::Slots;
        self.sync_selection_viewports();
        self.status_message = format!(
            "Focused {}.",
            self.projection_stack
                .last()
                .map(|view| self.projection_view_label(view))
                .unwrap_or_else(|| "Everything".to_string())
        );
    }

    fn pop_projection_breadcrumb(&mut self) {
        let popped = self.projection_stack.pop();
        self.pool_surface = PoolSurface::Slots;
        self.active_breadcrumb_index = self
            .projection_stack
            .len()
            .min(self.breadcrumb_count().saturating_sub(1));
        self.active_slot_index = 0;
        self.active_row_index = 0;
        self.sync_selection_viewports();
        self.status_message = match popped {
            Some(view) => format!("Closed {}.", self.projection_view_label(&view)),
            None => "Returned to the full object pool.".to_string(),
        };
    }

    fn current_projection_view(&self) -> Option<&JsonProjectionView> {
        self.projection_stack.last()
    }

    fn projection_view_slot_count(&self, view: &JsonProjectionView) -> usize {
        1 + self.projection_child_count(view)
    }

    fn projection_child_count(&self, view: &JsonProjectionView) -> usize {
        self.projection_descendant_count(view.root_slot_id, &view.path)
    }

    fn projection_child_path(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
        child_index: usize,
    ) -> Option<Vec<JsonPathSegment>> {
        self.projection_descendant_path_at(root_slot_id, parent_path, child_index)
    }

    #[cfg(test)]
    fn projection_descendant_paths(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
    ) -> Vec<Vec<JsonPathSegment>> {
        let descendant_count = self.projection_descendant_count(root_slot_id, parent_path);
        let mut paths = Vec::with_capacity(descendant_count);
        for child_index in 0..descendant_count {
            let Some(path) =
                self.projection_descendant_path_at(root_slot_id, parent_path, child_index)
            else {
                break;
            };
            paths.push(path);
        }
        paths
    }

    fn projection_descendant_count(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
    ) -> usize {
        let cache_key = ProjectionCacheKey::new(root_slot_id, parent_path);
        if let Some(cached_count) = self
            .projection_cache
            .borrow()
            .descendant_counts
            .get(&cache_key)
            .copied()
        {
            return cached_count;
        }

        let Some(value) = self.json_value_at_path(root_slot_id, parent_path) else {
            return 0;
        };

        let descendant_count = match value {
            Value::Array(items) => {
                let mut descendant_count = 0;
                for index in 0..items.len() {
                    let path = append_json_path_segment(parent_path, JsonPathSegment::Index(index));
                    descendant_count += 1 + self.projection_descendant_count(root_slot_id, &path);
                }
                descendant_count
            }
            Value::Object(object) if self.projection_path_is_map(root_slot_id, parent_path) => {
                let mut descendant_count = 0;
                for key in object.keys() {
                    let path =
                        append_json_path_segment(parent_path, JsonPathSegment::Key(key.clone()));
                    descendant_count += 1 + self.projection_descendant_count(root_slot_id, &path);
                }
                descendant_count
            }
            Value::Object(object) => {
                let mut descendant_count = 0;
                for field_name in object.keys() {
                    let path = append_json_path_segment(
                        parent_path,
                        JsonPathSegment::Field(field_name.clone()),
                    );
                    descendant_count += 1 + self.projection_descendant_count(root_slot_id, &path);
                }
                descendant_count
            }
            _ => 0,
        };

        self.projection_cache
            .borrow_mut()
            .descendant_counts
            .insert(cache_key, descendant_count);
        descendant_count
    }

    fn projection_descendant_path_at(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
        child_index: usize,
    ) -> Option<Vec<JsonPathSegment>> {
        let value = self.json_value_at_path(root_slot_id, parent_path)?;

        match value {
            Value::Array(items) => {
                let mut remaining = child_index;
                for index in 0..items.len() {
                    let path = append_json_path_segment(parent_path, JsonPathSegment::Index(index));
                    if remaining == 0 {
                        return Some(path);
                    }
                    remaining -= 1;

                    let descendant_count = self.projection_descendant_count(root_slot_id, &path);
                    if remaining < descendant_count {
                        return self.projection_descendant_path_at(root_slot_id, &path, remaining);
                    }
                    remaining = remaining.saturating_sub(descendant_count);
                }
                None
            }
            Value::Object(object) if self.projection_path_is_map(root_slot_id, parent_path) => {
                let mut remaining = child_index;
                for key in object.keys() {
                    let path =
                        append_json_path_segment(parent_path, JsonPathSegment::Key(key.clone()));
                    if remaining == 0 {
                        return Some(path);
                    }
                    remaining -= 1;

                    let descendant_count = self.projection_descendant_count(root_slot_id, &path);
                    if remaining < descendant_count {
                        return self.projection_descendant_path_at(root_slot_id, &path, remaining);
                    }
                    remaining = remaining.saturating_sub(descendant_count);
                }
                None
            }
            Value::Object(object) => {
                let mut remaining = child_index;
                for field_name in object.keys() {
                    let path = append_json_path_segment(
                        parent_path,
                        JsonPathSegment::Field(field_name.clone()),
                    );
                    if remaining == 0 {
                        return Some(path);
                    }
                    remaining -= 1;

                    let descendant_count = self.projection_descendant_count(root_slot_id, &path);
                    if remaining < descendant_count {
                        return self.projection_descendant_path_at(root_slot_id, &path, remaining);
                    }
                    remaining = remaining.saturating_sub(descendant_count);
                }
                None
            }
            _ => None,
        }
    }
    fn pool_entry_at(&self, slot_index: usize) -> Option<PoolEntry> {
        if let Some(view) = self.current_projection_view() {
            if slot_index == 0 {
                return Some(PoolEntry::Projection(ProjectionSlot {
                    root_slot_id: view.root_slot_id,
                    path: view.path.clone(),
                    role: ProjectionSlotRole::ContainerRoot,
                }));
            }

            let child_index = slot_index - 1;
            let path = self.projection_child_path(view.root_slot_id, &view.path, child_index)?;
            return Some(PoolEntry::Projection(ProjectionSlot {
                root_slot_id: view.root_slot_id,
                path,
                role: ProjectionSlotRole::Child,
            }));
        }

        let mut remaining = slot_index;
        for slot in &self.object_slots {
            if remaining == 0 {
                return Some(PoolEntry::RealSlot(slot.id));
            }
            remaining -= 1;

            let child_count = self.top_level_projection_child_count(slot.id);
            if remaining < child_count {
                let path = self.projection_child_path(slot.id, &[], remaining)?;
                return Some(PoolEntry::Projection(ProjectionSlot {
                    root_slot_id: slot.id,
                    path,
                    role: ProjectionSlotRole::Child,
                }));
            }
            remaining = remaining.saturating_sub(child_count);
        }

        (remaining == 0).then_some(PoolEntry::NewSlot)
    }
    fn projection_value<'a>(&'a self, projection: &ProjectionSlot) -> Option<&'a Value> {
        self.json_value_at_path(projection.root_slot_id, &projection.path)
    }

    fn json_value_at_path<'a>(
        &'a self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
    ) -> Option<&'a Value> {
        let data_slot_id = self.data_slot_id_for(root_slot_id).unwrap_or(root_slot_id);
        let slot = self.slot_by_id(data_slot_id)?;
        let SlotRuntimeState::ResolvedValue { value, .. } = slot.runtime_state.as_ref()? else {
            return None;
        };
        let mut current = value;
        for segment in path {
            current = match (segment, current) {
                (JsonPathSegment::Field(field_name), Value::Object(object)) => {
                    object.get(field_name)?
                }
                (JsonPathSegment::Index(index), Value::Array(items)) => items.get(*index)?,
                (JsonPathSegment::Key(key), Value::Object(object)) => object.get(key)?,
                _ => return None,
            };
        }
        Some(current)
    }

    fn projection_view_label(&self, view: &JsonProjectionView) -> String {
        projection_label(view.root_slot_id, &view.path)
    }

    fn projection_shape_name_at_path(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
    ) -> Option<String> {
        let mut current_shape = self
            .slot_shape_name(root_slot_id)
            .and_then(|shape_name| self.thing_for_shape_name(shape_name))
            .map(|thing| thing.shape)?;

        for segment in path {
            current_shape = match segment {
                JsonPathSegment::Field(field_name) => shape_field_shape(current_shape, field_name)?,
                JsonPathSegment::Index(_) => sequence_element_shape(current_shape)?,
                JsonPathSegment::Key(_) => registry_map_value_shape(current_shape)?,
            };
        }

        Some(describe_shape(current_shape))
    }

    fn projection_path_is_map(&self, root_slot_id: usize, path: &[JsonPathSegment]) -> bool {
        self.slot_shape_name(root_slot_id)
            .and_then(|shape_name| self.thing_for_shape_name(shape_name))
            .map(|thing| thing.shape)
            .and_then(|mut current_shape| {
                for segment in path {
                    current_shape = match segment {
                        JsonPathSegment::Field(field_name) => {
                            shape_field_shape(current_shape, field_name)?
                        }
                        JsonPathSegment::Index(_) => sequence_element_shape(current_shape)?,
                        JsonPathSegment::Key(_) => registry_map_value_shape(current_shape)?,
                    };
                }
                Some(current_shape)
            })
            .and_then(registry_map_value_shape)
            .is_some()
    }

    fn projection_field_type_label(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
        field_name: &str,
        field_value: &Value,
    ) -> String {
        self.slot_shape_name(root_slot_id)
            .and_then(|shape_name| self.thing_for_shape_name(shape_name))
            .map(|thing| thing.shape)
            .and_then(|mut current_shape| {
                for segment in path {
                    current_shape = match segment {
                        JsonPathSegment::Field(segment_field_name) => {
                            shape_field_shape(current_shape, segment_field_name)?
                        }
                        JsonPathSegment::Index(_) => sequence_element_shape(current_shape)?,
                        JsonPathSegment::Key(_) => registry_map_value_shape(current_shape)?,
                    };
                }
                shape_field_shape(current_shape, field_name)
            })
            .map(describe_shape)
            .unwrap_or_else(|| json_type_label(field_value))
    }

    fn projection_map_entry_type_label(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
        entry_value: &Value,
    ) -> String {
        self.slot_shape_name(root_slot_id)
            .and_then(|shape_name| self.thing_for_shape_name(shape_name))
            .map(|thing| thing.shape)
            .and_then(|mut current_shape| {
                for segment in path {
                    current_shape = match segment {
                        JsonPathSegment::Field(field_name) => {
                            shape_field_shape(current_shape, field_name)?
                        }
                        JsonPathSegment::Index(_) => sequence_element_shape(current_shape)?,
                        JsonPathSegment::Key(_) => registry_map_value_shape(current_shape)?,
                    };
                }
                registry_map_value_shape(current_shape)
            })
            .map(describe_shape)
            .unwrap_or_else(|| json_type_label(entry_value))
    }
    fn projection_header_label(&self, projection: &ProjectionSlot, value: &Value) -> String {
        self.projection_shape_name_at_path(projection.root_slot_id, &projection.path)
            .unwrap_or_else(|| json_value_summary(value))
    }

    fn projection_focusable_rows(&self, projection: &ProjectionSlot) -> usize {
        match self.projection_value(projection) {
            Some(Value::Object(object)) => 1 + object.len() * 2,
            Some(_) | None => 1,
        }
    }
    fn activate_runtime_value(&mut self, slot_id: usize) {
        self.activate_json_projection(slot_id, Vec::new());
    }

    fn activate_json_projection(&mut self, root_slot_id: usize, path: Vec<JsonPathSegment>) {
        let Some(value) = self.json_value_at_path(root_slot_id, &path) else {
            self.status_message = "That projection is no longer available.".to_string();
            return;
        };

        if matches!(value, Value::Array(_) | Value::Object(_)) {
            self.projection_stack
                .push(JsonProjectionView { root_slot_id, path });
            self.active_slot_index = 0;
            self.active_row_index = 0;
            self.pool_surface = PoolSurface::Slots;
            self.sync_selection_viewports();
            self.status_message = format!(
                "Browsing {}.",
                self.projection_stack
                    .last()
                    .map(|view| self.projection_view_label(view))
                    .unwrap_or_else(|| format!("slot {}", root_slot_id))
            );
        } else {
            self.status_message = json_value_summary(value).to_string();
        }
    }

    fn activate_projection_slot_row(&mut self, projection: &ProjectionSlot, row_index: usize) {
        let Some(value) = self.projection_value(projection) else {
            self.status_message = "That projection is no longer available.".to_string();
            return;
        };

        if let Value::Object(object) = value {
            if row_index == 0 {
                if projection.role == ProjectionSlotRole::Child {
                    self.activate_json_projection(projection.root_slot_id, projection.path.clone());
                } else {
                    self.status_message =
                        self.projection_header_label(projection, value).to_string();
                }
                return;
            }

            if self.projection_path_is_map(projection.root_slot_id, &projection.path) {
                let entry_offset = row_index - 1;
                let entry_index = entry_offset / 2;
                let is_value_row = entry_offset % 2 == 1;
                let Some((entry_key, entry_value)) = object
                    .iter()
                    .nth(entry_index)
                    .map(|(entry_key, entry_value)| (entry_key.clone(), entry_value.clone()))
                else {
                    return;
                };

                if is_value_row {
                    let mut path = projection.path.clone();
                    path.push(JsonPathSegment::Key(entry_key));
                    self.activate_json_projection(projection.root_slot_id, path);
                } else {
                    self.status_message = format!(
                        "{}[{entry_key}] has type {}.",
                        projection_label(projection.root_slot_id, &projection.path),
                        self.projection_map_entry_type_label(
                            projection.root_slot_id,
                            &projection.path,
                            &entry_value,
                        )
                    );
                }
                return;
            }

            let field_offset = row_index - 1;
            let field_index = field_offset / 2;
            let is_value_row = field_offset % 2 == 1;
            let Some((field_name, field_value)) = object
                .iter()
                .nth(field_index)
                .map(|(field_name, field_value)| (field_name.clone(), field_value.clone()))
            else {
                return;
            };

            if is_value_row {
                let mut path = projection.path.clone();
                path.push(JsonPathSegment::Field(field_name));
                self.activate_json_projection(projection.root_slot_id, path);
            } else {
                self.status_message = format!(
                    "{}.{} has type {}.",
                    projection_label(projection.root_slot_id, &projection.path),
                    field_name,
                    self.projection_field_type_label(
                        projection.root_slot_id,
                        &projection.path,
                        &field_name,
                        &field_value,
                    )
                );
            }
            return;
        }

        if projection.role == ProjectionSlotRole::Child {
            self.activate_json_projection(projection.root_slot_id, projection.path.clone());
        } else {
            self.status_message = json_value_summary(value).to_string();
        }
    }
    fn draw_projection_slot(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        projection: &ProjectionSlot,
        is_active: bool,
    ) {
        let title = projection_label(projection.root_slot_id, &projection.path);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("{} [projection]", title))
            .border_style(slot_border_style(Color::Cyan, is_active));
        let inner = block.inner(area);
        if is_active {
            self.last_visible_row_count = usize::from(inner.height).max(1);
            self.clamp_row_view_offset();
            self.ensure_active_row_visible();
        }
        let scroll_offset = if is_active { self.row_view_offset } else { 0 };
        let rendered_line_count = self.projection_rendered_line_count(projection);
        let visible_line_count = usize::from(inner.height).max(1);
        let lines = self.projection_slot_lines_window(
            projection,
            is_active.then_some(self.active_row_index),
            scroll_offset..scroll_offset.saturating_add(visible_line_count),
        );
        let paragraph = Paragraph::new(lines.clone()).block(block);
        frame.render_widget(paragraph, area);

        if is_active
            && usize::from(inner.height) > 0
            && rendered_line_count > usize::from(inner.height)
        {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(rendered_line_count)
                .position(scroll_offset)
                .viewport_content_length(usize::from(inner.height));
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    #[cfg(test)]
    fn projection_slot_lines(
        &self,
        projection: &ProjectionSlot,
        active_row: Option<usize>,
    ) -> Vec<Line<'static>> {
        let rendered_line_count = self.projection_rendered_line_count(projection);
        self.projection_slot_lines_window(projection, active_row, 0..rendered_line_count.max(1))
    }

    fn projection_slot_lines_window(
        &self,
        projection: &ProjectionSlot,
        active_row: Option<usize>,
        line_range: Range<usize>,
    ) -> Vec<Line<'static>> {
        let Some(value) = self.projection_value(projection) else {
            return vec![Line::from("  unavailable")];
        };

        match value {
            Value::Object(object) => {
                let object_is_map =
                    self.projection_path_is_map(projection.root_slot_id, &projection.path);
                let total_line_count = if object.is_empty() {
                    1
                } else {
                    2 + object.len() * 2
                };
                let line_start = line_range.start.min(total_line_count);
                let line_end = line_range.end.min(total_line_count);
                if line_start >= line_end {
                    return Vec::new();
                }

                let mut lines = Vec::with_capacity(line_end.saturating_sub(line_start));
                if line_start == 0 {
                    lines.push(selectable_plain_line(
                        self.projection_header_label(projection, value),
                        active_row == Some(0),
                    ));
                }

                if !object.is_empty() && line_start <= 1 && line_end > 1 {
                    lines.push(separator_line(if object_is_map {
                        "entries"
                    } else {
                        "fields"
                    }));
                }

                if object.is_empty() || line_end <= 2 {
                    return lines;
                }

                let entry_line_start = line_start.max(2);
                let first_entry_index = (entry_line_start - 2) / 2;
                let last_entry_index = (line_end - 2).div_ceil(2).min(object.len());

                for (index, (field_name, field_value)) in object
                    .iter()
                    .enumerate()
                    .skip(first_entry_index)
                    .take(last_entry_index.saturating_sub(first_entry_index))
                {
                    let accent = field_group_color(index);
                    let field_type_label = if object_is_map {
                        self.projection_map_entry_type_label(
                            projection.root_slot_id,
                            &projection.path,
                            field_value,
                        )
                    } else {
                        self.projection_field_type_label(
                            projection.root_slot_id,
                            &projection.path,
                            field_name,
                            field_value,
                        )
                    };

                    let type_line_index = 1 + index * 2;
                    if line_start <= type_line_index && line_end > type_line_index {
                        lines.push(selectable_spans_line(
                            vec![
                                Span::styled(
                                    "type ",
                                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                                ),
                                Span::styled(
                                    field_type_label,
                                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                                ),
                            ],
                            active_row == Some(type_line_index),
                        ));
                    }

                    let value_line_index = 2 + index * 2;
                    if line_start <= value_line_index && line_end > value_line_index {
                        lines.push(selectable_spans_line(
                            vec![
                                Span::styled(
                                    format!("{}: ", field_name),
                                    Style::default().fg(accent),
                                ),
                                Span::styled(
                                    json_value_summary(field_value),
                                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                                ),
                            ],
                            active_row == Some(value_line_index),
                        ));
                    }
                }
                lines
            }
            _ => {
                let total_line_count = 3;
                let line_start = line_range.start.min(total_line_count);
                let line_end = line_range.end.min(total_line_count);
                if line_start >= line_end {
                    return Vec::new();
                }

                let mut lines = Vec::with_capacity(line_end.saturating_sub(line_start));
                if line_start == 0 {
                    lines.push(selectable_plain_line(
                        self.projection_header_label(projection, value),
                        active_row == Some(0),
                    ));
                }
                if line_start <= 1 && line_end > 1 {
                    lines.push(separator_line("value"));
                }
                if line_start <= 2 && line_end > 2 {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            json_value_detail(value),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                }
                lines
            }
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
            (
                picker.selected_choice()?,
                picker.required_shape_name.clone(),
            )
        };
        match choice {
            FieldPickerChoice::ExistingSlot { slot_id }
            | FieldPickerChoice::ExistingProducerSlot { slot_id } => {
                Some(self.slot_preview_lines(slot_id))
            }
            FieldPickerChoice::CreateProducer {
                input_shape_name, ..
            } => self
                .shape_choices
                .iter()
                .find(|shape| shape.label == input_shape_name)
                .map(shape_preview_lines),
            FieldPickerChoice::CreateNew => self
                .shape_choices
                .iter()
                .find(|shape| shape.label == required_shape_name)
                .map(shape_preview_lines),
        }
    }

    fn function_picker_preview_lines(&mut self) -> Option<Vec<Line<'static>>> {
        let function = {
            let picker = self.function_picker.as_ref()?;
            picker.selected_function()?
        };
        let mut lines = vec![Line::from(self.function_picker_label(function))];
        lines.push(Line::from(format!(
            "input: {}",
            describe_shape(function.input_shape)
        )));
        lines.push(Line::from(format!(
            "output: {}",
            describe_shape(function.output_shape)
        )));
        lines.push(Line::from(format!("origin: {}", function.origin)));
        lines.push(Line::from(format!(
            "receiver: {:?}",
            function.receiver_mode
        )));
        if !function.effects.is_empty() {
            lines.push(Line::from(format!("effects: {:?}", function.effects)));
        }
        if let Some(thing) = self
            .shape_choices
            .iter()
            .find(|shape| shape.thing.shape.is_shape(function.input_shape))
            .map(|shape| shape.thing)
        {
            let dependencies = thing.input_dependencies();
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
        Some(lines)
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
                Line::from(slot_label.to_string()),
                Line::from(format!("will move into {field_label}.")),
                Line::from(""),
                Line::from("The old parent link will be cleared, leaving a hole there if needed."),
                Line::from("This keeps the same slot card, but repoints it at the new field."),
            ],
            LinkAction::Clone => vec![
                Line::from(slot_label.to_string()),
                Line::from(format!(
                    "will stay put and {field_label} gets a new view slot."
                )),
                Line::from(""),
                Line::from("Use this when the current top-level card should remain where it is."),
                Line::from("The new field gets its own slot card for navigation."),
            ],
        }
    }

    fn slot_lines(
        &mut self,
        slot_id: usize,
        is_active: bool,
        active_row: usize,
    ) -> Vec<Line<'static>> {
        if is_active
            && let Some(slot_search) = self
                .slot_search
                .as_ref()
                .filter(|slot_search| slot_search.slot_id == slot_id)
        {
            return render_slot_search_matches(
                &slot_search.filtered_matches,
                slot_search.selected_match_index,
            );
        }

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

    fn slot_search_matches(&mut self, slot_id: usize, query: &str) -> Vec<SlotSearchMatch> {
        let focusable_rows = self
            .slot_display_rows(slot_id)
            .into_iter()
            .filter_map(|row| match row {
                SlotDisplayRow::Focusable { target, spans } => Some((target, spans)),
                SlotDisplayRow::Static(_) => None,
            })
            .collect::<Vec<_>>();
        let search_labels = focusable_rows
            .iter()
            .map(|(target, spans)| self.slot_search_label(slot_id, *target, spans))
            .collect::<Vec<_>>();

        ranked_slot_search_indices(query, &search_labels)
            .into_iter()
            .filter_map(|index| {
                let (target, spans) = focusable_rows.get(index)?.clone();
                let matched_indices =
                    match_indices(query, &spans_plain_text(&spans)).unwrap_or_default();
                Some(SlotSearchMatch {
                    target,
                    spans,
                    matched_indices,
                })
            })
            .collect()
    }

    fn slot_search_label(
        &self,
        slot_id: usize,
        target: SlotFocusTarget,
        spans: &[Span<'static>],
    ) -> String {
        match target {
            SlotFocusTarget::Shape => self
                .slot_shape_name(slot_id)
                .map(|shape_name| format!("shape {shape_name}"))
                .unwrap_or_else(|| "shape unset".to_string()),
            SlotFocusTarget::ViewPointer => self
                .slot_by_id(slot_id)
                .and_then(|slot| match &slot.kind {
                    SlotKind::View(info) => Some(format!(
                        "pointer {} {}",
                        info.owner_slot_id, info.field_name
                    )),
                    SlotKind::Owned => None,
                })
                .unwrap_or_else(|| spans_plain_text(spans)),
            SlotFocusTarget::Variant => self
                .slot_body(slot_id)
                .and_then(|body| match body {
                    SlotBody::Enum {
                        variants,
                        selected_variant,
                        ..
                    } => selected_variant
                        .and_then(|index| variants.get(index))
                        .map(|variant| format!("variant {}", variant.info.variant_name))
                        .or_else(|| Some("variant unset".to_string())),
                    _ => None,
                })
                .unwrap_or_else(|| spans_plain_text(spans)),
            SlotFocusTarget::FieldType(field_index) => self
                .slot_field(slot_id, field_index)
                .map(|field| {
                    format!(
                        "type {} {}",
                        field.info.field_name, field.info.field_shape_name
                    )
                })
                .unwrap_or_else(|| spans_plain_text(spans)),
            SlotFocusTarget::FieldValue(field_index) => self
                .slot_field(slot_id, field_index)
                .map(|field| field.info.field_name.to_string())
                .unwrap_or_else(|| spans_plain_text(spans)),
            SlotFocusTarget::Action(action) => self.slot_action_label(slot_id, action),
            _ => spans_plain_text(spans),
        }
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
    SlotSearch,
    ShapePicker,
    VariantPicker,
    FieldPicker,
    FunctionPicker,
    LinkActionPicker,
    RenameSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PoolSurface {
    Slots,
    Breadcrumbs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotAxis {
    Horizontal,
    Vertical,
}

impl SlotAxis {
    fn label(self) -> &'static str {
        match self {
            SlotAxis::Horizontal => "horizontal",
            SlotAxis::Vertical => "vertical",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct JsonProjectionView {
    root_slot_id: usize,
    path: Vec<JsonPathSegment>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum JsonPathSegment {
    Field(String),
    Index(usize),
    Key(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ProjectionCacheKey {
    root_slot_id: usize,
    path: Vec<JsonPathSegment>,
}

impl ProjectionCacheKey {
    fn new(root_slot_id: usize, path: &[JsonPathSegment]) -> Self {
        Self {
            root_slot_id,
            path: path.to_vec(),
        }
    }
}

#[derive(Default)]
struct ProjectionCache {
    descendant_counts: HashMap<ProjectionCacheKey, usize>,
}

impl ProjectionCache {
    fn clear(&mut self) {
        self.descendant_counts.clear();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProjectionSlot {
    root_slot_id: usize,
    path: Vec<JsonPathSegment>,
    role: ProjectionSlotRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProjectionSlotRole {
    ContainerRoot,
    Child,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PoolEntry {
    RealSlot(usize),
    NewSlot,
    Projection(ProjectionSlot),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotFocusTarget {
    Shape,
    ViewPointer,
    Variant,
    FieldType(usize),
    FieldValue(usize),
    Inlink(usize),
    CreatedFor,
    ProducedBy,
    RuntimeValue,
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
    InvokeArbitrary,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SlotCompletion {
    Unset,
    Partial,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SlotCreatedFor {
    owner_slot_id: usize,
    field_index: usize,
    field_name: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SlotInlink {
    owner_slot_id: usize,
    field_index: usize,
    field_name: &'static str,
}

#[derive(Clone, Debug)]
struct SlotSearchMatch {
    target: SlotFocusTarget,
    spans: Vec<Span<'static>>,
    matched_indices: Vec<u32>,
}

struct SlotSearchState {
    slot_id: usize,
    query: TextArea<'static>,
    filtered_matches: Vec<SlotSearchMatch>,
    selected_match_index: usize,
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
        self.choices.get(index).cloned()
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
enum FunctionPickerTarget {
    InvokeSlot(usize),
    InvokeArbitrarySlot(usize),
}

struct FunctionPickerState {
    target: FunctionPickerTarget,
    labels: Vec<String>,
    functions: Vec<&'static Function>,
    search: PickerSearchState,
}

impl FunctionPickerState {
    fn new(
        target: FunctionPickerTarget,
        functions: Vec<&'static Function>,
        labels: Vec<String>,
    ) -> Self {
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        Self {
            target,
            labels,
            functions,
            search,
        }
    }

    fn selected_function(&self) -> Option<&'static Function> {
        let index = self.search.selected_filtered_index()?;
        self.functions.get(index).copied()
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum FieldPickerChoice {
    ExistingSlot {
        slot_id: usize,
    },
    ExistingProducerSlot {
        slot_id: usize,
    },
    CreateProducer {
        input_shape_name: String,
        function_label: String,
    },
    CreateNew,
}

fn field_picker_choice_is_arbitrary_producer(choice: &FieldPickerChoice) -> bool {
    matches!(
        choice,
        FieldPickerChoice::CreateProducer {
            input_shape_name,
            ..
        } if input_shape_name == "ArbitraryBytes"
    )
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
    ResolvedValue { json: String, value: Value },
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
    created_for: Option<SlotCreatedFor>,
    produced_by_slot_id: Option<usize>,
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
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: snapshot.value_json.and_then(|json| {
                serde_json::from_str::<Value>(&json)
                    .ok()
                    .map(|value| SlotRuntimeState::ResolvedValue { json, value })
            }),
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
            created_for: None,
            produced_by_slot_id: None,
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
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(SlotRuntimeState::Pending(pending)),
            display_cache: None,
        }
    }

    fn new_resolved_result(id: usize, shape_name: String, json: String, value: Value) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::Owned,
            shape_name: Some(shape_name),
            body: SlotBody::Unset,
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(SlotRuntimeState::ResolvedValue { json, value }),
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
            created_for: None,
            produced_by_slot_id: None,
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

fn variant_row(variants: &[ObjectVariantState], selected_variant: Option<usize>) -> SlotDisplayRow {
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
            vec![SlotDisplayRow::Static(Line::from(
                "  pending invocation...",
            ))]
        }
        SlotRuntimeState::Failed { message } => vec![SlotDisplayRow::Static(Line::from(vec![
            Span::raw("  "),
            Span::styled("failed", unset_style()),
            Span::raw(format!(": {message}")),
        ]))],
        SlotRuntimeState::ResolvedValue { value, .. } => vec![focusable_spans_row(
            SlotFocusTarget::RuntimeValue,
            vec![
                Span::styled(
                    "resolved ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(json_value_summary(value)),
            ],
        )],
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

fn render_slot_search_matches(
    matches: &[SlotSearchMatch],
    selected_match_index: usize,
) -> Vec<Line<'static>> {
    if matches.is_empty() {
        return vec![Line::from(Span::styled(
            "  no matches",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        ))];
    }

    matches
        .iter()
        .enumerate()
        .map(|(index, matched)| {
            selectable_spans_line(
                highlight_matched_spans(&matched.spans, &matched.matched_indices),
                index == selected_match_index,
            )
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

    let functions = functions_from(choice.thing.shape);
    if !functions.is_empty() {
        lines.push(separator_line("functions"));
        for function in functions {
            lines.push(Line::from(format!("  {}", describe_function(function))));
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
    ranked_match_indices(query, labels)
        .into_iter()
        .map(|(index, _)| index)
        .collect()
}

fn ranked_slot_search_indices(query: &str, labels: &[String]) -> Vec<usize> {
    if query.trim().is_empty() {
        return (0..labels.len()).collect();
    }

    let query_lower = query.to_lowercase();
    let mut ranked = Vec::new();
    let mut taken = BTreeSet::new();

    for (index, label) in labels.iter().enumerate() {
        if label.to_lowercase().starts_with(&query_lower) {
            taken.insert(index);
            ranked.push(index);
        }
    }

    for (index, label) in labels.iter().enumerate() {
        if taken.contains(&index) {
            continue;
        }
        if label.to_lowercase().contains(&query_lower) {
            taken.insert(index);
            ranked.push(index);
        }
    }

    if ranked.is_empty() {
        ranked_match_indices(query, labels)
            .into_iter()
            .map(|(index, _)| index)
            .collect()
    } else {
        ranked
    }
}

fn ranked_match_indices(query: &str, labels: &[String]) -> Vec<(usize, Vec<u32>)> {
    if query.trim().is_empty() {
        return labels
            .iter()
            .enumerate()
            .map(|(index, _)| (index, Vec::new()))
            .collect();
    }

    let query_lower = query.to_lowercase();
    let mut taken = BTreeSet::new();
    let mut ranked = Vec::new();

    for (index, label) in labels.iter().enumerate() {
        if label.to_lowercase().starts_with(&query_lower) {
            taken.insert(index);
            ranked.push((index, match_indices(query, label).unwrap_or_default()));
        }
    }

    for (index, label) in labels.iter().enumerate() {
        if taken.contains(&index) {
            continue;
        }
        if label.to_lowercase().contains(&query_lower) {
            taken.insert(index);
            ranked.push((index, match_indices(query, label).unwrap_or_default()));
        }
    }

    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(nucleo::Config::DEFAULT);
    for (matched_label, _score) in pattern.match_list(labels, &mut matcher) {
        let Some(index) = labels.iter().enumerate().find_map(|(index, label)| {
            (label == matched_label && taken.insert(index)).then_some(index)
        }) else {
            continue;
        };
        ranked.push((
            index,
            match_indices(query, matched_label).unwrap_or_default(),
        ));
    }

    ranked
}

fn match_indices(query: &str, label: &str) -> Option<Vec<u32>> {
    if query.trim().is_empty() {
        return Some(Vec::new());
    }

    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
    let mut matcher = Matcher::new(nucleo::Config::DEFAULT);
    let mut haystack_buf = Vec::new();
    let haystack = Utf32Str::new(label, &mut haystack_buf);
    let mut indices = Vec::new();
    pattern.indices(haystack, &mut matcher, &mut indices)?;
    indices.sort_unstable();
    indices.dedup();
    Some(indices)
}

fn spans_plain_text(spans: &[Span<'static>]) -> String {
    spans.iter().map(|span| span.content.as_ref()).collect()
}

fn highlight_matched_spans(spans: &[Span<'static>], matched_indices: &[u32]) -> Vec<Span<'static>> {
    if matched_indices.is_empty() {
        return spans.to_vec();
    }

    let mut highlighted = Vec::new();
    let mut match_cursor = 0usize;
    let mut char_index = 0u32;

    for span in spans {
        let mut run = String::new();
        let mut run_highlighted = None;
        for character in span.content.chars() {
            let is_highlighted = matched_indices
                .get(match_cursor)
                .is_some_and(|matched_index| *matched_index == char_index);
            if is_highlighted {
                match_cursor += 1;
            }

            match run_highlighted {
                Some(previous) if previous != is_highlighted => {
                    highlighted.push(styled_span(run, span.style, previous));
                    run = String::new();
                    run_highlighted = Some(is_highlighted);
                }
                None => run_highlighted = Some(is_highlighted),
                _ => {}
            }

            run.push(character);
            char_index += 1;
        }

        if let Some(is_highlighted) = run_highlighted {
            highlighted.push(styled_span(run, span.style, is_highlighted));
        }
    }

    highlighted
}

fn styled_span(content: String, base_style: Style, is_highlighted: bool) -> Span<'static> {
    if is_highlighted {
        Span::styled(
            content,
            base_style.patch(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        )
    } else {
        Span::styled(content, base_style)
    }
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

fn vertical_marker_paragraph(marker: &str, height: u16, style: Style) -> Paragraph<'static> {
    let lines = (0..height.max(1))
        .map(|_| Line::from(Span::styled(marker.to_string(), style)))
        .collect::<Vec<_>>();
    Paragraph::new(lines).alignment(Alignment::Center)
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

fn resize_dimension(current: u16, minimum: u16, step: u16, direction: isize) -> u16 {
    if direction < 0 {
        current.saturating_sub(step).max(minimum)
    } else {
        current.saturating_add(step).max(minimum)
    }
}
fn append_json_path_segment(
    parent_path: &[JsonPathSegment],
    segment: JsonPathSegment,
) -> Vec<JsonPathSegment> {
    let mut path = parent_path.to_vec();
    path.push(segment);
    path
}

fn projection_label(root_slot_id: usize, path: &[JsonPathSegment]) -> String {
    let mut label = format!("slot {}", root_slot_id);
    for segment in path {
        match segment {
            JsonPathSegment::Field(field_name) => {
                label.push('.');
                label.push_str(field_name);
            }
            JsonPathSegment::Index(index) => {
                label.push('[');
                label.push_str(&index.to_string());
                label.push(']');
            }
            JsonPathSegment::Key(key) => {
                label.push('[');
                label.push_str(key);
                label.push(']');
            }
        }
    }
    label
}

fn json_type_label(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "bool".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(items) => format!("array[{}]", items.len()),
        Value::Object(object) => format!("object[{}]", object.len()),
    }
}

fn json_value_detail(value: &Value) -> String {
    match value {
        Value::String(text) => format!("\"{}\"", text),
        _ => json_value_summary(value),
    }
}
fn json_value_summary(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => {
            let truncated = if text.chars().count() > 40 {
                format!("{}...", text.chars().take(37).collect::<String>())
            } else {
                text.clone()
            };
            format!("\"{}\"", truncated)
        }
        Value::Array(items) => format!("{} entries", items.len()),
        Value::Object(object) => format!("object with {} fields", object.len()),
    }
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

fn horizontal_scrollbar_overlay_area(area: Rect) -> Rect {
    if area.width <= 2 || area.height == 0 {
        return Rect::default();
    }

    Rect {
        x: area.x + 1,
        y: area.y + area.height - 1,
        width: area.width.saturating_sub(2),
        height: 1,
    }
}

fn vertical_scrollbar_overlay_area(area: Rect) -> Rect {
    if area.width == 0 || area.height <= 2 {
        return Rect::default();
    }

    Rect {
        x: area.x + area.width - 1,
        y: area.y + 1,
        width: 1,
        height: area.height.saturating_sub(2),
    }
}
#[cfg(test)]
mod tests {
    use super::FieldPickerChoice;
    use super::ObjectBrowserApp;
    use super::ObjectSlot;
    use super::ShapeVariantInfo;
    use super::SlotBody;
    use super::SlotFocusTarget;
    use super::SlotKind;
    use super::UiMode;
    use arbitrary::Arbitrary;
    use cloud_terrastodon_registry::describe_shape;
    use cloud_terrastodon_registry::known_shapes;
    use facet::Facet;
    use ratatui::crossterm::event::KeyCode;
    use ratatui::crossterm::event::KeyEvent;
    use ratatui::crossterm::event::KeyModifiers;
    use std::future::Future;
    use std::future::IntoFuture;

    #[derive(Debug, Clone, Arbitrary, Facet)]
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
    cloud_terrastodon_registry::register_thing!(DummyInvokeRequest);
    cloud_terrastodon_registry::register_into_future!(DummyInvokeRequest => DummyInvokeOutput);
    cloud_terrastodon_registry::register_fn_mut!(
        cloud_terrastodon_registry::ArbitraryBytes => DummyInvokeOutput,
        kind = cloud_terrastodon_registry::FunctionKind::Constructor,
        label = "arbitrary",
        origin = "Arbitrary",
        invoke = cloud_terrastodon_registry::arbitrary_from_bytes::<DummyInvokeOutput>
    );
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
    fn field_picker_offers_request_producers_for_matching_output_types() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("EntraUserListRequest"))
            .expect("EntraUserListRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.active_row_index = 2;
        app.activate_current_row();

        let picker = app.field_picker.as_ref().expect("field picker should open");
        assert!(picker.choices.iter().any(|choice| matches!(
            choice,
            FieldPickerChoice::CreateProducer { input_shape_name, .. }
                if input_shape_name == "AzureTenantIdResolveRequest"
        )));
    }

    #[test]
    fn field_picker_lists_arbitrary_producers_after_create_new() {
        let mut app = ObjectBrowserApp::default();
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
        let create_new_index = picker
            .choices
            .iter()
            .position(|choice| choice == &FieldPickerChoice::CreateNew)
            .expect("create-new choice should be present");
        let arbitrary_index = picker
            .choices
            .iter()
            .position(super::field_picker_choice_is_arbitrary_producer)
            .expect("arbitrary producer choice should be present");

        assert!(
            create_new_index < arbitrary_index,
            "choices were {:?}",
            picker.labels
        );
        assert!(super::field_picker_choice_is_arbitrary_producer(
            picker.choices.last().expect("last choice should exist")
        ));
        assert_eq!(picker.selected_choice(), Some(FieldPickerChoice::CreateNew));
    }

    #[test]
    fn selecting_a_request_producer_creates_a_request_slot() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("EntraUserListRequest"))
            .expect("EntraUserListRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.active_row_index = 2;
        app.activate_current_row();

        let producer_index = app
            .field_picker
            .as_ref()
            .and_then(|picker| {
                picker.choices.iter().position(|choice| {
                    matches!(
                        choice,
                        FieldPickerChoice::CreateProducer { input_shape_name, .. }
                            if input_shape_name == "AzureTenantIdResolveRequest"
                    )
                })
            })
            .expect("field picker should offer AzureTenantIdResolveRequest");
        app.field_picker
            .as_mut()
            .expect("field picker should still be open")
            .search
            .list_state
            .select(Some(producer_index));

        app.apply_field_picker_selection();

        let created_slot = app
            .slot_by_id(2)
            .expect("selecting the producer should create a new source slot");
        assert!(matches!(created_slot.kind, SlotKind::Owned));
        assert_eq!(
            created_slot.shape_name.as_deref(),
            Some("AzureTenantIdResolveRequest")
        );
        assert!(matches!(
            created_slot.created_for,
            Some(super::SlotCreatedFor {
                owner_slot_id: 1,
                field_index: 0,
                field_name: "tenant_id",
            })
        ));
        assert!(matches!(
            app.slot_field(1, 0).map(|field| field.value_state),
            Some(super::FieldValueState::Unset)
        ));
    }

    #[test]
    fn field_picker_lists_existing_request_slots_that_can_produce_the_field_type() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_resolve_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(tenant_resolve_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_resolve_index));
        app.apply_shape_selection();

        app.active_slot_index = 1;
        app.activate_current_row();
        let user_list_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("EntraUserListRequest"))
            .expect("EntraUserListRequest should be registered");
        app.shape_picker.open(Some(user_list_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(user_list_index));
        app.apply_shape_selection();

        app.active_row_index = 2;
        app.activate_current_row();

        let picker = app.field_picker.as_ref().expect("field picker should open");
        assert!(
            picker
                .choices
                .contains(&FieldPickerChoice::ExistingProducerSlot { slot_id: 1 })
        );
    }

    #[test]
    fn field_picker_prefers_exact_value_slots_over_request_producers() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_id_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantId"))
            .expect("AzureTenantId should be registered");
        app.shape_picker.open(Some(tenant_id_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_id_index));
        app.apply_shape_selection();

        app.active_slot_index = 1;
        app.activate_current_row();
        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("EntraUserListRequest"))
            .expect("EntraUserListRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.active_row_index = 2;
        app.activate_current_row();

        let picker = app.field_picker.as_ref().expect("field picker should open");
        let selected_choice = picker
            .selected_choice()
            .expect("a choice should be selected");
        assert_eq!(
            selected_choice,
            FieldPickerChoice::ExistingSlot { slot_id: 1 }
        );
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

        let dummy_function = app
            .applicable_functions_for_slot(1)
            .into_iter()
            .find(|function| describe_shape(function.output_shape) == "DummyInvokeOutput")
            .expect("DummyInvokeOutput constructor should be available");
        app.invoke_registered_function(1, dummy_function);
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
                super::SlotRuntimeState::ResolvedValue { json, .. } => Some(json.clone()),
                _ => None,
            })
            .expect("result slot should resolve");
        assert!(resolved_json.contains("\"message\":\"done\""));
    }

    #[test]
    fn invoke_arbitrary_action_creates_a_resolved_result_slot() {
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

        let shape_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label == "ArbitraryBytes")
            .expect("ArbitraryBytes should be registered");
        app.shape_picker.open(Some(shape_index));
        app.shape_picker.search.list_state.select(Some(shape_index));
        app.apply_shape_selection();
        if let Some(slot) = app.slot_by_id_mut(1) {
            slot.runtime_state = Some(super::SlotRuntimeState::ResolvedValue {
                json: "[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]"
                    .to_string(),
                value: serde_json::from_str(
                    "[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]",
                )
                .unwrap(),
            });
        }

        let dummy_function = app
            .applicable_functions_for_slot(1)
            .into_iter()
            .find(|function| describe_shape(function.output_shape) == "DummyInvokeOutput")
            .expect("DummyInvokeOutput constructor should be available");
        app.invoke_registered_function(1, dummy_function);
        let result_slot_id = app
            .slot_by_id(1)
            .and_then(|slot| slot.result_slot_ids.first().copied())
            .expect("invocation should create a result slot");
        let resolved_value = app
            .slot_by_id(result_slot_id)
            .and_then(|slot| slot.runtime_state.as_ref())
            .and_then(|runtime| match runtime {
                super::SlotRuntimeState::ResolvedValue { value, .. } => Some(value.clone()),
                _ => None,
            })
            .expect("result slot should resolve immediately");
        assert!(resolved_value.get("message").is_some());
    }

    #[test]
    fn request_slots_show_invoke_arbitrary_action_when_fake_results_are_registered() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let tenant_request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(tenant_request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(tenant_request_index));
        app.apply_shape_selection();

        assert!(
            app.slot_focus_targets(1)
                .contains(&SlotFocusTarget::Action(super::SlotAction::InvokeArbitrary))
        );
        assert_eq!(
            app.slot_action_label(1, super::SlotAction::InvokeArbitrary),
            "invoke arbitrary"
        );

        app.activate_current_row();
        let user_request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("EntraUserListRequest"))
            .expect("EntraUserListRequest should be registered");
        app.shape_picker.open(Some(user_request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(user_request_index));
        app.apply_shape_selection();

        assert!(
            app.slot_focus_targets(1)
                .contains(&SlotFocusTarget::Action(super::SlotAction::InvokeArbitrary))
        );
        assert_eq!(
            app.slot_action_label(1, super::SlotAction::InvokeArbitrary),
            "invoke arbitrary"
        );
    }

    #[test]
    fn invoke_arbitrary_action_creates_a_fake_result_for_registered_requests() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let request_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("EntraUserListRequest"))
            .expect("EntraUserListRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.activate_slot_action(1, super::SlotAction::InvokeArbitrary);

        let result_slot_id = app
            .slot_by_id(1)
            .and_then(|slot| slot.result_slot_ids.first().copied())
            .unwrap_or_else(|| {
                panic!(
                    "fake invocation should create a result slot: {}",
                    app.status_message
                )
            });
        let resolved_value = app
            .slot_by_id(result_slot_id)
            .and_then(|slot| slot.runtime_state.as_ref())
            .and_then(|runtime| match runtime {
                super::SlotRuntimeState::ResolvedValue { value, .. } => Some(value.clone()),
                _ => None,
            })
            .expect("fake result slot should resolve immediately");
        assert!(resolved_value.is_array());
    }

    #[test]
    fn typing_starts_slot_search_and_jumps_to_matching_field() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();

        let shape_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label == "EntraUser")
            .expect("EntraUser should be registered");
        app.shape_picker.open(Some(shape_index));
        app.shape_picker.search.list_state.select(Some(shape_index));
        app.apply_shape_selection();

        let mail_index = match app.slot_by_id(1).map(|slot| &slot.body) {
            Some(SlotBody::Struct { fields }) => fields
                .iter()
                .position(|field| field.info.field_name == "mail")
                .expect("EntraUser should expose a mail field"),
            _ => panic!("EntraUser should build as a struct"),
        };

        app.handle_pool_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));
        app.handle_pool_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        assert_eq!(app.mode, UiMode::SlotSearch);
        assert_eq!(
            app.slot_search_current_target(),
            Some(SlotFocusTarget::FieldValue(mail_index)),
            "query={:?}, labels={:?}",
            app.slot_search
                .as_ref()
                .map(|search| search.query.lines().join("\\n")),
            app.slot_search.as_ref().map(|search| search
                .filtered_matches
                .iter()
                .map(|matched| matched.target)
                .collect::<Vec<_>>())
        );
        assert_eq!(
            app.active_row_index,
            app.focus_row_for_slot_target(1, SlotFocusTarget::FieldValue(mail_index))
                .expect("mail row should be focusable")
        );
    }

    #[test]
    fn typing_in_slot_search_resets_selection_to_the_top_match() {
        let mut app = ObjectBrowserApp::default();
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

        app.handle_pool_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        app.move_slot_search_selection(1);
        app.handle_slot_search_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        assert_eq!(app.mode, UiMode::SlotSearch);
        assert_eq!(
            app.slot_search
                .as_ref()
                .map(|search| search.selected_match_index),
            Some(0)
        );
        assert_eq!(
            app.slot_search_current_target(),
            Some(SlotFocusTarget::Action(super::SlotAction::Invoke)),
            "query={:?}, labels={:?}",
            app.slot_search
                .as_ref()
                .map(|search| search.query.lines().join("\\n")),
            app.slot_search.as_ref().map(|search| search
                .filtered_matches
                .iter()
                .map(|matched| matched.target)
                .collect::<Vec<_>>())
        );
    }

    #[test]
    fn slot_search_can_jump_to_invoke_action() {
        let mut app = ObjectBrowserApp::default();
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

        app.handle_pool_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        app.handle_pool_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        assert_eq!(app.mode, UiMode::SlotSearch);
        assert_eq!(
            app.slot_search_current_target(),
            Some(SlotFocusTarget::Action(super::SlotAction::Invoke)),
            "query={:?}, labels={:?}",
            app.slot_search
                .as_ref()
                .map(|search| search.query.lines().join("\\n")),
            app.slot_search.as_ref().map(|search| search
                .filtered_matches
                .iter()
                .map(|matched| matched.target)
                .collect::<Vec<_>>())
        );

        let lines = app.slot_lines(1, true, app.active_row_index);
        assert!(lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.style.fg == Some(ratatui::prelude::Color::Yellow))
        }));
    }

    #[test]
    fn creating_a_slot_after_projection_children_selects_the_new_owned_slot() {
        let mut app = ObjectBrowserApp::default();
        let value = serde_json::json!([
            { "displayName": "Ada" },
            { "displayName": "Grace" }
        ]);
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some("Vec<EntraUser>".to_string()),
            body: super::SlotBody::Unset,
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(super::SlotRuntimeState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            }),
            display_cache: None,
        });

        app.move_slot_end();
        app.activate_current_row();

        assert!(
            matches!(
                app.current_pool_entry(),
                Some(super::PoolEntry::RealSlot(2))
            ),
            "active_slot_index={}, total_slot_count={}, slots={}, entry={:?}",
            app.active_slot_index,
            app.total_slot_count(),
            app.object_slots.len(),
            app.current_pool_entry()
        );
    }

    #[test]
    fn resolved_hashmap_fields_expand_into_entry_projection_cards() {
        let mut app = ObjectBrowserApp::default();
        let shape_name = app
            .shape_choices
            .iter()
            .find(|shape| shape.label == "RoleDefinitionsAndAssignments")
            .map(|shape| shape.label.clone())
            .expect("RoleDefinitionsAndAssignments should be registered");
        let value = serde_json::json!({
            "role_assignments": {
                "/subscriptions/sub-1/providers/Microsoft.Authorization/roleAssignments/assignment-a": {
                    "id": "/subscriptions/sub-1/providers/Microsoft.Authorization/roleAssignments/assignment-a",
                    "principal_id": "principal-a",
                    "role_definition_id": "/subscriptions/sub-1/providers/Microsoft.Authorization/roleDefinitions/definition-a",
                    "scope": "/subscriptions/sub-1"
                },
                "/subscriptions/sub-1/providers/Microsoft.Authorization/roleAssignments/assignment-b": {
                    "id": "/subscriptions/sub-1/providers/Microsoft.Authorization/roleAssignments/assignment-b",
                    "principal_id": "principal-b",
                    "role_definition_id": "/subscriptions/sub-1/providers/Microsoft.Authorization/roleDefinitions/definition-a",
                    "scope": "/subscriptions/sub-1/resourceGroups/rg-a"
                }
            },
            "role_definitions": {
                "/subscriptions/sub-1/providers/Microsoft.Authorization/roleDefinitions/definition-a": {
                    "id": "/subscriptions/sub-1/providers/Microsoft.Authorization/roleDefinitions/definition-a",
                    "name": "Reader"
                }
            }
        });
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some(shape_name),
            body: super::SlotBody::Unset,
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(super::SlotRuntimeState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            }),
            display_cache: None,
        });

        app.activate_runtime_value(1);
        assert_eq!(app.projection_stack.len(), 1);
        assert_eq!(app.total_slot_count(), 16);

        let root_projection = super::ProjectionSlot {
            root_slot_id: 1,
            path: Vec::new(),
            role: super::ProjectionSlotRole::ContainerRoot,
        };
        app.activate_projection_slot_row(&root_projection, 2);

        let map_view = app
            .projection_stack
            .last()
            .expect("role_assignments projection should open");
        assert_eq!(
            app.projection_shape_name_at_path(map_view.root_slot_id, &map_view.path)
                .as_deref(),
            Some("HashMap<RoleAssignmentId, RoleAssignment>")
        );
        assert_eq!(app.total_slot_count(), 11);
        let descendant_paths =
            app.projection_descendant_paths(map_view.root_slot_id, &map_view.path);
        assert!(descendant_paths.iter().any(
            |path| matches!(path.last(), Some(super::JsonPathSegment::Key(key)) if key.contains("assignment-a"))
        ));
        assert!(descendant_paths.iter().any(
            |path| matches!(path.last(), Some(super::JsonPathSegment::Key(key)) if key.contains("assignment-b"))
        ));

        let lines = app.projection_slot_lines(
            &super::ProjectionSlot {
                root_slot_id: map_view.root_slot_id,
                path: map_view.path.clone(),
                role: super::ProjectionSlotRole::ContainerRoot,
            },
            None,
        );
        let rendered = lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("--- entries ---"), "{rendered}");
        assert!(rendered.contains("RoleAssignment"), "{rendered}");
    }

    #[test]
    fn role_permission_action_projection_preserves_element_shape() {
        let mut app = ObjectBrowserApp::default();
        let value = serde_json::json!({
            "description": "Can perform read and write-level data plane operations for Storage Accounts and Key Vaults.",
            "display_name": "Storage and Key Vault Operator",
            "permissions": [
                {
                    "actions": [
                        "Microsoft.Storage/storageAccounts/blobServices/read",
                        "Microsoft.Storage/storageAccounts/blobServices/generateUserDelegationKey/action"
                    ],
                    "not_actions": [],
                    "data_actions": [],
                    "not_data_actions": []
                }
            ],
            "assignable_scopes": ["/subscriptions/sub-1"],
            "id": "/providers/Microsoft.Authorization/roleDefinitions/00430a36-0657-0ac7-76d9-33e2a4f9e656",
            "kind": "BuiltInRole"
        });
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some("RoleDefinition".to_string()),
            body: super::SlotBody::Unset,
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(super::SlotRuntimeState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            }),
            display_cache: None,
        });

        let description_path = vec![super::JsonPathSegment::Field("description".to_string())];
        assert_eq!(
            app.projection_shape_name_at_path(1, &description_path)
                .as_deref(),
            Some("String")
        );

        let action_path = vec![
            super::JsonPathSegment::Field("permissions".to_string()),
            super::JsonPathSegment::Index(0),
            super::JsonPathSegment::Field("actions".to_string()),
            super::JsonPathSegment::Index(1),
        ];
        assert_eq!(
            app.projection_shape_name_at_path(1, &action_path)
                .as_deref(),
            Some("RolePermissionAction")
        );

        let lines = app.projection_slot_lines(
            &super::ProjectionSlot {
                root_slot_id: 1,
                path: action_path,
                role: super::ProjectionSlotRole::Child,
            },
            None,
        );
        let rendered = lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("RolePermissionAction"), "{rendered}");
        assert!(
            rendered.contains("generateUserDelegationKey/action"),
            "{rendered}"
        );
    }
    #[test]
    fn primitive_projection_cards_show_their_value() {
        let mut app = ObjectBrowserApp::default();
        let value = serde_json::json!({
            "id": "/subscriptions/sub-1/providers/Microsoft.Authorization/roleAssignments/assignment-a"
        });
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some("RoleAssignment".to_string()),
            body: super::SlotBody::Unset,
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(super::SlotRuntimeState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            }),
            display_cache: None,
        });

        let lines = app.projection_slot_lines(
            &super::ProjectionSlot {
                root_slot_id: 1,
                path: vec![super::JsonPathSegment::Field("id".to_string())],
                role: super::ProjectionSlotRole::Child,
            },
            None,
        );
        let rendered = lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("RoleAssignmentId"), "{rendered}");
        assert!(rendered.contains("--- value ---"), "{rendered}");
        assert!(rendered.contains("assignment-a"), "{rendered}");
    }

    #[test]
    fn global_view_hotkeys_toggle_axis_fill_and_help() {
        let mut app = ObjectBrowserApp::default();

        app.handle_event(&ratatui::crossterm::event::Event::Key(KeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(app.slot_axis, super::SlotAxis::Vertical);

        let original_height = app.slot_height;
        app.handle_event(&ratatui::crossterm::event::Event::Key(KeyEvent::new(
            KeyCode::Char('+'),
            KeyModifiers::ALT,
        )));
        assert!(app.slot_height > original_height);
        app.handle_event(&ratatui::crossterm::event::Event::Key(KeyEvent::new(
            KeyCode::Char('-'),
            KeyModifiers::ALT,
        )));
        assert_eq!(app.slot_height, original_height);

        app.handle_event(&ratatui::crossterm::event::Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::ALT,
        )));
        assert!(app.focused_slot_fill);

        app.handle_event(&ratatui::crossterm::event::Event::Key(KeyEvent::new(
            KeyCode::Char('/'),
            KeyModifiers::ALT,
        )));
        assert!(app.show_hotkey_help);
    }
    #[test]
    fn horizontal_navigation_only_scrolls_once_selection_reaches_the_edge() {
        let mut app = ObjectBrowserApp::default();
        app.last_visible_slot_count = 3;

        for _ in 0..5 {
            app.append_slot();
        }

        app.active_slot_index = 0;
        app.slot_view_offset = 0;
        app.move_slot_right();
        assert_eq!(app.slot_view_offset, 0);

        app.move_slot_right();
        assert_eq!(app.slot_view_offset, 0);

        app.move_slot_right();
        assert_eq!(app.slot_view_offset, 1);

        app.move_slot_right();
        assert_eq!(app.slot_view_offset, 2);
    }

    #[test]
    fn ctrl_style_viewport_shift_does_not_move_the_selection() {
        let mut app = ObjectBrowserApp::default();
        app.last_visible_slot_count = 3;

        for _ in 0..5 {
            app.append_slot();
        }

        app.active_slot_index = 1;
        app.slot_view_offset = 0;
        app.shift_slot_view_right(1);

        assert_eq!(app.active_slot_index, 1);
        assert_eq!(app.slot_view_offset, 1);
    }

    #[test]
    fn home_and_end_move_within_the_active_card() {
        let mut app = ObjectBrowserApp::default();
        app.activate_current_row();
        app.last_visible_row_count = 2;

        let shape_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label.contains("AzureTenantArgument"))
            .expect("AzureTenantArgument should be registered for the UI tests");
        app.shape_picker.open(Some(shape_index));
        app.shape_picker.search.list_state.select(Some(shape_index));
        app.apply_shape_selection();

        app.move_row_end();
        assert_eq!(
            app.active_row_index,
            app.active_focusable_rows().saturating_sub(1)
        );

        app.move_row_home();
        assert_eq!(app.active_row_index, 0);
    }

    #[test]
    fn shifted_home_end_and_paging_operate_horizontally() {
        let mut app = ObjectBrowserApp::default();
        app.last_visible_slot_count = 3;

        for _ in 0..5 {
            app.append_slot();
        }

        app.active_slot_index = 2;
        app.page_slots_right();
        assert_eq!(app.active_slot_index, 4);

        app.page_slots_left();
        assert_eq!(app.active_slot_index, 2);

        app.move_slot_end();
        assert_eq!(
            app.active_slot_index,
            app.total_slot_count().saturating_sub(1)
        );

        app.move_slot_home();
        assert_eq!(app.active_slot_index, 0);
    }

    #[test]
    fn scrollbar_overlay_areas_skip_block_corners() {
        let area = ratatui::layout::Rect::new(10, 5, 12, 8);

        assert_eq!(
            super::horizontal_scrollbar_overlay_area(area),
            ratatui::layout::Rect::new(11, 12, 10, 1)
        );
        assert_eq!(
            super::vertical_scrollbar_overlay_area(area),
            ratatui::layout::Rect::new(21, 6, 1, 6)
        );
    }

    #[test]
    fn resolved_array_enters_projection_view() {
        let mut app = ObjectBrowserApp::default();
        let value = serde_json::json!([
            { "displayName": "Ada" },
            { "displayName": "Grace" }
        ]);
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some("Vec<EntraUser>".to_string()),
            body: super::SlotBody::Unset,
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            runtime_state: Some(super::SlotRuntimeState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            }),
            display_cache: None,
        });
        app.active_slot_index = 0;
        app.active_row_index = 1;

        app.activate_current_row();

        assert_eq!(app.projection_stack.len(), 1);
        assert_eq!(app.total_slot_count(), 5);
        assert!(matches!(
            app.pool_entry_at(1),
            Some(super::PoolEntry::Projection(_))
        ));
        assert!(matches!(
            app.pool_entry_at(2),
            Some(super::PoolEntry::Projection(super::ProjectionSlot { path, .. }))
                if matches!(
                    path.as_slice(),
                    [super::JsonPathSegment::Index(0), super::JsonPathSegment::Field(field_name)]
                        if field_name == "displayName"
                )
        ));
    }
}
