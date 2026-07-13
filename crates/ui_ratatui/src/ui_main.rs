use crate::projection_shapes::projection_field_shape as shape_field_shape;
use crate::projection_shapes::projection_fields;
use crate::projection_shapes::projection_map_value_shape as registry_map_value_shape;
use crate::projection_shapes::projection_sequence_element_shape as sequence_element_shape;
use crate::projection_shapes::projection_shape_names;
use cloud_terrastodon_registry::ArbitraryBytes;
use cloud_terrastodon_registry::Function;
use cloud_terrastodon_registry::FunctionInvocation;
use cloud_terrastodon_registry::FunctionKind;
use cloud_terrastodon_registry::KnownShapeInfo;
use cloud_terrastodon_registry::ProductionKind;
use cloud_terrastodon_registry::ReceiverMode;
use cloud_terrastodon_registry::ShapeFieldInfo;
use cloud_terrastodon_registry::ShapeVariantInfo;
use cloud_terrastodon_registry::describe_function;
use cloud_terrastodon_registry::describe_shape;
use cloud_terrastodon_registry::functions_from;
use cloud_terrastodon_registry::functions_to;
use cloud_terrastodon_registry::known_functions;
use cloud_terrastodon_registry::known_shapes;
use cloud_terrastodon_registry::shape_fields_for_thing;
use cloud_terrastodon_registry::shape_variants_for_thing;
use crossterm::event::EventStream;
use eyre::Result;
use facet::ScalarType;
use futures::FutureExt;
use futures::StreamExt;
use nucleo::Matcher;
use nucleo::Nucleo;
use nucleo::Utf32Str;
use nucleo::Utf32String;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;
use rand::RngExt;
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
use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;
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
    breadcrumb_filters: Vec<BreadcrumbFilter>,
    tabs: Vec<BrowserTabState>,
    active_tab_index: usize,
    recent_escape_presses: Vec<Instant>,
    shape_picker: ShapePickerState,
    variant_picker: Option<VariantPickerState>,
    field_picker: Option<FieldPickerState>,
    arbitrary_source_picker: Option<ArbitrarySourcePickerState>,
    function_picker: Option<FunctionPickerState>,
    link_action_picker: Option<LinkActionPickerState>,
    partition_picker: Option<PartitionPickerState>,
    filter_kind_picker: Option<FilterKindPickerState>,
    value_filter_editor: Option<ValueFilterEditorState>,
    value_filter_choice_picker: Option<ValueFilterChoicePickerState>,
    general_value_editor: Option<GeneralValueEditorState>,
    boolean_value_picker: Option<BooleanValuePickerState>,
    rename_slot: Option<RenameSlotState>,
    slot_search: Option<SlotSearchState>,
    projection_search: Option<ProjectionSlotSearchState>,
    slot_axis: SlotAxis,
    focused_slot_fill: bool,
    show_hotkey_help: bool,
    slot_width: u16,
    slot_height: u16,
    shape_choices: Vec<KnownShapeInfo>,
    reflected_shapes: HashMap<String, &'static facet::Shape>,
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
        let mut reflected_shapes = shape_choices
            .iter()
            .map(|choice| (choice.label.clone(), choice.thing.shape))
            .collect::<HashMap<_, _>>();
        for function in known_functions() {
            for shape in [function.input_shape, function.output_shape] {
                reflected_shapes
                    .entry(describe_shape(shape))
                    .or_insert(shape);
            }
        }

        Self {
            should_quit: false,
            mode: UiMode::Pool,
            pool_surface: PoolSurface::Slots,
            active_breadcrumb_index: 0,
            projection_stack: Vec::new(),
            breadcrumb_filters: Vec::new(),
            tabs: vec![BrowserTabState::default()],
            active_tab_index: 0,
            recent_escape_presses: Vec::new(),
            shape_picker,
            variant_picker: None,
            field_picker: None,
            arbitrary_source_picker: None,
            function_picker: None,
            link_action_picker: None,
            partition_picker: None,
            filter_kind_picker: None,
            value_filter_editor: None,
            value_filter_choice_picker: None,
            general_value_editor: None,
            boolean_value_picker: None,
            rename_slot: None,
            slot_search: None,
            projection_search: None,
            slot_axis: SlotAxis::Horizontal,
            focused_slot_fill: false,
            show_hotkey_help: false,
            slot_width: Self::MIN_SLOT_WIDTH,
            slot_height: Self::MIN_SLOT_HEIGHT,
            shape_choices,
            reflected_shapes,
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
        let mut changed = false;
        let mut latest_status = None;
        for slot_index in 0..self.object_slots.len() {
            let is_finished = matches!(
                self.object_slots.get(slot_index).map(|slot| &slot.value_state),
                Some(SlotValueState::Pending(pending)) if pending.join_handle.is_finished()
            );
            if !is_finished {
                continue;
            }

            let Some(slot) = self.object_slots.get_mut(slot_index) else {
                continue;
            };
            let SlotValueState::Pending(pending) = std::mem::replace(
                &mut slot.value_state,
                SlotValueState::Building(SlotBody::Unset),
            ) else {
                unreachable!("finished-state check guarantees a pending slot")
            };

            let next_state = match pending
                .join_handle
                .now_or_never()
                .expect("finished join handle should resolve immediately")
            {
                Ok(Ok(output)) => match (pending.output_serialize)(output.as_ref()) {
                    Ok(json) => match serde_json::from_str::<Value>(&json) {
                        Ok(value) => SlotValueState::ResolvedValue { json, value },
                        Err(error) => SlotValueState::Failed {
                            message: format!("could not parse invocation result json: {error}"),
                        },
                    },
                    Err(error) => SlotValueState::Failed {
                        message: format!("could not serialize invocation result: {error}"),
                    },
                },
                Ok(Err(error)) => SlotValueState::Failed {
                    message: error.to_string(),
                },
                Err(error) => SlotValueState::Failed {
                    message: format!("task join failed: {error}"),
                },
            };
            latest_status = Some(match &next_state {
                SlotValueState::ResolvedValue { .. } => {
                    format!("Result slot {} resolved.", slot.id)
                }
                SlotValueState::Failed { message } => {
                    format!("Result slot {} failed: {message}", slot.id)
                }
                SlotValueState::Building(_) | SlotValueState::Pending(_) => {
                    unreachable!("a completed invocation only resolves or fails")
                }
            });
            slot.value_state = next_state;
            changed = true;
        }
        if let Some(status) = latest_status {
            self.status_message = status;
        }
        if changed {
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

        let title = Line::from(format!("Tab {}", self.active_tab_index + 1))
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
            UiMode::ArbitrarySourcePicker => self.draw_arbitrary_source_picker_popup(frame),
            UiMode::FunctionPicker => self.draw_function_picker_popup(frame),
            UiMode::LinkActionPicker => self.draw_link_action_picker_popup(frame),
            UiMode::PartitionPicker => self.draw_partition_picker_popup(frame),
            UiMode::FilterKindPicker => self.draw_filter_kind_picker_popup(frame),
            UiMode::ValueFilterEditor => self.draw_value_filter_editor_popup(frame),
            UiMode::ValueFilterChoicePicker => self.draw_value_filter_choice_picker_popup(frame),
            UiMode::GeneralValueEditor => self.draw_general_value_editor_popup(frame),
            UiMode::BooleanValuePicker => self.draw_boolean_value_picker_popup(frame),
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
                    .position(self.slot_view_offset)
                    .viewport_content_length(visible.len());
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
        let scroll_offset = self.row_view_offset;
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
        let preview_lines = self.field_picker_preview_lines().unwrap_or_default();
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
    fn draw_arbitrary_source_picker_popup(&mut self, frame: &mut Frame) {
        let Some(preview_lines) = self.arbitrary_source_picker_preview_lines() else {
            return;
        };
        let Some((items, total_count)) = self
            .arbitrary_source_picker
            .as_ref()
            .map(|picker| (picker.list_items(), picker.labels.len()))
        else {
            return;
        };
        let search = &mut self
            .arbitrary_source_picker
            .as_mut()
            .expect("picker exists")
            .search;
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
        let preview_lines = self.link_action_preview_lines();
        let Some((items, total_count)) = self
            .link_action_picker
            .as_ref()
            .map(|picker| (picker.list_items(), picker.labels.len()))
        else {
            return;
        };
        let search = &mut self
            .link_action_picker
            .as_mut()
            .expect("picker exists")
            .search;
        draw_picker_popup(
            frame,
            "Move or Clone",
            "Consequence",
            search,
            items,
            total_count,
            preview_lines,
        );
    }

    fn draw_partition_picker_popup(&mut self, frame: &mut Frame) {
        let Some(picker) = self.partition_picker.as_mut() else {
            return;
        };
        let area = centered_rect(88, 82, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Filter Shapes")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [lists_area, search_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(inner);
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(lists_area);

        let build_items = |included_side: bool| {
            picker
                .search
                .filtered_indices
                .iter()
                .map(|index| {
                    let on_side = picker.included_indices.contains(index) == included_side;
                    let label = if on_side {
                        picker.labels[*index].clone()
                    } else {
                        Default::default()
                    };
                    let style = if picker.selected_indices.contains(index) {
                        Style::default().bg(Color::Blue).fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(label).style(style)
                })
                .collect::<Vec<_>>()
        };
        let mut left_state = ListState::default();
        left_state.select(picker.search.list_state.selected());
        let mut right_state = ListState::default();
        right_state.select(picker.search.list_state.selected());
        frame.render_stateful_widget(
            List::new(build_items(false)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Available (Left)"),
            ),
            left_area,
            &mut left_state,
        );
        frame.render_stateful_widget(
            List::new(build_items(true)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Included (Right)"),
            ),
            right_area,
            &mut right_state,
        );
        picker
            .search
            .query
            .set_block(Block::default().borders(Borders::ALL).title("Search"));
        picker.search.query.render(search_area, frame.buffer_mut());
    }

    fn draw_filter_kind_picker_popup(&mut self, frame: &mut Frame) {
        let Some(picker) = self.filter_kind_picker.as_mut() else {
            return;
        };
        let items = picker
            .search
            .filtered_indices
            .iter()
            .filter_map(|index| picker.labels.get(*index))
            .cloned()
            .map(ListItem::new)
            .collect();
        draw_picker_popup(
            frame,
            "Add Breadcrumb",
            "Filter",
            &mut picker.search,
            items,
            picker.labels.len(),
            vec![
                Line::from("filter shape: include selected projected shapes"),
                Line::from("filter value: match shape/name/value metadata"),
                Line::from("filter slot kind: show owned, view, and/or projection slots"),
            ],
        );
    }

    fn draw_value_filter_choice_picker_popup(&mut self, frame: &mut Frame) {
        let Some(picker) = self.value_filter_choice_picker.as_mut() else {
            return;
        };
        picker.refresh_worker();
        let items = picker
            .search
            .filtered_indices
            .iter()
            .filter_map(|index| picker.labels.get(*index))
            .cloned()
            .map(ListItem::new)
            .collect();
        draw_picker_popup(
            frame,
            "Choose Value Filter Property",
            "Selection",
            &mut picker.search,
            items,
            picker.labels.len(),
            vec![Line::from("Enter: choose | Esc: return to editor")],
        );
    }

    fn draw_value_filter_editor_popup(&mut self, frame: &mut Frame) {
        let Some(editor) = self.value_filter_editor.as_ref() else {
            return;
        };
        let literal = editor
            .literal_input
            .lines()
            .first()
            .map(String::as_str)
            .unwrap_or("");
        let shown_value = if literal.is_empty() {
            editor.draft.value.as_str()
        } else {
            literal
        };
        let rows = [
            format!("field shape: {}", editor.draft.field_shape),
            format!("field name: {}", editor.draft.field_name),
            format!("operator: {}", editor.draft.operator.label()),
            format!("value source: {}", editor.source.label()),
            format!("value: {shown_value}"),
            "save filter".to_string(),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let style = if index == editor.active_row {
                Style::default().bg(Color::Blue).fg(Color::Yellow)
            } else {
                Default::default()
            };
            ListItem::new(label).style(style)
        })
        .collect::<Vec<_>>();
        let area = centered_rect(72, 58, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Filter Value")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [form_area, hint_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(inner);
        frame.render_widget(List::new(rows), form_area);
        frame.render_widget(
            Paragraph::new(
                "Up/Down: field | Enter: choose/save | Left/Right: operator/source | Type: literal value | Esc: cancel",
            )
            .wrap(Wrap { trim: true }),
            hint_area,
        );
    }

    fn draw_general_value_editor_popup(&mut self, frame: &mut Frame) {
        let Some(editor) = self.general_value_editor.as_mut() else {
            return;
        };
        let area = centered_rect(70, 34, frame.area());
        frame.render_widget(Clear, area);
        let border_color = editor
            .validation_error
            .as_ref()
            .map_or(Color::Cyan, |_| Color::Red);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Edit {}", editor.shape_name))
            .border_style(Style::default().fg(border_color));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let error_height = editor
            .validation_error
            .as_ref()
            .map(|message| message.lines().count().clamp(1, 3) as u16)
            .unwrap_or(0);
        let [hint_area, input_area, error_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(error_height),
        ])
        .areas(inner);
        frame.render_widget(
            Paragraph::new("Enter: save | Esc: cancel | Value is validated before saving"),
            hint_area,
        );
        let input_border = if editor.validation_error.is_some() {
            Color::Red
        } else {
            Color::DarkGray
        };
        editor.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Value")
                .border_style(Style::default().fg(input_border)),
        );
        editor.textarea.render(input_area, frame.buffer_mut());
        if let Some(message) = &editor.validation_error {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    message.clone(),
                    Style::default().fg(Color::Red),
                )))
                .wrap(Wrap { trim: true }),
                error_area,
            );
        }
    }

    fn draw_boolean_value_picker_popup(&mut self, frame: &mut Frame) {
        let Some(picker) = self.boolean_value_picker.as_mut() else {
            return;
        };
        let items = picker
            .search
            .filtered_indices
            .iter()
            .filter_map(|index| picker.labels.get(*index))
            .cloned()
            .map(ListItem::new)
            .collect::<Vec<_>>();
        let area = centered_rect(44, 34, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Choose {}", picker.shape_name))
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [list_area, hint_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(2)]).areas(inner);
        let mut list_state = picker.search.list_state.clone();
        frame.render_stateful_widget(List::new(items), list_area, &mut list_state);
        frame.render_widget(
            Paragraph::new("Up/Down: choose | Enter: save | Esc: cancel"),
            hint_area,
        );
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
            Line::from("Shift+[/Shift+]: previous/next tab"),
            Line::from("Shift+;: add filter breadcrumb"),
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
        if self.show_hotkey_help && key.code == KeyCode::Esc {
            self.show_hotkey_help = false;
            return;
        }
        if self.mode == UiMode::Pool
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char('[') | KeyCode::Char('{'))
        {
            self.switch_tab_previous();
            return;
        }
        if self.mode == UiMode::Pool
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char(']') | KeyCode::Char('}'))
        {
            self.switch_tab_next();
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
        if self.mode == UiMode::Pool
            && key.modifiers.contains(KeyModifiers::SHIFT)
            && matches!(key.code, KeyCode::Char(';') | KeyCode::Char(':'))
        {
            self.pool_surface = PoolSurface::Breadcrumbs;
            self.active_breadcrumb_index = self.breadcrumb_count().saturating_sub(1);
            self.status_message =
                "Add breadcrumb selected; press Enter to choose a filter.".to_string();
            return;
        }

        match self.mode {
            UiMode::Pool => self.handle_pool_key(*key),
            UiMode::SlotSearch => self.handle_slot_search_key(*key),
            UiMode::ShapePicker => self.handle_shape_picker_key(*key),
            UiMode::VariantPicker => self.handle_variant_picker_key(*key),
            UiMode::FieldPicker => self.handle_field_picker_key(*key),
            UiMode::ArbitrarySourcePicker => self.handle_arbitrary_source_picker_key(*key),
            UiMode::FunctionPicker => self.handle_function_picker_key(*key),
            UiMode::LinkActionPicker => self.handle_link_action_picker_key(*key),
            UiMode::PartitionPicker => self.handle_partition_picker_key(*key),
            UiMode::FilterKindPicker => self.handle_filter_kind_picker_key(*key),
            UiMode::ValueFilterEditor => self.handle_value_filter_editor_key(*key),
            UiMode::ValueFilterChoicePicker => self.handle_value_filter_choice_picker_key(*key),
            UiMode::GeneralValueEditor => self.handle_general_value_editor_key(*key),
            UiMode::BooleanValuePicker => self.handle_boolean_value_picker_key(*key),
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
                KeyCode::Delete | KeyCode::Backspace => self.delete_current_breadcrumb(),
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

    fn current_tab_state(&self) -> BrowserTabState {
        BrowserTabState {
            projection_stack: self.projection_stack.clone(),
            breadcrumb_filters: self.breadcrumb_filters.clone(),
            pool_surface: self.pool_surface,
            active_breadcrumb_index: self.active_breadcrumb_index,
            active_slot_index: self.active_slot_index,
            active_row_index: self.active_row_index,
            slot_view_offset: self.slot_view_offset,
            row_view_offset: self.row_view_offset,
        }
    }

    fn save_current_tab(&mut self) {
        let state = self.current_tab_state();
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            *tab = state;
        }
    }

    fn load_active_tab(&mut self) {
        let Some(tab) = self.tabs.get(self.active_tab_index).cloned() else {
            return;
        };
        self.projection_stack = tab.projection_stack;
        self.breadcrumb_filters = tab.breadcrumb_filters;
        self.pool_surface = tab.pool_surface;
        self.active_breadcrumb_index = tab
            .active_breadcrumb_index
            .min(self.breadcrumb_count().saturating_sub(1));
        self.active_slot_index = tab.active_slot_index;
        self.active_row_index = tab.active_row_index;
        self.slot_view_offset = tab.slot_view_offset;
        self.row_view_offset = tab.row_view_offset;
        self.projection_cache.borrow_mut().clear();
        self.active_slot_index = self
            .active_slot_index
            .min(self.total_slot_count().saturating_sub(1));
        self.clamp_active_row();
        self.sync_selection_viewports();
        self.status_message = format!("Switched to Tab {}.", self.active_tab_index + 1);
    }

    fn switch_tab_previous(&mut self) {
        if self.active_tab_index == 0 {
            return;
        }
        self.save_current_tab();
        self.active_tab_index -= 1;
        self.load_active_tab();
    }

    fn switch_tab_next(&mut self) {
        self.save_current_tab();
        if self.active_tab_index + 1 == self.tabs.len() {
            self.tabs.push(BrowserTabState::default());
        }
        self.active_tab_index += 1;
        self.load_active_tab();
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
                if let Some(slot_search) = self.slot_search.as_mut() {
                    if slot_search.query.input(key) {
                        slot_search.query.cancel_selection();
                        slot_search.query.move_cursor(CursorMove::End);
                        self.refresh_slot_search(false);
                    }
                } else if let Some(projection_search) = self.projection_search.as_mut() {
                    if projection_search.query.input(key) {
                        projection_search.query.cancel_selection();
                        projection_search.query.move_cursor(CursorMove::End);
                        self.refresh_projection_search();
                    }
                } else {
                    self.mode = UiMode::Pool;
                }
            }
        }
    }

    fn handle_escape(&mut self) {
        if !self.projection_stack.is_empty() || !self.breadcrumb_filters.is_empty() {
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

    fn handle_arbitrary_source_picker_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.arbitrary_source_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };

        match picker.search.handle_key(key, &picker.labels) {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.arbitrary_source_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "ArbitraryBytes selection cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_arbitrary_source_picker_selection(),
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

        match link_action_picker
            .search
            .handle_key(key, &link_action_picker.labels)
        {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.link_action_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "Move/clone selection cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_link_action_selection(),
        }
    }

    fn handle_partition_picker_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.partition_picker = None;
            self.mode = UiMode::Pool;
            self.status_message = "Breadcrumb filter cancelled.".to_string();
            return;
        }
        if key.code == KeyCode::Enter {
            if let Some(picker) = self.partition_picker.as_mut()
                && picker.included_indices.is_empty()
                && let Some(index) = picker.current_index()
            {
                picker.included_indices.insert(index);
            }
            self.apply_partition_picker_selection();
            return;
        }
        let Some(picker) = self.partition_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('a') | KeyCode::Char('A'))
        {
            picker.select_all_filtered();
            return;
        }
        match key.code {
            KeyCode::Left => picker.move_selected(false),
            KeyCode::Right => picker.move_selected(true),
            KeyCode::Tab | KeyCode::BackTab => picker.toggle_selected(),
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageUp
            | KeyCode::PageDown => {
                match key.code {
                    KeyCode::Up => picker.search.list_state.select_previous(),
                    KeyCode::Down => picker.search.list_state.select_next(),
                    KeyCode::Home if !picker.search.filtered_indices.is_empty() => {
                        picker.search.list_state.select(Some(0));
                    }
                    KeyCode::End if !picker.search.filtered_indices.is_empty() => {
                        picker
                            .search
                            .list_state
                            .select(Some(picker.search.filtered_indices.len().saturating_sub(1)));
                    }
                    KeyCode::PageUp if !picker.search.filtered_indices.is_empty() => {
                        let position = picker
                            .search
                            .list_state
                            .selected()
                            .unwrap_or(0)
                            .saturating_sub(10);
                        picker.search.list_state.select(Some(position));
                    }
                    KeyCode::PageDown if !picker.search.filtered_indices.is_empty() => {
                        let position = picker
                            .search
                            .list_state
                            .selected()
                            .unwrap_or(0)
                            .saturating_add(10)
                            .min(picker.search.filtered_indices.len().saturating_sub(1));
                        picker.search.list_state.select(Some(position));
                    }
                    _ => {}
                }
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    picker.extend_selection_to_current();
                } else {
                    picker.reset_selection_to_current();
                }
            }
            _ => {
                let previous_query = picker.search.query.lines().join("\n");
                if picker.search.query.input(key) {
                    picker.search.query.cancel_selection();
                    picker.search.query.move_cursor(CursorMove::End);
                    let query = picker.search.query.lines().join("\n");
                    if query != previous_query {
                        picker.search.filtered_indices = filter_indices(&query, &picker.labels);
                        picker.search.select_preferred(None);
                        picker.search.preview_scroll = 0;
                        picker.reset_selection_to_current();
                    }
                }
            }
        }
    }

    fn handle_filter_kind_picker_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.filter_kind_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };
        match picker.search.handle_key(key, &picker.labels) {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.filter_kind_picker = None;
                self.mode = UiMode::Pool;
            }
            PickerSearchAction::Submit => {
                let selected = picker.search.selected_filtered_index();
                self.filter_kind_picker = None;
                match selected {
                    Some(0) => self.open_shape_filter_picker(None),
                    Some(2) => self.open_slot_kind_filter_picker(None),
                    Some(1) => self.open_value_filter_editor(
                        None,
                        ValueFilterView {
                            field_shape: "*".to_string(),
                            field_name: "*".to_string(),
                            operator: ValueFilterOperator::Equals,
                            value: String::new(),
                        },
                    ),
                    _ => self.mode = UiMode::Pool,
                }
            }
        }
    }

    fn handle_value_filter_editor_key(&mut self, key: KeyEvent) {
        let Some(editor) = self.value_filter_editor.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.value_filter_editor = None;
                self.mode = UiMode::Pool;
            }
            KeyCode::Up => editor.active_row = editor.active_row.saturating_sub(1),
            KeyCode::Down => editor.active_row = (editor.active_row + 1).min(5),
            KeyCode::Home | KeyCode::PageUp => editor.active_row = 0,
            KeyCode::End | KeyCode::PageDown => editor.active_row = 5,
            KeyCode::Left | KeyCode::Right if editor.active_row == 2 => {
                editor.draft.operator = match editor.draft.operator {
                    ValueFilterOperator::Equals => ValueFilterOperator::NotEquals,
                    ValueFilterOperator::NotEquals => ValueFilterOperator::Contains,
                    ValueFilterOperator::Contains => ValueFilterOperator::Equals,
                };
            }
            KeyCode::Left | KeyCode::Right if editor.active_row == 3 => {
                editor.source = match editor.source {
                    ValueFilterSource::Existing => ValueFilterSource::Literal,
                    ValueFilterSource::Literal => ValueFilterSource::Existing,
                };
            }
            KeyCode::Enter => {
                let row = editor.active_row;
                let source = editor.source;
                match row {
                    0 => self.open_value_filter_choice(ValueFilterChoiceTarget::FieldShape),
                    1 => self.open_value_filter_choice(ValueFilterChoiceTarget::FieldName),
                    2 => self.open_value_filter_choice(ValueFilterChoiceTarget::Operator),
                    3 => {
                        if let Some(editor) = self.value_filter_editor.as_mut() {
                            editor.source = match editor.source {
                                ValueFilterSource::Existing => ValueFilterSource::Literal,
                                ValueFilterSource::Literal => ValueFilterSource::Existing,
                            };
                        }
                    }
                    4 if source == ValueFilterSource::Existing => {
                        self.open_value_filter_choice(ValueFilterChoiceTarget::ExistingValue)
                    }
                    4 => {
                        if let Some(editor) = self.value_filter_editor.as_mut() {
                            editor.active_row = 5;
                        }
                    }
                    5 => self.apply_value_filter_editor(),
                    _ => {}
                }
            }
            _ if editor.active_row == 4 && editor.source == ValueFilterSource::Literal => {
                editor.literal_input.input(key);
            }
            _ => {}
        }
    }

    fn handle_value_filter_choice_picker_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.value_filter_choice_picker.as_mut() else {
            self.mode = UiMode::ValueFilterEditor;
            return;
        };
        match picker.handle_key(key) {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.value_filter_choice_picker = None;
                self.mode = UiMode::ValueFilterEditor;
            }
            PickerSearchAction::Submit => {
                let target = picker.target;
                let selected = picker
                    .search
                    .selected_filtered_index()
                    .and_then(|index| picker.labels.get(index))
                    .cloned();
                self.value_filter_choice_picker = None;
                self.mode = UiMode::ValueFilterEditor;
                let Some(selected) = selected else {
                    return;
                };
                let Some(editor) = self.value_filter_editor.as_mut() else {
                    return;
                };
                match target {
                    ValueFilterChoiceTarget::FieldShape => {
                        editor.draft.field_shape = selected;
                        editor.draft.field_name = "*".to_string();
                    }
                    ValueFilterChoiceTarget::FieldName => editor.draft.field_name = selected,
                    ValueFilterChoiceTarget::Operator => {
                        editor.draft.operator = match selected.as_str() {
                            "not equals" => ValueFilterOperator::NotEquals,
                            "contains" => ValueFilterOperator::Contains,
                            _ => ValueFilterOperator::Equals,
                        }
                    }
                    ValueFilterChoiceTarget::ExistingValue => {
                        editor.draft.value = selected.clone();
                        editor.literal_input = build_text_area(&selected);
                    }
                }
            }
        }
    }

    fn handle_general_value_editor_key(&mut self, key: KeyEvent) {
        let Some(editor) = self.general_value_editor.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };
        match key.code {
            KeyCode::Esc => {
                self.general_value_editor = None;
                self.mode = UiMode::Pool;
                self.status_message = "Value editing cancelled.".to_string();
            }
            KeyCode::Enter => self.apply_general_value_editor(),
            _ => {
                if editor.textarea.input(key) {
                    editor.validation_error = None;
                }
            }
        }
    }

    fn handle_boolean_value_picker_key(&mut self, key: KeyEvent) {
        let Some(picker) = self.boolean_value_picker.as_mut() else {
            self.mode = UiMode::Pool;
            return;
        };
        match picker.search.handle_key(key, &picker.labels) {
            PickerSearchAction::None => {}
            PickerSearchAction::Cancel => {
                self.boolean_value_picker = None;
                self.mode = UiMode::Pool;
                self.status_message = "Boolean value editing cancelled.".to_string();
            }
            PickerSearchAction::Submit => self.apply_boolean_value_picker(),
        }
    }

    fn apply_general_value_editor(&mut self) {
        let Some((slot_id, shape_name, shape, input)) =
            self.general_value_editor.as_ref().map(|editor| {
                (
                    editor.slot_id,
                    editor.shape_name.clone(),
                    editor.shape,
                    editor.textarea.lines().join("\n"),
                )
            })
        else {
            return;
        };

        match self.parse_general_value(&shape_name, shape, &input) {
            Ok(value) => {
                self.general_value_editor = None;
                self.set_scalar_slot_value(slot_id, value);
                self.mode = UiMode::Pool;
                self.status_message = format!("Updated value in slot {slot_id}.");
            }
            Err(error) => {
                if let Some(editor) = self.general_value_editor.as_mut() {
                    editor.validation_error = Some(error.to_string());
                }
                self.status_message = "Value is invalid; fix the error before saving.".to_string();
            }
        }
    }

    fn apply_boolean_value_picker(&mut self) {
        let Some(picker) = self.boolean_value_picker.as_ref() else {
            return;
        };
        let Some(index) = picker.search.selected_filtered_index() else {
            self.status_message = "No boolean value is selected.".to_string();
            return;
        };
        let Some(label) = picker.labels.get(index) else {
            return;
        };
        let value = label == "true";
        let slot_id = picker.slot_id;
        self.boolean_value_picker = None;
        self.set_scalar_slot_value(slot_id, Value::Bool(value));
        self.mode = UiMode::Pool;
        self.status_message = format!("Updated value in slot {slot_id}.");
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
        let mut query = build_text_area("");
        if !query.input(key) {
            return;
        }
        query.cancel_selection();
        query.move_cursor(CursorMove::End);

        match self.current_pool_entry() {
            Some(PoolEntry::RealSlot(slot_id)) => {
                self.slot_search = Some(SlotSearchState {
                    slot_id,
                    query,
                    filtered_matches: Vec::new(),
                    selected_match_index: 0,
                });
                self.mode = UiMode::SlotSearch;
                self.refresh_slot_search(false);
            }
            Some(PoolEntry::Projection(projection)) => {
                self.projection_search = Some(ProjectionSlotSearchState {
                    projection,
                    query,
                    filtered_matches: Vec::new(),
                    selected_match_index: 0,
                });
                self.mode = UiMode::SlotSearch;
                self.refresh_projection_search();
            }
            Some(PoolEntry::NewSlot) | None => {}
        }
    }

    fn cancel_slot_search(&mut self) {
        self.slot_search = None;
        self.projection_search = None;
        self.mode = UiMode::Pool;
        self.status_message = "Row search cancelled.".to_string();
    }

    fn submit_slot_search(&mut self) {
        let has_match = self.slot_search_current_target().is_some()
            || self.projection_search_current_row().is_some();
        self.slot_search = None;
        self.projection_search = None;
        self.mode = UiMode::Pool;
        if has_match {
            self.activate_current_row();
        } else {
            self.status_message = "No matching rows in the active slot.".to_string();
        }
    }

    fn move_slot_search_selection(&mut self, direction: isize) {
        if self.slot_search.is_none() {
            let Some(search) = self.projection_search.as_mut() else {
                return;
            };
            if search.filtered_matches.is_empty() {
                return;
            }
            let max_index = search.filtered_matches.len().saturating_sub(1) as isize;
            search.selected_match_index =
                (search.selected_match_index as isize + direction).clamp(0, max_index) as usize;
            self.sync_slot_search_selection();
            return;
        }
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
        if self.slot_search.is_none() {
            let Some(search) = self.projection_search.as_mut() else {
                return;
            };
            if search.filtered_matches.is_empty() {
                return;
            }
            search.selected_match_index = if to_start {
                0
            } else {
                search.filtered_matches.len().saturating_sub(1)
            };
            self.sync_slot_search_selection();
            return;
        }
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
        if self.slot_search.is_none() {
            let step = self.last_visible_row_count.saturating_sub(1).max(1) as isize;
            let Some(search) = self.projection_search.as_mut() else {
                return;
            };
            if search.filtered_matches.is_empty() {
                return;
            }
            let max_index = search.filtered_matches.len().saturating_sub(1) as isize;
            search.selected_match_index = (search.selected_match_index as isize + direction * step)
                .clamp(0, max_index) as usize;
            self.sync_slot_search_selection();
            return;
        }
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

    fn refresh_projection_search(&mut self) {
        let Some((projection, query)) = self
            .projection_search
            .as_ref()
            .map(|search| (search.projection.clone(), search.query.lines().concat()))
        else {
            return;
        };
        let filtered_matches = self.projection_search_matches(&projection, &query);
        if let Some(search) = self.projection_search.as_mut() {
            search.filtered_matches = filtered_matches;
            search.selected_match_index = 0;
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
        } else if let Some(row_index) = self.projection_search_current_row() {
            self.active_row_index = row_index;
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

    fn projection_search_current_row(&self) -> Option<usize> {
        let search = self.projection_search.as_ref()?;
        search
            .filtered_matches
            .get(search.selected_match_index)
            .map(|matched| matched.row_index)
    }

    fn update_slot_search_status(&mut self) {
        if let Some(search) = self.projection_search.as_ref() {
            let query = search.query.lines().concat();
            let match_count = search.filtered_matches.len();
            self.status_message = format!(
                "Search {}: {} ({} match{}) | Up/Down/PgUp/PgDn: navigate | Enter: activate | Esc: cancel",
                projection_label(search.projection.root_slot_id, &search.projection.path),
                if query.is_empty() {
                    "<empty>"
                } else {
                    query.as_str()
                },
                match_count,
                if match_count == 1 { "" } else { "es" },
            );
            return;
        }
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
                .saturating_add(self.breadcrumb_filters.len())
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
            Some(PoolEntry::Projection(projection)) => self
                .projection_search
                .as_ref()
                .filter(|search| search.projection == projection)
                .map(|search| search.filtered_matches.len().max(1))
                .unwrap_or_else(|| self.projection_rendered_line_count(&projection)),
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
            Some(PoolEntry::Projection(projection)) => self
                .projection_search
                .as_ref()
                .filter(|search| search.projection == projection)
                .map(|search| search.selected_match_index)
                .unwrap_or_else(|| self.projection_line_index(&projection, self.active_row_index)),
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

        if self.is_general_value_field(owner_slot_id, field_index) {
            if let FieldValueState::Linked { slot_id } = field.value_state {
                self.jump_to_slot(slot_id);
                self.status_message = format!(
                    "Jumped to slot {} for {} on slot {}.",
                    slot_id, field.info.field_name, owner_slot_id
                );
            } else {
                let wrapper_slot = self
                    .slot_shape_name(owner_slot_id)
                    .and_then(|shape_name| self.shape_for_shape_name(shape_name))
                    .is_some_and(is_general_value_shape);
                if field_index == 0
                    && wrapper_slot
                    && self.promote_general_value_slot(owner_slot_id)
                {
                    self.open_slot_value_editor(owner_slot_id);
                } else {
                    self.open_general_field_picker(owner_slot_id, field_index, &field);
                }
            }
            return;
        }
        let required_shape_name =
            self.field_shape_name_for_matching(owner_slot_id, field_index, &field);
        if !self.has_known_shape_label(&required_shape_name) {
            self.toggle_default_field_value(owner_slot_id, field_index);
            return;
        }

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
        if let Some(thing) = self.thing_for_shape_name(&required_shape_name) {
            choices.extend(shape_variants_for_thing(thing).into_iter().enumerate().map(
                |(variant_index, variant)| FieldPickerChoice::CreateNewVariant {
                    variant_index,
                    variant_name: variant.variant_name.to_string(),
                },
            ));
        }
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
                        matches!(choice, FieldPickerChoice::InvokeDefaultProducer { .. })
                            && !field_picker_choice_is_arbitrary_producer(choice)
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
            required_shape_name.clone(),
            choices,
            labels,
            preferred_index,
        ));
        self.mode = UiMode::FieldPicker;
        self.status_message =
            "Choose an object for the field, or a request that can produce one. Type to search; PgUp/PgDn scrolls the preview pane."
                .to_string();
    }

    fn open_general_field_picker(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        field: &ObjectFieldState,
    ) {
        let required_shape_name =
            self.field_shape_name_for_matching(owner_slot_id, field_index, field);
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
        choices.push(FieldPickerChoice::CreateNewValue);
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
                        matches!(choice, FieldPickerChoice::InvokeDefaultProducer { .. })
                            && !field_picker_choice_is_arbitrary_producer(choice)
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
                        .position(|choice| choice == &FieldPickerChoice::CreateNewValue)
                }),
        };
        self.field_picker = Some(FieldPickerState::new(
            owner_slot_id,
            field_index,
            required_shape_name.clone(),
            choices,
            labels,
            preferred_index,
        ));
        self.mode = UiMode::FieldPicker;
        self.status_message = format!(
            "Choose how to create or link the {} value. Type to search.",
            required_shape_name
        );
    }
    fn is_general_value_field(&self, owner_slot_id: usize, field_index: usize) -> bool {
        self.field_shape_for_field(owner_slot_id, field_index)
            .is_some_and(is_general_value_shape)
    }
    fn field_shape_for_field(
        &self,
        owner_slot_id: usize,
        field_index: usize,
    ) -> Option<&'static facet::Shape> {
        let field = self.slot_field(owner_slot_id, field_index)?;
        let parent_shape = self
            .slot_shape_name(owner_slot_id)
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))?;
        if field.info.field_name == "0" && is_general_value_shape(parent_shape) {
            return Some(parent_shape);
        }
        if let Some(shape) = self.shape_for_shape_name(&field.info.field_shape_name) {
            return Some(shape);
        }
        match parent_shape.ty {
            facet::Type::User(facet::UserType::Struct(_)) => {
                shape_field_shape(parent_shape, field.info.field_name)
            }
            facet::Type::User(facet::UserType::Enum(enum_type)) => {
                let selected_variant =
                    self.slot_body(owner_slot_id).and_then(|body| match body {
                        SlotBody::Enum {
                            selected_variant, ..
                        } => *selected_variant,
                        _ => None,
                    })?;
                enum_type
                    .variants
                    .get(selected_variant)?
                    .data
                    .fields
                    .get(field_index)
                    .map(|field| field.proxy_shape().unwrap_or_else(|| field.shape()))
            }
            _ => None,
        }
    }

    fn field_shape_name_for_matching(
        &self,
        owner_slot_id: usize,
        field_index: usize,
        field: &ObjectFieldState,
    ) -> String {
        self.field_shape_for_field(owner_slot_id, field_index)
            .map(|shape| describe_shape(shape.inner.unwrap_or(shape)))
            .unwrap_or_else(|| field.info.field_shape_name.clone())
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
            slot.value_state = match snapshot.value_state {
                SlotSnapshotValueState::Building(body) => SlotValueState::Building(body),
                SlotSnapshotValueState::ResolvedValue { json, value } => {
                    SlotValueState::ResolvedValue { json, value }
                }
            };
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
                self.link_result_to_created_field(slot_id, result_slot_id);
                let result_index = self.slot_by_id_mut(slot_id).map(|slot| {
                    let result_index = slot.result_slot_ids.len();
                    slot.result_slot_ids.push(result_slot_id);
                    result_index
                });
                self.invalidate_all_slot_display_caches();
                if self
                    .slot_by_id(slot_id)
                    .is_none_or(|slot| slot.created_for.is_none())
                    && let Some(result_index) = result_index
                {
                    self.jump_to_slot_target(slot_id, SlotFocusTarget::Result(result_index));
                }
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
            slot.value_state = SlotValueState::ResolvedValue { json, value };
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
        self.link_result_to_created_field(slot_id, result_slot_id);
        let result_index = self.slot_by_id_mut(slot_id).map(|slot| {
            let result_index = slot.result_slot_ids.len();
            slot.result_slot_ids.push(result_slot_id);
            result_index
        });
        self.invalidate_all_slot_display_caches();
        if self
            .slot_by_id(slot_id)
            .is_none_or(|slot| slot.created_for.is_none())
            && let Some(result_index) = result_index
        {
            self.jump_to_slot_target(slot_id, SlotFocusTarget::Result(result_index));
        }
        self.status_message = format!(
            "Invoked {} into result slot {}.",
            describe_function(function),
            result_slot_id
        );
    }

    fn link_result_to_created_field(&mut self, producer_slot_id: usize, result_slot_id: usize) {
        let Some(created_for) = self
            .slot_by_id(producer_slot_id)
            .and_then(|slot| slot.created_for)
        else {
            return;
        };
        self.set_field_link(
            created_for.owner_slot_id,
            created_for.field_index,
            result_slot_id,
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

        let choices = self
            .object_slots
            .iter()
            .filter(|slot| slot.id != slot_id)
            .filter(|slot| self.slot_shape_name(slot.id) == Some("ArbitraryBytes"))
            .filter(|slot| {
                let is_owned = matches!(slot.kind, SlotKind::Owned);
                constructor.supports_slot_kind(is_owned)
            })
            .map(|slot| ArbitrarySourceChoice::ExistingSlot { slot_id: slot.id })
            .chain(std::iter::once(ArbitrarySourceChoice::CreateNew))
            .collect::<Vec<_>>();
        let output_shape_name = describe_shape(function.output_shape);
        let labels = choices
            .iter()
            .map(|choice| match choice {
                ArbitrarySourceChoice::ExistingSlot { slot_id } => format!(
                    "{} [produces {}]",
                    self.slot_picker_label(*slot_id),
                    output_shape_name
                ),
                ArbitrarySourceChoice::CreateNew => "+ create new ArbitraryBytes".to_string(),
            })
            .collect();
        self.arbitrary_source_picker = Some(ArbitrarySourcePickerState::new(
            slot_id,
            function,
            constructor,
            choices,
            labels,
        ));
        self.mode = UiMode::ArbitrarySourcePicker;
        self.status_message = format!(
            "Choose the ArbitraryBytes source for {}.",
            output_shape_name
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
            Some(SlotBody::Value { .. }) => format!("Value slot created for {}.", choice.label),
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
                "Selected {}::{}. This payload is a {} scalar; edit it from its value slot.",
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
            } => {
                self.create_producer_request_for_field(
                    owner_slot_id,
                    field_index,
                    &input_shape_name,
                    &required_shape_name,
                );
            }
            FieldPickerChoice::InvokeDefaultProducer {
                input_shape_name,
                function_label,
            } => self.invoke_default_producer_for_field(
                owner_slot_id,
                field_index,
                &input_shape_name,
                &required_shape_name,
                &function_label,
            ),
            FieldPickerChoice::InvokeArbitraryProducer {
                input_shape_name,
                function_label,
            } => self.invoke_arbitrary_producer_for_field(
                owner_slot_id,
                field_index,
                &input_shape_name,
                &required_shape_name,
                &function_label,
            ),
            FieldPickerChoice::CreateNewValue => {
                self.create_general_value_slot(owner_slot_id, field_index)
            }
            FieldPickerChoice::CreateNew => {
                self.create_field_object(owner_slot_id, field_index, &required_shape_name, None)
            }
            FieldPickerChoice::CreateNewVariant { variant_index, .. } => self.create_field_object(
                owner_slot_id,
                field_index,
                &required_shape_name,
                Some(variant_index),
            ),
        }
    }

    fn apply_arbitrary_source_picker_selection(&mut self) {
        let Some((request_slot_id, request_function, constructor, choice)) =
            self.arbitrary_source_picker.as_ref().and_then(|picker| {
                Some((
                    picker.request_slot_id,
                    picker.request_function,
                    picker.constructor,
                    picker.selected_choice()?,
                ))
            })
        else {
            self.status_message = "No ArbitraryBytes source is selected.".to_string();
            return;
        };

        self.arbitrary_source_picker = None;
        self.mode = UiMode::Pool;

        let (source_slot_id, created_new) = match choice {
            ArbitrarySourceChoice::ExistingSlot { slot_id } => (slot_id, false),
            ArbitrarySourceChoice::CreateNew => {
                let Some(slot_id) = self.create_random_arbitrary_bytes_slot() else {
                    return;
                };
                (slot_id, true)
            }
        };
        if created_new {
            self.record_production_link(request_slot_id, source_slot_id);
        }
        let result_count = self
            .slot_by_id(source_slot_id)
            .map(|slot| slot.result_slot_ids.len())
            .unwrap_or(0);
        self.invoke_registered_function(source_slot_id, constructor);
        let invocation_succeeded = self
            .slot_by_id(source_slot_id)
            .is_some_and(|slot| slot.result_slot_ids.len() > result_count);
        if invocation_succeeded {
            self.status_message = format!(
                "Used ArbitraryBytes slot {} to produce {} selected from request slot {} ({}).",
                source_slot_id,
                describe_shape(request_function.output_shape),
                request_slot_id,
                describe_function(request_function)
            );
        }
    }

    fn record_production_link(&mut self, producer_slot_id: usize, result_slot_id: usize) {
        if let Some(result_slot) = self.slot_by_id_mut(result_slot_id) {
            result_slot.produced_by_slot_id = Some(producer_slot_id);
        }
        if let Some(producer_slot) = self.slot_by_id_mut(producer_slot_id)
            && !producer_slot.result_slot_ids.contains(&result_slot_id)
        {
            producer_slot.result_slot_ids.push(result_slot_id);
        }
        self.invalidate_all_slot_display_caches();
    }

    fn create_random_arbitrary_bytes_slot(&mut self) -> Option<usize> {
        let choice = self
            .shape_choices
            .iter()
            .find(|shape| shape.label == "ArbitraryBytes")
            .cloned()?;
        let mut raw = vec![0_u8; 4096];
        rand::rng().fill(raw.as_mut_slice());
        let arbitrary_bytes = ArbitraryBytes::new(raw);
        let json = match choice.thing.serialize_boxed(&arbitrary_bytes) {
            Ok(json) => json,
            Err(error) => {
                self.status_message = format!("Could not serialize new ArbitraryBytes: {error}");
                return None;
            }
        };
        let value = match serde_json::from_str(&json) {
            Ok(value) => value,
            Err(error) => {
                self.status_message = format!("Could not parse new ArbitraryBytes JSON: {error}");
                return None;
            }
        };

        let slot_id = self.allocate_slot_id();
        let mut slot = ObjectSlot::new(slot_id);
        slot.apply_shape_choice(&choice);
        slot.value_state = SlotValueState::ResolvedValue { json, value };
        self.object_slots.push(slot);
        self.invalidate_all_slot_display_caches();
        Some(slot_id)
    }

    fn apply_link_action_selection(&mut self) {
        let Some((owner_slot_id, field_index, selected_slot_id, action)) =
            self.link_action_picker.as_ref().and_then(|picker| {
                Some((
                    picker.owner_slot_id,
                    picker.field_index,
                    picker.selected_slot_id,
                    picker.selected_action()?,
                ))
            })
        else {
            self.status_message = "No move/clone action is selected.".to_string();
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
    ) -> Option<usize> {
        let field_name = self
            .slot_field(owner_slot_id, field_index)
            .map(|field| field.info.field_name)?;
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
            return None;
        };

        let slot_id = self.allocate_slot_id();

        let mut slot = ObjectSlot::new(slot_id);
        slot.apply_shape_choice(&choice);
        slot.created_for = Some(SlotCreatedFor {
            owner_slot_id,
            field_index,
            field_name,
        });
        self.object_slots.push(slot);
        self.invalidate_all_slot_display_caches();
        self.status_message = format!(
            "Created source slot {} ({}) to produce {} for slot {}.{}.",
            slot_id, input_shape_name, required_shape_name, owner_slot_id, field_name
        );
        Some(slot_id)
    }

    fn producer_function_for_choice(
        &self,
        input_shape_name: &str,
        required_shape_name: &str,
        function_label: &str,
    ) -> Option<&'static Function> {
        let input_thing = self.thing_for_shape_name(input_shape_name)?;
        let required_thing = self.thing_for_shape_name(required_shape_name)?;
        functions_from(input_thing.shape)
            .into_iter()
            .find(|function| {
                function.production_kind(required_thing.shape) == Some(ProductionKind::Exact)
                    && self.function_picker_label(function) == function_label
            })
    }

    fn invoke_default_producer_for_field(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        input_shape_name: &str,
        required_shape_name: &str,
        function_label: &str,
    ) {
        let Some(function) = self.producer_function_for_choice(
            input_shape_name,
            required_shape_name,
            function_label,
        ) else {
            self.status_message = "The selected producer function is no longer registered.".into();
            return;
        };
        let value = match self.default_json_for_shape(input_shape_name) {
            Ok(value) => value,
            Err(error) => {
                self.status_message =
                    format!("Could not build default {input_shape_name}: {error}");
                return;
            }
        };
        let json = match serde_json::to_string(&value) {
            Ok(json) => json,
            Err(error) => {
                self.status_message =
                    format!("Could not serialize default {input_shape_name}: {error}");
                return;
            }
        };
        let Some(slot_id) = self.create_producer_request_for_field(
            owner_slot_id,
            field_index,
            input_shape_name,
            required_shape_name,
        ) else {
            return;
        };
        if let Some(slot) = self.slot_by_id_mut(slot_id) {
            slot.value_state = SlotValueState::ResolvedValue { json, value };
        }
        self.invalidate_all_slot_display_caches();
        self.invoke_registered_function(slot_id, function);
    }

    fn invoke_arbitrary_producer_for_field(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        input_shape_name: &str,
        required_shape_name: &str,
        function_label: &str,
    ) {
        let Some(function) = self.producer_function_for_choice(
            input_shape_name,
            required_shape_name,
            function_label,
        ) else {
            self.status_message = "The selected producer function is no longer registered.".into();
            return;
        };
        let Some(slot_id) = self.create_producer_request_for_field(
            owner_slot_id,
            field_index,
            input_shape_name,
            required_shape_name,
        ) else {
            return;
        };
        self.invoke_arbitrary_registered_function(slot_id, function);
    }

    fn default_json_for_shape(&self, shape_name: &str) -> Result<Value> {
        let thing = self
            .thing_for_shape_name(shape_name)
            .ok_or_else(|| eyre::eyre!("{shape_name} is not registered"))?;
        let variants = shape_variants_for_thing(thing);
        if !variants.is_empty() {
            let variant = variants
                .iter()
                .find(|variant| variant.is_default)
                .ok_or_else(|| eyre::eyre!("{shape_name} has no Default variant"))?;
            if !variant.payload_fields.is_empty() {
                eyre::bail!("{}::Default requires a payload", shape_name);
            }
            return Ok(Value::String(variant.variant_name.to_string()));
        }

        let fields = shape_fields_for_thing(thing);
        if fields.is_empty() {
            if thing
                .shape
                .type_ops
                .is_some_and(|type_ops| type_ops.has_default_in_place())
            {
                return Ok(Value::Object(Map::new()));
            }
            eyre::bail!("{shape_name} has no reflected default construction");
        }
        let mut object = Map::new();
        for field in fields {
            let value = if reflected_field_default_is_materializable(&field) {
                self.default_json_value(
                    &field.field_shape_name,
                    field.default_value_label.as_deref(),
                )?
            } else {
                self.default_json_for_shape(&field.field_shape_name)?
            };
            object.insert(field.field_name.to_string(), value);
        }
        Ok(Value::Object(object))
    }

    fn create_general_value_slot(&mut self, owner_slot_id: usize, field_index: usize) {
        let Some(field) = self.slot_field(owner_slot_id, field_index).cloned() else {
            return;
        };
        let Some(shape) = self.field_shape_for_field(owner_slot_id, field_index) else {
            self.status_message =
                format!("Could not reflect the shape of {}.", field.info.field_name);
            return;
        };
        let slot_id = self.allocate_slot_id();
        let slot = ObjectSlot::new_scalar(slot_id, describe_shape(shape), shape);
        self.object_slots.push(slot);
        self.set_field_link(owner_slot_id, field_index, slot_id);
        self.invalidate_all_slot_display_caches();
        self.status_message = format!(
            "Created owned value slot {} for {}. Choose or enter its value.",
            slot_id, field.info.field_name
        );
        self.open_slot_value_editor(slot_id);
    }
    fn promote_general_value_slot(&mut self, slot_id: usize) -> bool {
        let Some(shape) = self.scalar_shape_for_slot(slot_id) else {
            return false;
        };
        if !is_general_value_shape(shape) {
            return false;
        }
        let Some(data_slot_id) = self.data_slot_id_for(slot_id) else {
            return false;
        };
        let Some(slot) = self.slot_by_id_mut(data_slot_id) else {
            return false;
        };
        match &slot.value_state {
            SlotValueState::Building(SlotBody::Value { .. }) => true,
            SlotValueState::Building(SlotBody::Struct { fields })
                if fields.len() == 1 && matches!(fields[0].value_state, FieldValueState::Unset) =>
            {
                slot.value_state = SlotValueState::Building(SlotBody::Value { shape, value: None });
                true
            }
            _ => false,
        }
    }

    fn open_slot_value_editor(&mut self, slot_id: usize) {
        let Some(shape) = self.scalar_shape_for_slot(slot_id) else {
            self.status_message = format!(
                "Slot {} does not contain an editable scalar value.",
                slot_id
            );
            return;
        };
        let shape_name = self.slot_shape_name(slot_id).unwrap_or("value").to_string();
        if shape.scalar_type() == Some(ScalarType::Bool) {
            self.open_boolean_value_picker(slot_id, shape_name);
            return;
        }
        let value = self
            .slot_by_id(self.data_slot_id_for(slot_id).unwrap_or(slot_id))
            .and_then(|slot| match &slot.value_state {
                SlotValueState::Building(SlotBody::Value { value, .. }) => value.clone(),
                SlotValueState::ResolvedValue { value, .. } => Some(value.clone()),
                _ => None,
            });
        self.general_value_editor = Some(GeneralValueEditorState::new(
            slot_id,
            shape_name,
            shape,
            value.as_ref().map(scalar_value_input).unwrap_or_default(),
        ));
        self.mode = UiMode::GeneralValueEditor;
        self.status_message = format!("Enter a value for slot {}.", slot_id);
    }
    fn open_boolean_value_picker(&mut self, slot_id: usize, shape_name: String) {
        let selected = self
            .slot_by_id(self.data_slot_id_for(slot_id).unwrap_or(slot_id))
            .and_then(|slot| match &slot.value_state {
                SlotValueState::Building(SlotBody::Value { value, .. }) => {
                    value.as_ref().and_then(Value::as_bool)
                }
                SlotValueState::ResolvedValue { value, .. } => value.as_bool(),
                _ => None,
            })
            .map(|value| usize::from(value));
        self.boolean_value_picker =
            Some(BooleanValuePickerState::new(slot_id, shape_name, selected));
        self.mode = UiMode::BooleanValuePicker;
        self.status_message = format!("Choose true or false for slot {}.", slot_id);
    }
    fn scalar_shape_for_slot(&self, slot_id: usize) -> Option<&'static facet::Shape> {
        let data_slot = self.slot_by_id(self.data_slot_id_for(slot_id).unwrap_or(slot_id))?;
        if let SlotValueState::Building(SlotBody::Value { shape, .. }) = &data_slot.value_state {
            return Some(*shape);
        }
        self.slot_shape_name(slot_id)
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))
    }
    fn parse_general_value(
        &self,
        shape_name: &str,
        shape: &'static facet::Shape,
        input: &str,
    ) -> Result<Value> {
        let scalar = shape.scalar_type();
        if shape_name == "Uuid" {
            let parsed = input
                .parse::<cloud_terrastodon_azure::uuid::Uuid>()
                .map_err(|error| eyre::eyre!("{error}"))?;
            return Ok(Value::String(parsed.to_string()));
        }
        let thing = self.thing_for_shape_name(shape_name);
        let expects_json_string = shape_name == "Uuid"
            || matches!(
                scalar,
                Some(ScalarType::String | ScalarType::Str | ScalarType::Char)
            )
            || shape_accepts_text_input(shape);
        let json_input = if expects_json_string {
            serde_json::to_string(input)?
        } else {
            input.to_string()
        };
        if let Some(thing) = thing {
            let parsed = thing
                .parse_boxed(&json_input)
                .map_err(|error| eyre::eyre!("{error}"))?;
            let serialized = thing.serialize_boxed(parsed.as_ref())?;
            return Ok(serde_json::from_str(&serialized)?);
        }
        let value: Value = serde_json::from_str(&json_input)?;
        match scalar {
            Some(ScalarType::Bool) if value.is_boolean() => Ok(value),
            Some(ScalarType::String | ScalarType::Str) if value.is_string() => Ok(value),
            Some(ScalarType::Char)
                if value.as_str().is_some_and(|text| text.chars().count() == 1) =>
            {
                Ok(value)
            }
            Some(ScalarType::F32 | ScalarType::F64) if value.is_number() => Ok(value),
            Some(
                ScalarType::U8
                | ScalarType::U16
                | ScalarType::U32
                | ScalarType::U64
                | ScalarType::USize,
            ) if value.as_u64().is_some() => Ok(value),
            Some(
                ScalarType::I8
                | ScalarType::I16
                | ScalarType::I32
                | ScalarType::I64
                | ScalarType::ISize,
            ) if value.as_i64().is_some() => Ok(value),
            _ => eyre::bail!("{input:?} is not a valid {shape_name}"),
        }
    }
    fn set_scalar_slot_value(&mut self, slot_id: usize, value: Value) {
        let json = match serde_json::to_string(&value) {
            Ok(json) => json,
            Err(error) => {
                self.status_message = format!("Could not serialize slot {}: {}", slot_id, error);
                return;
            }
        };
        let data_slot_id = self.data_slot_id_for(slot_id).unwrap_or(slot_id);
        if let Some(slot) = self.slot_by_id_mut(data_slot_id) {
            match &mut slot.value_state {
                SlotValueState::Building(SlotBody::Value { value: current, .. }) => {
                    *current = Some(value.clone())
                }
                SlotValueState::ResolvedValue {
                    json: current_json,
                    value: current,
                    ..
                } => {
                    *current_json = json;
                    *current = value.clone();
                }
                _ => {
                    self.status_message =
                        format!("Slot {} is not an editable scalar slot.", slot_id);
                    return;
                }
            }
        }
        self.invalidate_all_slot_display_caches();
    }
    fn view_slot_id_for_field(&self, owner_slot_id: usize, field_index: usize) -> Option<usize> {
        self.object_slots.iter().find_map(|slot| match &slot.kind {
            SlotKind::View(info)
                if info.owner_slot_id == owner_slot_id && info.field_index == field_index =>
            {
                Some(slot.id)
            }
            _ => None,
        })
    }
    fn create_field_object(
        &mut self,
        owner_slot_id: usize,
        field_index: usize,
        required_shape_name: &str,
        selected_variant: Option<usize>,
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
        if let Some(variant_index) = selected_variant
            && slot.select_variant(variant_index).is_none()
        {
            self.status_message =
                format!("Could not select variant {variant_index} for {required_shape_name}.");
            return;
        }
        slot.kind = SlotKind::View(ViewInfo {
            source_slot_id: slot_id,
            owner_slot_id,
            field_index,
            field_name,
        });

        self.object_slots.push(slot);
        self.set_field_link(owner_slot_id, field_index, slot_id);
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
        self.jump_to_slot_target(owner_slot_id, SlotFocusTarget::FieldValue(field_index));
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
        self.jump_to_slot_target(owner_slot_id, SlotFocusTarget::FieldValue(field_index));
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
            let Some(field) = self
                .materialized_fields(owner_slot_id)
                .get(field_index)
                .cloned()
            else {
                return;
            };
            if let Some(slot_id) = self.view_slot_id_for_field(owner_slot_id, field_index) {
                self.jump_to_slot(slot_id);
                self.status_message = format!(
                    "Jumped to slot {} for {} on slot {}.",
                    slot_id, field.info.field_name, owner_slot_id
                );
                return;
            }
            self.activate_json_projection(owner_slot_id, field.projection_path);
            return;
        };

        if self.is_general_value_field(owner_slot_id, field_index) {
            if let FieldValueState::Linked { slot_id } = field.value_state {
                self.jump_to_slot(slot_id);
                self.status_message = format!(
                    "Jumped to slot {} for {} on slot {}.",
                    slot_id, field.info.field_name, owner_slot_id
                );
            } else {
                let wrapper_slot = self
                    .slot_shape_name(owner_slot_id)
                    .and_then(|shape_name| self.shape_for_shape_name(shape_name))
                    .is_some_and(is_general_value_shape);
                if field_index == 0
                    && wrapper_slot
                    && self.promote_general_value_slot(owner_slot_id)
                {
                    self.open_slot_value_editor(owner_slot_id);
                } else {
                    self.open_general_field_picker(owner_slot_id, field_index, &field);
                }
            }
            return;
        }
        match field.value_state {
            FieldValueState::Linked { slot_id } => {
                self.jump_to_slot(slot_id);
                self.status_message = format!(
                    "Jumped to slot {} for {} on slot {}.",
                    slot_id, field.info.field_name, owner_slot_id
                );
            }
            FieldValueState::Defaulted | FieldValueState::Unset
                if self.has_known_shape_label(&self.field_shape_name_for_matching(
                    owner_slot_id,
                    field_index,
                    &field,
                )) =>
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
                    "{} is required and has no registered creation path.",
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
        let field_info = self
            .slot_field(slot_id, field_index)
            .map(|field| field.info.clone())
            .or_else(|| {
                self.materialized_fields(slot_id)
                    .get(field_index)
                    .map(|field| field.info.clone())
            });
        let Some(field_info) = field_info else { return };

        self.status_message = format!(
            "{} has type {}. Type-scoped actions like browsing matching objects or producers are the next interaction to add.",
            field_info.field_name, field_info.field_shape_name
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
            let SlotValueState::Building(body) = &mut slot.value_state else {
                continue;
            };
            match body {
                SlotBody::Value { .. } | SlotBody::Unset => {}
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
        let value_state = match &data_slot.value_state {
            SlotValueState::Building(body) => SlotSnapshotValueState::Building(body.clone()),
            SlotValueState::ResolvedValue { json, value } => {
                SlotSnapshotValueState::ResolvedValue {
                    json: json.clone(),
                    value: value.clone(),
                }
            }
            SlotValueState::Pending(_) | SlotValueState::Failed { .. } => return None,
        };
        Some(SlotSnapshot {
            name: display_slot.name.clone().or_else(|| data_slot.name.clone()),
            shape_name: data_slot.shape_name.clone(),
            value_state,
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
                && let SlotValueState::Pending(pending) = &slot.value_state
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
        if let Some(view) = self.current_projection_view()
            && !self.breadcrumb_filters.is_empty()
        {
            return usize::from(
                self.projection_path_matches_shape_filter(view.root_slot_id, &view.path),
            ) + self.filtered_projection_descendant_count(view.root_slot_id, &view.path);
        }
        if !self.breadcrumb_filters.is_empty() {
            return self
                .object_slots
                .iter()
                .map(|slot| {
                    usize::from(self.real_slot_matches_shape_filter(slot.id))
                        + self.filtered_projection_descendant_count(slot.id, &[])
                })
                .sum::<usize>()
                + 1;
        }
        self.unfiltered_total_slot_count()
    }

    fn unfiltered_total_slot_count(&self) -> usize {
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
            .and_then(ObjectSlot::building_body)
    }

    fn materialized_fields(&self, slot_id: usize) -> Vec<MaterializedFieldState> {
        let Some(data_slot) = self.slot_by_id(self.data_slot_id_for(slot_id).unwrap_or(slot_id))
        else {
            return Vec::new();
        };
        let SlotValueState::ResolvedValue { value, .. } = &data_slot.value_state else {
            return Vec::new();
        };
        let Some(shape_name) = data_slot.shape_name.as_deref() else {
            return Vec::new();
        };
        let Some(thing) = self.thing_for_shape_name(shape_name) else {
            return Vec::new();
        };
        let fields = shape_fields_for_thing(thing);
        let transparent_value =
            (thing.shape.is_transparent() && fields.len() == 1).then_some(value);

        fields
            .into_iter()
            .filter_map(|info| {
                let (value, projection_path) = match value {
                    Value::Object(object) => (
                        object.get(info.field_name)?.clone(),
                        vec![JsonPathSegment::Field(info.field_name.to_string())],
                    ),
                    _ => (transparent_value?.clone(), Vec::new()),
                };
                Some(MaterializedFieldState {
                    info,
                    value,
                    projection_path,
                })
            })
            .collect()
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
        if !self.breadcrumb_filters.is_empty() {
            return (0..self.total_slot_count()).find(|index| {
                matches!(self.pool_entry_at(*index), Some(PoolEntry::RealSlot(id)) if id == slot_id)
            });
        }
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

    fn shape_for_shape_name(&self, shape_name: &str) -> Option<&'static facet::Shape> {
        self.reflected_shapes.get(shape_name).copied()
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

        match &slot.value_state {
            SlotValueState::Pending(_) => {
                eyre::bail!("slot {} is still pending", slot.id);
            }
            SlotValueState::Failed { message } => {
                eyre::bail!("slot {} failed: {}", slot.id, message);
            }
            SlotValueState::ResolvedValue { value, .. } => {
                return Ok(value.clone());
            }
            SlotValueState::Building(_) => {}
        }

        let SlotValueState::Building(body) = &slot.value_state else {
            unreachable!("non-building states returned above")
        };
        let uses_proxy = slot
            .shape_name
            .as_deref()
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))
            .is_some_and(|shape| shape.proxy.is_some());
        match body {
            SlotBody::Value {
                value: Some(value), ..
            } => Ok(value.clone()),
            SlotBody::Value { value: None, .. } => eyre::bail!("slot {} is still unset", slot.id),
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
                if uses_proxy && fields.len() == 1 {
                    return Ok(payload);
                }

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

        rows.push(SlotDisplayRow::Static(separator_line("shape")));
        rows.push(shape_row(self.slot_shape_name(slot_id)));
        for output_shape_name in self.request_output_shape_names(slot_id) {
            rows.push(SlotDisplayRow::Static(Line::from(vec![
                Span::styled(
                    "produces ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(output_shape_name, Style::default().fg(Color::Cyan)),
            ])));
        }

        let materialized_fields = self.materialized_fields(slot_id);
        match self.slot_body(slot_id) {
            Some(SlotBody::Value { value, .. }) => {
                rows.push(SlotDisplayRow::Static(separator_line("value")));
                rows.push(focusable_spans_row(
                    SlotFocusTarget::RuntimeValue,
                    vec![
                        Span::raw("value: "),
                        Span::styled(
                            value
                                .as_ref()
                                .map_or_else(|| "unset".to_string(), json_value_summary),
                            value.as_ref().map_or_else(unset_style, |_| {
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD)
                            }),
                        ),
                    ],
                ));
            }
            Some(SlotBody::Unset) => {}
            None => {
                if !materialized_fields.is_empty() {
                    rows.push(SlotDisplayRow::Static(separator_line("fields")));
                    rows.extend(materialized_field_rows(&materialized_fields));
                }
            }
            Some(SlotBody::Struct { fields }) => {
                if !fields.is_empty() {
                    rows.push(SlotDisplayRow::Static(separator_line("fields")));
                }
                rows.extend(self.field_rows(fields));
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
                rows.extend(self.field_rows(fields));
            }
        }

        if let Some(runtime_state) = self.slot_runtime_state(slot_id) {
            rows.push(SlotDisplayRow::Static(separator_line("status")));
            rows.extend(runtime_state_rows(runtime_state));
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

    fn request_output_shape_names(&self, slot_id: usize) -> Vec<String> {
        let Some(shape_name) = self.slot_shape_name(slot_id) else {
            return Vec::new();
        };
        let Some(thing) = self.thing_for_shape_name(shape_name) else {
            return Vec::new();
        };

        functions_from(thing.shape)
            .into_iter()
            .filter(|function| function.kind == FunctionKind::AsyncInvoke)
            .map(|function| describe_shape(function.output_shape))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
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
                SlotBody::Value { .. } => targets.push(SlotFocusTarget::RuntimeValue),
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
        } else {
            for index in 0..self.materialized_fields(slot_id).len() {
                targets.push(SlotFocusTarget::FieldType(index));
                targets.push(SlotFocusTarget::FieldValue(index));
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
            Some(SlotValueState::ResolvedValue { .. })
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
            let Some(body) = slot.building_body() else {
                continue;
            };
            let fields = match body {
                SlotBody::Value { .. } | SlotBody::Unset => continue,
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
            if function.kind == FunctionKind::AsyncInvoke {
                if self.shape_supports_default(&input_shape_name) {
                    choices.push(FieldPickerChoice::InvokeDefaultProducer {
                        input_shape_name: input_shape_name.clone(),
                        function_label: function_label.clone(),
                    });
                }
            }
            choices.push(FieldPickerChoice::CreateProducer {
                input_shape_name: input_shape_name.clone(),
                function_label: function_label.clone(),
            });
            if function.kind == FunctionKind::AsyncInvoke {
                if functions_to(function.output_shape)
                    .into_iter()
                    .any(|candidate| describe_shape(candidate.input_shape) == "ArbitraryBytes")
                {
                    choices.push(FieldPickerChoice::InvokeArbitraryProducer {
                        input_shape_name,
                        function_label,
                    });
                }
            }
        }
        choices
    }

    fn shape_supports_default(&self, shape_name: &str) -> bool {
        self.shape_supports_default_inner(shape_name, &mut BTreeSet::new())
    }

    fn shape_supports_default_inner(
        &self,
        shape_name: &str,
        visiting: &mut BTreeSet<String>,
    ) -> bool {
        if !visiting.insert(shape_name.to_string()) {
            return false;
        }
        let result = self.thing_for_shape_name(shape_name).is_some_and(|thing| {
            let variants = shape_variants_for_thing(thing);
            if !variants.is_empty() {
                return variants.iter().any(|variant| {
                    variant.is_default
                        && variant.payload_fields.iter().all(|field| {
                            reflected_field_default_is_materializable(field)
                                || self
                                    .shape_supports_default_inner(&field.field_shape_name, visiting)
                        })
                });
            }
            let fields = shape_fields_for_thing(thing);
            if fields.is_empty() {
                thing
                    .shape
                    .type_ops
                    .is_some_and(|type_ops| type_ops.has_default_in_place())
            } else {
                fields.iter().all(|field| {
                    reflected_field_default_is_materializable(field)
                        || self.shape_supports_default_inner(&field.field_shape_name, visiting)
                })
            }
        });
        visiting.remove(shape_name);
        result
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
            } => format!(
                "+ create {} for {} via {}",
                input_shape_name, required_shape_name, function_label
            ),
            FieldPickerChoice::CreateNewValue => {
                format!("+ create new owned {required_shape_name}")
            }
            FieldPickerChoice::CreateNew => {
                let is_enum = self
                    .thing_for_shape_name(required_shape_name)
                    .is_some_and(|thing| !shape_variants_for_thing(thing).is_empty());
                if is_enum {
                    format!("+ create {required_shape_name} with variant unset")
                } else {
                    format!("+ create new {required_shape_name}")
                }
            }
            FieldPickerChoice::InvokeDefaultProducer {
                input_shape_name, ..
            } => format!("+ invoke default {input_shape_name} for {required_shape_name}"),
            FieldPickerChoice::InvokeArbitraryProducer {
                input_shape_name, ..
            } => format!("+ invoke arbitrary {input_shape_name} for {required_shape_name}"),
            FieldPickerChoice::CreateNewVariant { variant_name, .. } => {
                format!("+ create {required_shape_name}::{variant_name}")
            }
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
            .chain(self.breadcrumb_filters.iter().map(|filter| match filter {
                BreadcrumbFilter::Shape(filter) => {
                    format!("shapes ({})", filter.included_shapes.len())
                }
                BreadcrumbFilter::SlotKind(filter) => {
                    format!("slots ({})", filter.included_kinds.len(),)
                }
                BreadcrumbFilter::Value(filter) => format!(
                    "value {}.{} {} {}",
                    filter.field_shape,
                    filter.field_name,
                    filter.operator.label(),
                    filter.value
                ),
            }))
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
        2 + self.projection_stack.len() + self.breadcrumb_filters.len()
    }

    fn move_breadcrumb_left(&mut self) {
        self.active_breadcrumb_index = self.active_breadcrumb_index.saturating_sub(1);
    }

    fn move_breadcrumb_right(&mut self) {
        let max_index = self.breadcrumb_count().saturating_sub(1);
        self.active_breadcrumb_index = (self.active_breadcrumb_index + 1).min(max_index);
    }

    fn delete_current_breadcrumb(&mut self) {
        let index = self.active_breadcrumb_index;
        if index == 0 || index + 1 == self.breadcrumb_count() {
            return;
        }
        let projection_end = self.projection_stack.len();
        if index <= projection_end {
            self.projection_stack.truncate(index.saturating_sub(1));
            self.breadcrumb_filters.clear();
        } else {
            let filter_index = index - projection_end - 1;
            if filter_index < self.breadcrumb_filters.len() {
                self.breadcrumb_filters.remove(filter_index);
            }
        }
        self.projection_cache.borrow_mut().clear();
        self.active_breadcrumb_index = index
            .saturating_sub(1)
            .min(self.breadcrumb_count().saturating_sub(1));
        self.active_slot_index = 0;
        self.active_row_index = 0;
        self.sync_selection_viewports();
        self.status_message = "Breadcrumb removed.".to_string();
    }

    fn activate_current_breadcrumb(&mut self) {
        let filter_start = self.projection_stack.len() + 1;
        let add_index = filter_start + self.breadcrumb_filters.len();
        if self.active_breadcrumb_index == add_index {
            self.open_filter_kind_picker();
            return;
        }

        if self.active_breadcrumb_index == 0 {
            self.projection_stack.clear();
            self.breadcrumb_filters.clear();
            self.projection_cache.borrow_mut().clear();
            self.active_slot_index = self
                .active_slot_index
                .min(self.total_slot_count().saturating_sub(1));
            self.active_row_index = 0;
            self.pool_surface = PoolSurface::Slots;
            self.sync_selection_viewports();
            self.status_message = "Returned to the full object pool.".to_string();
            return;
        }

        if self.active_breadcrumb_index >= filter_start {
            let filter_index = self.active_breadcrumb_index - filter_start;
            match self.breadcrumb_filters.get(filter_index) {
                Some(BreadcrumbFilter::Shape(_)) => {
                    self.open_shape_filter_picker(Some(filter_index));
                }
                Some(BreadcrumbFilter::Value(filter)) => {
                    self.open_value_filter_editor(Some(filter_index), filter.clone());
                }
                Some(BreadcrumbFilter::SlotKind(_)) => {
                    self.open_slot_kind_filter_picker(Some(filter_index));
                }
                None => {}
            }
            return;
        }
        self.projection_stack.truncate(self.active_breadcrumb_index);
        self.breadcrumb_filters.clear();
        self.projection_cache.borrow_mut().clear();
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
        if let Some(filter) = self.breadcrumb_filters.pop() {
            self.projection_cache.borrow_mut().clear();
            self.pool_surface = PoolSurface::Slots;
            self.active_breadcrumb_index =
                self.projection_stack.len() + self.breadcrumb_filters.len();
            self.active_slot_index = 0;
            self.active_row_index = 0;
            self.sync_selection_viewports();
            self.status_message = match filter {
                BreadcrumbFilter::Shape(_) => "Closed shape filter.".to_string(),
                BreadcrumbFilter::Value(_) => "Closed value filter.".to_string(),
                BreadcrumbFilter::SlotKind(_) => "Closed slot kind filter.".to_string(),
            };
            return;
        }
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

    fn open_filter_kind_picker(&mut self) {
        self.filter_kind_picker = Some(FilterKindPickerState::new());
        self.mode = UiMode::FilterKindPicker;
        self.status_message = "Choose the kind of breadcrumb filter.".to_string();
    }

    fn open_value_filter_editor(&mut self, edit_index: Option<usize>, filter: ValueFilterView) {
        self.value_filter_editor = Some(ValueFilterEditorState::new(edit_index, filter));
        self.mode = UiMode::ValueFilterEditor;
        self.status_message = "Configure the value filter, then save it.".to_string();
    }

    fn available_projection_fields(&self) -> BTreeSet<(String, String)> {
        let mut fields = BTreeSet::new();
        if let Some(view) = self.current_projection_view() {
            if let Some(mut shape) = self
                .slot_shape_name(view.root_slot_id)
                .and_then(|shape_name| self.shape_for_shape_name(shape_name))
            {
                for segment in &view.path {
                    let Some(next) = (match segment {
                        JsonPathSegment::Field(name) => shape_field_shape(shape, name),
                        JsonPathSegment::Index(_) => sequence_element_shape(shape),
                        JsonPathSegment::Key(_) => registry_map_value_shape(shape),
                    }) else {
                        return fields;
                    };
                    shape = next;
                }
                fields.extend(projection_fields(shape));
            }
        } else {
            for slot in &self.object_slots {
                if let Some(shape) = self
                    .slot_shape_name(slot.id)
                    .and_then(|shape_name| self.shape_for_shape_name(shape_name))
                {
                    fields.extend(projection_fields(shape));
                }
            }
        }
        fields
    }

    fn open_value_filter_choice(&mut self, target: ValueFilterChoiceTarget) {
        let Some(editor) = self.value_filter_editor.as_ref() else {
            return;
        };
        let labels = match target {
            ValueFilterChoiceTarget::FieldShape => {
                let mut labels = self
                    .available_projection_fields()
                    .into_iter()
                    .map(|(shape, _)| shape)
                    .collect::<BTreeSet<_>>();
                labels.insert("*".to_string());
                labels.into_iter().collect()
            }
            ValueFilterChoiceTarget::FieldName => {
                let selected_shape = editor.draft.field_shape.clone();
                let mut labels = self
                    .available_projection_fields()
                    .into_iter()
                    .filter_map(|(shape, name)| {
                        (selected_shape == "*" || selected_shape == shape).then_some(name)
                    })
                    .collect::<BTreeSet<_>>();
                labels.insert("*".to_string());
                labels.into_iter().collect()
            }
            ValueFilterChoiceTarget::Operator => vec![
                "equals".to_string(),
                "not equals".to_string(),
                "contains".to_string(),
            ],
            ValueFilterChoiceTarget::ExistingValue => self
                .existing_value_filter_choices(&editor.draft.field_shape, &editor.draft.field_name),
        };
        if labels.is_empty() {
            self.status_message = "No matching existing values were found.".to_string();
            return;
        }
        self.value_filter_choice_picker = Some(ValueFilterChoicePickerState::new(target, labels));
        self.mode = UiMode::ValueFilterChoicePicker;
    }

    fn existing_value_filter_choices(&self, field_shape: &str, field_name: &str) -> Vec<String> {
        let mut values = BTreeSet::new();
        if let Some(view) = self.current_projection_view() {
            if let Some(value) = self.json_value_at_path(view.root_slot_id, &view.path) {
                self.collect_existing_filter_values(
                    view.root_slot_id,
                    &view.path,
                    value,
                    field_shape,
                    field_name,
                    &mut values,
                );
            }
        } else {
            for slot in &self.object_slots {
                if let Some(value) = self.json_value_at_path(slot.id, &[]) {
                    self.collect_existing_filter_values(
                        slot.id,
                        &[],
                        value,
                        field_shape,
                        field_name,
                        &mut values,
                    );
                }
            }
        }
        values.into_iter().collect()
    }

    fn collect_existing_filter_values(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
        value: &Value,
        field_shape: &str,
        field_name: &str,
        values: &mut BTreeSet<String>,
    ) {
        let shape_matches = field_shape == "*"
            || self
                .projection_shape_name_at_path(root_slot_id, path)
                .is_some_and(|shape| shape == field_shape);
        let name_matches = field_name == "*"
            || path.last().is_some_and(|segment| match segment {
                JsonPathSegment::Field(name) | JsonPathSegment::Key(name) => name == field_name,
                JsonPathSegment::Index(_) => false,
            });
        if shape_matches && name_matches && !matches!(value, Value::Array(_) | Value::Object(_)) {
            values.insert(match value {
                Value::String(value) => value.clone(),
                _ => value.to_string(),
            });
        }
        match value {
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    let child_path = append_json_path_segment(path, JsonPathSegment::Index(index));
                    self.collect_existing_filter_values(
                        root_slot_id,
                        &child_path,
                        child,
                        field_shape,
                        field_name,
                        values,
                    );
                }
            }
            Value::Object(object) => {
                let is_map = self.projection_path_is_map(root_slot_id, path);
                for (name, child) in object {
                    let segment = if is_map {
                        JsonPathSegment::Key(name.clone())
                    } else {
                        JsonPathSegment::Field(name.clone())
                    };
                    let child_path = append_json_path_segment(path, segment);
                    self.collect_existing_filter_values(
                        root_slot_id,
                        &child_path,
                        child,
                        field_shape,
                        field_name,
                        values,
                    );
                }
            }
            _ => {}
        }
    }

    fn apply_value_filter_editor(&mut self) {
        let Some(mut editor) = self.value_filter_editor.take() else {
            return;
        };
        if editor.source == ValueFilterSource::Literal {
            editor.draft.value = editor
                .literal_input
                .lines()
                .first()
                .cloned()
                .unwrap_or_default();
        }
        let filter = BreadcrumbFilter::Value(editor.draft);
        if let Some(index) = editor.edit_index {
            if let Some(existing) = self.breadcrumb_filters.get_mut(index) {
                *existing = filter;
            }
        } else {
            self.breadcrumb_filters.push(filter);
        }
        self.projection_cache.borrow_mut().clear();
        self.mode = UiMode::Pool;
        self.pool_surface = PoolSurface::Breadcrumbs;
        self.active_breadcrumb_index = self.projection_stack.len() + self.breadcrumb_filters.len();
        self.active_slot_index = 0;
        self.active_row_index = 0;
        self.sync_selection_viewports();
        self.status_message = format!(
            "Value filter applied; {} entries visible.",
            self.total_slot_count()
        );
    }

    fn open_shape_filter_picker(&mut self, edit_index: Option<usize>) {
        let mut labels = BTreeSet::new();
        if let Some(view) = self.current_projection_view() {
            labels.extend(self.projection_shape_names_at_path(view.root_slot_id, &view.path));
        } else {
            for slot in &self.object_slots {
                if let Some(shape) = self
                    .slot_shape_name(slot.id)
                    .and_then(|shape_name| self.shape_for_shape_name(shape_name))
                {
                    labels.extend(projection_shape_names(shape));
                }
            }
        }
        for (index, filter) in self.breadcrumb_filters.iter().enumerate() {
            if Some(index) == edit_index {
                continue;
            }
            if let BreadcrumbFilter::Shape(filter) = filter {
                labels.retain(|label| filter.included_shapes.contains(label));
            }
        }
        let labels = labels.into_iter().collect::<Vec<_>>();
        if labels.is_empty() {
            self.status_message = "The current view has no shapes to filter.".to_string();
            return;
        }
        let included = edit_index
            .and_then(|index| self.breadcrumb_filters.get(index))
            .and_then(|filter| match filter {
                BreadcrumbFilter::Shape(filter) => Some(filter.included_shapes.clone()),
                BreadcrumbFilter::Value(_) | BreadcrumbFilter::SlotKind(_) => None,
            })
            .unwrap_or_default();
        self.partition_picker = Some(PartitionPickerState::with_included_labels(
            PartitionPickerTarget::ShapeFilter { edit_index },
            labels,
            &included,
        ));
        self.mode = UiMode::PartitionPicker;
        self.pool_surface = PoolSurface::Slots;
        self.status_message = "Move shapes to Included and press Enter.".to_string();
    }

    fn open_slot_kind_filter_picker(&mut self, edit_index: Option<usize>) {
        let labels = vec![
            SlotFilterKind::Owned.label().to_string(),
            SlotFilterKind::View.label().to_string(),
            SlotFilterKind::Projection.label().to_string(),
        ];
        let included = edit_index
            .and_then(|index| self.breadcrumb_filters.get(index))
            .and_then(|filter| match filter {
                BreadcrumbFilter::SlotKind(filter) => Some(
                    filter
                        .included_kinds
                        .iter()
                        .map(|kind| kind.label().to_string())
                        .collect::<BTreeSet<_>>(),
                ),
                BreadcrumbFilter::Shape(_) | BreadcrumbFilter::Value(_) => None,
            })
            .unwrap_or_default();
        self.partition_picker = Some(PartitionPickerState::with_included_labels(
            PartitionPickerTarget::SlotKindFilter { edit_index },
            labels,
            &included,
        ));
        self.mode = UiMode::PartitionPicker;
        self.pool_surface = PoolSurface::Slots;
        self.status_message = "Move slot kinds to Included and press Enter.".to_string();
    }

    fn apply_partition_picker_selection(&mut self) {
        let Some(picker) = self.partition_picker.take() else {
            return;
        };
        let filter_label = match picker.target {
            PartitionPickerTarget::ShapeFilter { .. } => "Shape",
            PartitionPickerTarget::SlotKindFilter { .. } => "Slot kind",
        };
        let focused_filter_index = match picker.target {
            PartitionPickerTarget::ShapeFilter { edit_index } => {
                let filter = BreadcrumbFilter::Shape(ShapeFilterView {
                    included_shapes: picker.included_labels(),
                });
                if let Some(index) = edit_index {
                    if let Some(existing) = self.breadcrumb_filters.get_mut(index) {
                        *existing = filter;
                    }
                    Some(index)
                } else {
                    self.breadcrumb_filters.push(filter);
                    self.breadcrumb_filters.len().checked_sub(1)
                }
            }
            PartitionPickerTarget::SlotKindFilter { edit_index } => {
                let included_kinds = picker
                    .included_labels()
                    .into_iter()
                    .filter_map(|label| SlotFilterKind::from_label(&label))
                    .collect();
                let filter = BreadcrumbFilter::SlotKind(SlotKindFilterView { included_kinds });
                if let Some(index) = edit_index {
                    if let Some(existing) = self.breadcrumb_filters.get_mut(index) {
                        *existing = filter;
                    }
                    Some(index)
                } else {
                    self.breadcrumb_filters.push(filter);
                    self.breadcrumb_filters.len().checked_sub(1)
                }
            }
        };
        self.projection_cache.borrow_mut().clear();
        self.mode = UiMode::Pool;
        self.pool_surface = PoolSurface::Breadcrumbs;
        self.active_breadcrumb_index = self.projection_stack.len()
            + 1
            + focused_filter_index.unwrap_or(self.breadcrumb_filters.len());
        self.active_slot_index = 0;
        self.active_row_index = 0;
        self.slot_view_offset = 0;
        self.row_view_offset = 0;
        self.sync_selection_viewports();
        self.status_message = format!(
            "{filter_label} filter applied; {} entries visible.",
            self.total_slot_count()
        );
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

    fn filtered_projection_descendant_count(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
    ) -> usize {
        if self.breadcrumb_filters.is_empty() {
            return self.projection_descendant_count(root_slot_id, parent_path);
        }
        let cache_key = ProjectionCacheKey::new(root_slot_id, parent_path);
        if let Some(cached_count) = self
            .projection_cache
            .borrow()
            .filtered_descendant_counts
            .get(&cache_key)
            .copied()
        {
            return cached_count;
        }
        if !self.projection_filter_relation(root_slot_id, parent_path).1 {
            self.projection_cache
                .borrow_mut()
                .filtered_descendant_counts
                .insert(cache_key, 0);
            return 0;
        }

        let Some(value) = self.json_value_at_path(root_slot_id, parent_path) else {
            return 0;
        };
        let descendant_count = match value {
            Value::Array(items) => {
                let sample_path = append_json_path_segment(parent_path, JsonPathSegment::Index(0));
                let (child_matches, child_has_matches_below) =
                    self.projection_filter_relation(root_slot_id, &sample_path);
                if !child_has_matches_below && !self.has_value_filters() {
                    items.len() * usize::from(child_matches)
                } else {
                    (0..items.len())
                        .map(|index| {
                            let path = append_json_path_segment(
                                parent_path,
                                JsonPathSegment::Index(index),
                            );
                            usize::from(
                                self.projection_path_matches_shape_filter(root_slot_id, &path),
                            ) + self.filtered_projection_descendant_count(root_slot_id, &path)
                        })
                        .sum()
                }
            }
            Value::Object(object) if self.projection_path_is_map(root_slot_id, parent_path) => {
                let sample_path =
                    append_json_path_segment(parent_path, JsonPathSegment::Key(String::new()));
                let (child_matches, child_has_matches_below) =
                    self.projection_filter_relation(root_slot_id, &sample_path);
                if !child_has_matches_below && !self.has_value_filters() {
                    object.len() * usize::from(child_matches)
                } else {
                    object
                        .keys()
                        .map(|key| {
                            let path = append_json_path_segment(
                                parent_path,
                                JsonPathSegment::Key(key.clone()),
                            );
                            usize::from(
                                self.projection_path_matches_shape_filter(root_slot_id, &path),
                            ) + self.filtered_projection_descendant_count(root_slot_id, &path)
                        })
                        .sum()
                }
            }
            Value::Object(object) => object
                .keys()
                .map(|field_name| {
                    let path = append_json_path_segment(
                        parent_path,
                        JsonPathSegment::Field(field_name.clone()),
                    );
                    usize::from(self.projection_path_matches_shape_filter(root_slot_id, &path))
                        + self.filtered_projection_descendant_count(root_slot_id, &path)
                })
                .sum(),
            _ => 0,
        };

        self.projection_cache
            .borrow_mut()
            .filtered_descendant_counts
            .insert(cache_key, descendant_count);
        descendant_count
    }

    fn filtered_projection_descendant_path_at(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
        child_index: usize,
    ) -> Option<Vec<JsonPathSegment>> {
        let cache_key = FilteredPathCacheKey {
            parent: ProjectionCacheKey::new(root_slot_id, parent_path),
            child_index,
        };
        if let Some(path) = self
            .projection_cache
            .borrow()
            .filtered_paths
            .get(&cache_key)
            .cloned()
        {
            return Some(path);
        }
        let path = self.filtered_projection_descendant_path_at_uncached(
            root_slot_id,
            parent_path,
            child_index,
        )?;
        self.projection_cache
            .borrow_mut()
            .filtered_paths
            .insert(cache_key, path.clone());
        Some(path)
    }

    fn filtered_projection_descendant_path_at_uncached(
        &self,
        root_slot_id: usize,
        parent_path: &[JsonPathSegment],
        child_index: usize,
    ) -> Option<Vec<JsonPathSegment>> {
        let value = self.json_value_at_path(root_slot_id, parent_path)?;
        let mut remaining = child_index;

        match value {
            Value::Array(items) => {
                let sample_path = append_json_path_segment(parent_path, JsonPathSegment::Index(0));
                let (child_matches, child_has_matches_below) =
                    self.projection_filter_relation(root_slot_id, &sample_path);
                if !child_has_matches_below && !self.has_value_filters() {
                    return (child_matches && remaining < items.len()).then(|| {
                        append_json_path_segment(parent_path, JsonPathSegment::Index(remaining))
                    });
                }
            }
            Value::Object(object) if self.projection_path_is_map(root_slot_id, parent_path) => {
                let sample_path =
                    append_json_path_segment(parent_path, JsonPathSegment::Key(String::new()));
                let (child_matches, child_has_matches_below) =
                    self.projection_filter_relation(root_slot_id, &sample_path);
                if !child_has_matches_below && !self.has_value_filters() {
                    let key = child_matches.then(|| object.keys().nth(remaining).cloned())??;
                    return Some(append_json_path_segment(
                        parent_path,
                        JsonPathSegment::Key(key),
                    ));
                }
            }
            _ => {}
        }

        let mut visit_child = |path: Vec<JsonPathSegment>| {
            if self.projection_path_matches_shape_filter(root_slot_id, &path) {
                if remaining == 0 {
                    return Some(path);
                }
                remaining -= 1;
            }
            let descendant_count = self.filtered_projection_descendant_count(root_slot_id, &path);
            if remaining < descendant_count {
                return self.filtered_projection_descendant_path_at(root_slot_id, &path, remaining);
            }
            remaining = remaining.saturating_sub(descendant_count);
            None
        };

        match value {
            Value::Array(items) => {
                for index in 0..items.len() {
                    let path = append_json_path_segment(parent_path, JsonPathSegment::Index(index));
                    if let Some(path) = visit_child(path) {
                        return Some(path);
                    }
                }
            }
            Value::Object(object) if self.projection_path_is_map(root_slot_id, parent_path) => {
                for key in object.keys() {
                    let path =
                        append_json_path_segment(parent_path, JsonPathSegment::Key(key.clone()));
                    if let Some(path) = visit_child(path) {
                        return Some(path);
                    }
                }
            }
            Value::Object(object) => {
                for field_name in object.keys() {
                    let path = append_json_path_segment(
                        parent_path,
                        JsonPathSegment::Field(field_name.clone()),
                    );
                    if let Some(path) = visit_child(path) {
                        return Some(path);
                    }
                }
            }
            _ => {}
        }
        None
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
        if self.breadcrumb_filters.is_empty() {
            return self.pool_entry_at_unfiltered(slot_index);
        }
        let mut remaining = slot_index;
        if let Some(view) = self.current_projection_view() {
            if self.projection_path_matches_shape_filter(view.root_slot_id, &view.path) {
                if remaining == 0 {
                    return Some(PoolEntry::Projection(ProjectionSlot {
                        root_slot_id: view.root_slot_id,
                        path: view.path.clone(),
                        role: ProjectionSlotRole::ContainerRoot,
                    }));
                }
                remaining -= 1;
            }
            let path = self.filtered_projection_descendant_path_at(
                view.root_slot_id,
                &view.path,
                remaining,
            )?;
            return Some(PoolEntry::Projection(ProjectionSlot {
                root_slot_id: view.root_slot_id,
                path,
                role: ProjectionSlotRole::Child,
            }));
        }

        for slot in &self.object_slots {
            if self.real_slot_matches_shape_filter(slot.id) {
                if remaining == 0 {
                    return Some(PoolEntry::RealSlot(slot.id));
                }
                remaining -= 1;
            }
            let descendant_count = self.filtered_projection_descendant_count(slot.id, &[]);
            if remaining < descendant_count {
                let path = self.filtered_projection_descendant_path_at(slot.id, &[], remaining)?;
                return Some(PoolEntry::Projection(ProjectionSlot {
                    root_slot_id: slot.id,
                    path,
                    role: ProjectionSlotRole::Child,
                }));
            }
            remaining = remaining.saturating_sub(descendant_count);
        }

        (remaining == 0).then_some(PoolEntry::NewSlot)
    }

    fn real_slot_matches_shape_filter(&self, slot_id: usize) -> bool {
        let shape_name = self.slot_shape_name(slot_id);
        let slot_kind = self.slot_by_id(slot_id).map(|slot| match &slot.kind {
            SlotKind::Owned => SlotFilterKind::Owned,
            SlotKind::View(_) => SlotFilterKind::View,
        });
        self.breadcrumb_filters.iter().all(|filter| match filter {
            BreadcrumbFilter::Shape(filter) => {
                shape_name.is_some_and(|shape| filter.included_shapes.contains(shape))
            }
            BreadcrumbFilter::Value(filter) => self.value_filter_path_matches(slot_id, &[], filter),
            BreadcrumbFilter::SlotKind(filter) => {
                slot_kind.is_some_and(|kind| filter.included_kinds.contains(&kind))
            }
        })
    }

    fn projection_path_matches_shape_filter(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
    ) -> bool {
        let shape_name = self.projection_shape_name_at_path(root_slot_id, path);
        self.breadcrumb_filters.iter().all(|filter| match filter {
            BreadcrumbFilter::Shape(filter) => shape_name
                .as_ref()
                .is_some_and(|shape| filter.included_shapes.contains(shape)),
            BreadcrumbFilter::Value(filter) => {
                self.value_filter_path_matches(root_slot_id, path, filter)
            }
            BreadcrumbFilter::SlotKind(filter) => {
                filter.included_kinds.contains(&SlotFilterKind::Projection)
            }
        })
    }

    fn projection_slot_kind_filter_matches(&self) -> bool {
        self.breadcrumb_filters.iter().all(|filter| match filter {
            BreadcrumbFilter::SlotKind(filter) => {
                filter.included_kinds.contains(&SlotFilterKind::Projection)
            }
            BreadcrumbFilter::Shape(_) | BreadcrumbFilter::Value(_) => true,
        })
    }

    fn projection_filter_relation(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
    ) -> (bool, bool) {
        if !self.projection_slot_kind_filter_matches() {
            return (false, false);
        }
        let shape_filters = self
            .breadcrumb_filters
            .iter()
            .filter_map(|filter| match filter {
                BreadcrumbFilter::Shape(filter) => Some(filter),
                BreadcrumbFilter::Value(_) | BreadcrumbFilter::SlotKind(_) => None,
            });
        let shape_filters = shape_filters.collect::<Vec<_>>();
        if shape_filters.is_empty() {
            return (true, true);
        }
        let Some(shape_name) = self.projection_shape_name_at_path(root_slot_id, path) else {
            return (false, false);
        };
        if let Some(relation) = self
            .projection_cache
            .borrow()
            .filter_shape_relations
            .get(&shape_name)
            .copied()
        {
            return relation;
        }
        let mut reachable_shapes = self.projection_shape_names_at_path(root_slot_id, path);
        reachable_shapes.remove(&shape_name);
        let relation = (
            shape_filters
                .iter()
                .all(|filter| filter.included_shapes.contains(&shape_name)),
            reachable_shapes.iter().any(|shape| {
                shape_filters
                    .iter()
                    .all(|filter| filter.included_shapes.contains(shape))
            }),
        );
        self.projection_cache
            .borrow_mut()
            .filter_shape_relations
            .insert(shape_name, relation);
        relation
    }

    fn has_value_filters(&self) -> bool {
        self.breadcrumb_filters
            .iter()
            .any(|filter| matches!(filter, BreadcrumbFilter::Value(_)))
    }

    fn value_filter_matches(
        &self,
        filter: &ValueFilterView,
        shape_name: Option<&str>,
        field_name: Option<&str>,
        value: Option<&Value>,
    ) -> bool {
        if filter.field_shape != "*" && shape_name.is_none_or(|shape| shape != filter.field_shape) {
            return false;
        }
        if filter.field_name != "*" && field_name.is_none_or(|name| name != filter.field_name) {
            return false;
        }
        let Some(value) = value else {
            return false;
        };
        let candidate = match value {
            Value::String(value) => value.clone(),
            _ => value.to_string(),
        };
        match filter.operator {
            ValueFilterOperator::Equals => candidate == filter.value,
            ValueFilterOperator::NotEquals => candidate != filter.value,
            ValueFilterOperator::Contains => candidate.contains(&filter.value),
        }
    }

    fn value_filter_path_matches(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
        filter: &ValueFilterView,
    ) -> bool {
        let cache_key = ValueFilterCacheKey {
            root_slot_id,
            filter: filter.clone(),
        };
        if !self
            .projection_cache
            .borrow()
            .value_filter_match_roots
            .contains_key(&cache_key)
        {
            let mut match_roots = HashSet::new();
            if let Some(value) = self.json_value_at_path(root_slot_id, &[]) {
                self.collect_value_filter_match_roots(
                    root_slot_id,
                    &[],
                    value,
                    filter,
                    &mut match_roots,
                );
            }
            self.projection_cache
                .borrow_mut()
                .value_filter_match_roots
                .insert(cache_key.clone(), match_roots);
        }
        let cache = self.projection_cache.borrow();
        let Some(match_roots) = cache.value_filter_match_roots.get(&cache_key) else {
            return false;
        };
        (0..=path.len()).any(|prefix_len| match_roots.contains(&path[..prefix_len]))
    }

    fn collect_value_filter_match_roots(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
        value: &Value,
        filter: &ValueFilterView,
        match_roots: &mut HashSet<Vec<JsonPathSegment>>,
    ) {
        match value {
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    let child_path = append_json_path_segment(path, JsonPathSegment::Index(index));
                    self.collect_value_filter_match_roots(
                        root_slot_id,
                        &child_path,
                        child,
                        filter,
                        match_roots,
                    );
                }
            }
            Value::Object(object) => {
                let is_map = self.projection_path_is_map(root_slot_id, path);
                if !is_map {
                    let object_matches = object.iter().any(|(field_name, field_value)| {
                        let field_path = append_json_path_segment(
                            path,
                            JsonPathSegment::Field(field_name.clone()),
                        );
                        let shape_name =
                            self.projection_shape_name_at_path(root_slot_id, &field_path);
                        self.value_filter_matches(
                            filter,
                            shape_name.as_deref(),
                            Some(field_name),
                            Some(field_value),
                        )
                    });
                    if object_matches {
                        match_roots.insert(path.to_vec());
                    }
                }
                for (name, child) in object {
                    let segment = if is_map {
                        JsonPathSegment::Key(name.clone())
                    } else {
                        JsonPathSegment::Field(name.clone())
                    };
                    let child_path = append_json_path_segment(path, segment);
                    self.collect_value_filter_match_roots(
                        root_slot_id,
                        &child_path,
                        child,
                        filter,
                        match_roots,
                    );
                }
            }
            _ => {}
        }
    }

    fn pool_entry_at_unfiltered(&self, slot_index: usize) -> Option<PoolEntry> {
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
        let SlotValueState::ResolvedValue { value, .. } = &slot.value_state else {
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
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))?;

        for segment in path {
            current_shape = match segment {
                JsonPathSegment::Field(field_name) => shape_field_shape(current_shape, field_name)?,
                JsonPathSegment::Index(_) => sequence_element_shape(current_shape)?,
                JsonPathSegment::Key(_) => registry_map_value_shape(current_shape)?,
            };
        }

        Some(describe_shape(current_shape))
    }

    fn projection_shape_names_at_path(
        &self,
        root_slot_id: usize,
        path: &[JsonPathSegment],
    ) -> BTreeSet<String> {
        let Some(mut current_shape) = self
            .slot_shape_name(root_slot_id)
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))
        else {
            return BTreeSet::new();
        };
        for segment in path {
            let Some(next_shape) = (match segment {
                JsonPathSegment::Field(field_name) => shape_field_shape(current_shape, field_name),
                JsonPathSegment::Index(_) => sequence_element_shape(current_shape),
                JsonPathSegment::Key(_) => registry_map_value_shape(current_shape),
            }) else {
                return BTreeSet::new();
            };
            current_shape = next_shape;
        }
        projection_shape_names(current_shape)
    }

    fn projection_path_is_map(&self, root_slot_id: usize, path: &[JsonPathSegment]) -> bool {
        self.slot_shape_name(root_slot_id)
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))
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
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))
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
            .and_then(|shape_name| self.shape_for_shape_name(shape_name))
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
        if self
            .scalar_shape_for_slot(slot_id)
            .is_some_and(is_general_value_shape)
        {
            self.open_slot_value_editor(slot_id);
            return;
        }
        self.activate_json_projection(slot_id, Vec::new());
    }

    fn activate_json_projection(&mut self, root_slot_id: usize, path: Vec<JsonPathSegment>) {
        let Some(value) = self.json_value_at_path(root_slot_id, &path) else {
            self.status_message = "That projection is no longer available.".to_string();
            return;
        };

        if matches!(value, Value::Array(_) | Value::Object(_)) {
            self.breadcrumb_filters.clear();
            self.projection_cache.borrow_mut().clear();
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
        let scroll_offset = self.row_view_offset;
        let active_search = is_active
            .then_some(self.projection_search.as_ref())
            .flatten()
            .filter(|search| search.projection == *projection);
        let rendered_line_count = active_search
            .map(|search| search.filtered_matches.len().max(1))
            .unwrap_or_else(|| self.projection_rendered_line_count(projection));
        let visible_line_count = usize::from(inner.height).max(1);
        let lines = if let Some(search) = active_search {
            render_projection_search_matches(&search.filtered_matches, search.selected_match_index)
                .into_iter()
                .skip(scroll_offset)
                .take(visible_line_count)
                .collect()
        } else {
            self.projection_slot_lines_window(
                projection,
                is_active.then_some(self.active_row_index),
                scroll_offset..scroll_offset.saturating_add(visible_line_count),
            )
        };
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
            FieldPickerChoice::InvokeDefaultProducer {
                input_shape_name, ..
            }
            | FieldPickerChoice::InvokeArbitraryProducer {
                input_shape_name, ..
            } => self
                .shape_choices
                .iter()
                .find(|shape| shape.label == input_shape_name)
                .map(shape_preview_lines),
            FieldPickerChoice::CreateNewValue => Some(vec![
                Line::from(format!("Create a new owned {required_shape_name} value.")),
                Line::from("The value editor will validate the input before saving."),
            ]),
            FieldPickerChoice::CreateNew => self
                .shape_choices
                .iter()
                .find(|shape| shape.label == required_shape_name)
                .map(shape_preview_lines),
            FieldPickerChoice::CreateNewVariant { variant_index, .. } => self
                .thing_for_shape_name(&required_shape_name)
                .and_then(|thing| shape_variants_for_thing(thing).get(variant_index).cloned())
                .map(|variant| variant_preview_lines(&required_shape_name, &variant)),
        }
    }

    fn arbitrary_source_picker_preview_lines(&mut self) -> Option<Vec<Line<'static>>> {
        let choice = self.arbitrary_source_picker.as_ref()?.selected_choice()?;
        match choice {
            ArbitrarySourceChoice::ExistingSlot { slot_id } => {
                Some(self.slot_preview_lines(slot_id))
            }
            ArbitrarySourceChoice::CreateNew => self
                .shape_choices
                .iter()
                .find(|shape| shape.label == "ArbitraryBytes")
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
        let Some(action) = picker.selected_action() else {
            return vec![Line::from("No matching action.")];
        };
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

    fn field_rows(&self, fields: &[ObjectFieldState]) -> Vec<SlotDisplayRow> {
        let mut rows = Vec::with_capacity(fields.len() * 2);
        for (index, field) in fields.iter().enumerate() {
            let accent = field_group_color(index);
            let (linked_style, linked_label) = match field.value_state {
                FieldValueState::Linked { slot_id } => match self.slot_runtime_state(slot_id) {
                    Some(SlotValueState::Pending(_)) => (
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                        "pending".to_string(),
                    ),
                    Some(SlotValueState::Failed { .. }) => (unset_style(), "failed".to_string()),
                    _ if !matches!(self.slot_completion(slot_id), SlotCompletion::Complete) => {
                        (unset_style(), format!("slot {slot_id}"))
                    }
                    _ => (
                        Style::default().fg(accent).add_modifier(Modifier::BOLD),
                        format!("slot {slot_id}"),
                    ),
                },
                _ => (
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                    String::new(),
                ),
            };
            rows.push(focusable_spans_row(
                SlotFocusTarget::FieldType(index),
                field.type_spans(accent),
            ));
            rows.push(focusable_spans_row(
                SlotFocusTarget::FieldValue(index),
                field.value_spans(accent, linked_style, linked_label),
            ));
        }
        rows
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

    fn projection_search_matches(
        &self,
        projection: &ProjectionSlot,
        query: &str,
    ) -> Vec<ProjectionSearchMatch> {
        let Some(value) = self.projection_value(projection) else {
            return Vec::new();
        };
        let mut rows = vec![(
            0,
            vec![Span::raw(format!(
                "{} {}",
                self.projection_header_label(projection, value),
                json_value_summary(value)
            ))],
        )];
        if let Value::Object(object) = value {
            let is_map = self.projection_path_is_map(projection.root_slot_id, &projection.path);
            for (index, (name, field_value)) in object.iter().enumerate() {
                let type_label = if is_map {
                    self.projection_map_entry_type_label(
                        projection.root_slot_id,
                        &projection.path,
                        field_value,
                    )
                } else {
                    self.projection_field_type_label(
                        projection.root_slot_id,
                        &projection.path,
                        name,
                        field_value,
                    )
                };
                rows.push((1 + index * 2, vec![Span::raw(format!("type {type_label}"))]));
                rows.push((
                    2 + index * 2,
                    vec![Span::raw(format!(
                        "{name}: {}",
                        json_value_summary(field_value)
                    ))],
                ));
            }
        }
        let labels = rows
            .iter()
            .map(|(_, spans)| spans_plain_text(spans))
            .collect::<Vec<_>>();
        ranked_slot_search_indices(query, &labels)
            .into_iter()
            .filter_map(|index| {
                let (row_index, spans) = rows.get(index)?.clone();
                let matched_indices =
                    match_indices(query, &spans_plain_text(&spans)).unwrap_or_default();
                Some(ProjectionSearchMatch {
                    row_index,
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
                SlotValueState::Pending(_) => Color::Yellow,
                SlotValueState::ResolvedValue { .. } => Color::Green,
                SlotValueState::Failed { .. } => Color::Red,
                SlotValueState::Building(_) => unreachable!("builders are filtered out"),
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
        self.slot_completion_inner(slot_id, &mut BTreeSet::new())
    }

    fn slot_completion_inner(
        &self,
        slot_id: usize,
        visiting: &mut BTreeSet<usize>,
    ) -> SlotCompletion {
        if !visiting.insert(slot_id) {
            return SlotCompletion::Partial;
        }

        let completion = if let Some(runtime_state) = self.slot_runtime_state(slot_id) {
            match runtime_state {
                SlotValueState::Pending(_) => SlotCompletion::Partial,
                SlotValueState::ResolvedValue { .. } => SlotCompletion::Complete,
                SlotValueState::Failed { .. } => SlotCompletion::Unset,
                SlotValueState::Building(_) => unreachable!("builders are filtered out"),
            }
        } else {
            let Some(shape_name) = self.slot_shape_name(slot_id) else {
                visiting.remove(&slot_id);
                return SlotCompletion::Unset;
            };
            if shape_name.is_empty() {
                visiting.remove(&slot_id);
                return SlotCompletion::Unset;
            }
            let Some(body) = self.slot_body(slot_id) else {
                visiting.remove(&slot_id);
                return SlotCompletion::Unset;
            };
            match body {
                SlotBody::Value { value, .. } => value
                    .as_ref()
                    .map_or(SlotCompletion::Unset, |_| SlotCompletion::Complete),
                SlotBody::Unset => SlotCompletion::Unset,
                SlotBody::Struct { fields } => {
                    if fields
                        .iter()
                        .all(|field| self.field_is_complete(field, visiting))
                    {
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
                    } else if fields
                        .iter()
                        .all(|field| self.field_is_complete(field, visiting))
                    {
                        SlotCompletion::Complete
                    } else {
                        SlotCompletion::Partial
                    }
                }
            }
        };

        visiting.remove(&slot_id);
        completion
    }

    fn field_is_complete(&self, field: &ObjectFieldState, visiting: &mut BTreeSet<usize>) -> bool {
        match field.value_state {
            FieldValueState::Defaulted => true,
            FieldValueState::Unset => false,
            FieldValueState::Linked { slot_id } => {
                self.slot_completion_inner(slot_id, visiting) == SlotCompletion::Complete
            }
        }
    }

    fn slot_runtime_state(&self, slot_id: usize) -> Option<&SlotValueState> {
        let data_slot_id = self.data_slot_id_for(slot_id)?;
        self.slot_by_id(data_slot_id)
            .map(|slot| &slot.value_state)
            .filter(|state| !matches!(state, SlotValueState::Building(_)))
    }

    fn result_slot_label(&self, slot_id: usize) -> String {
        let Some(slot) = self.slot_by_id(slot_id) else {
            return format!("slot {slot_id}");
        };
        let shape_name = slot.shape_name.as_deref().unwrap_or("unset");
        let status = match &slot.value_state {
            SlotValueState::Pending(_) => "pending",
            SlotValueState::ResolvedValue { .. } => "resolved",
            SlotValueState::Failed { .. } => "failed",
            SlotValueState::Building(_) => "ready",
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
    ArbitrarySourcePicker,
    FunctionPicker,
    LinkActionPicker,
    PartitionPicker,
    FilterKindPicker,
    ValueFilterEditor,
    ValueFilterChoicePicker,
    GeneralValueEditor,
    BooleanValuePicker,
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

#[derive(Clone, Debug)]
struct BrowserTabState {
    projection_stack: Vec<JsonProjectionView>,
    breadcrumb_filters: Vec<BreadcrumbFilter>,
    pool_surface: PoolSurface,
    active_breadcrumb_index: usize,
    active_slot_index: usize,
    active_row_index: usize,
    slot_view_offset: usize,
    row_view_offset: usize,
}

impl Default for BrowserTabState {
    fn default() -> Self {
        Self {
            projection_stack: Vec::new(),
            breadcrumb_filters: Vec::new(),
            pool_surface: PoolSurface::Slots,
            active_breadcrumb_index: 0,
            active_slot_index: 0,
            active_row_index: 0,
            slot_view_offset: 0,
            row_view_offset: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShapeFilterView {
    included_shapes: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SlotKindFilterView {
    included_kinds: BTreeSet<SlotFilterKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum SlotFilterKind {
    Owned,
    View,
    Projection,
}

impl SlotFilterKind {
    fn label(self) -> &'static str {
        match self {
            Self::Owned => "owned",
            Self::View => "view",
            Self::Projection => "projection",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "owned" => Some(Self::Owned),
            "view" => Some(Self::View),
            "projection" => Some(Self::Projection),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum BreadcrumbFilter {
    Shape(ShapeFilterView),
    Value(ValueFilterView),
    SlotKind(SlotKindFilterView),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ValueFilterView {
    field_shape: String,
    field_name: String,
    operator: ValueFilterOperator,
    value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum ValueFilterOperator {
    Equals,
    NotEquals,
    Contains,
}

impl ValueFilterOperator {
    fn label(self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "not equals",
            Self::Contains => "contains",
        }
    }
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
    filtered_descendant_counts: HashMap<ProjectionCacheKey, usize>,
    filter_shape_relations: HashMap<String, (bool, bool)>,
    value_filter_match_roots: HashMap<ValueFilterCacheKey, HashSet<Vec<JsonPathSegment>>>,
    filtered_paths: HashMap<FilteredPathCacheKey, Vec<JsonPathSegment>>,
}

impl ProjectionCache {
    fn clear(&mut self) {
        self.descendant_counts.clear();
        self.filtered_descendant_counts.clear();
        self.filter_shape_relations.clear();
        self.value_filter_match_roots.clear();
        self.filtered_paths.clear();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct ValueFilterCacheKey {
    root_slot_id: usize,
    filter: ValueFilterView,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct FilteredPathCacheKey {
    parent: ProjectionCacheKey,
    child_index: usize,
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
struct MaterializedFieldState {
    info: ShapeFieldInfo,
    value: Value,
    projection_path: Vec<JsonPathSegment>,
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

#[derive(Clone, Debug)]
struct ProjectionSearchMatch {
    row_index: usize,
    spans: Vec<Span<'static>>,
    matched_indices: Vec<u32>,
}

struct ProjectionSlotSearchState {
    projection: ProjectionSlot,
    query: TextArea<'static>,
    filtered_matches: Vec<ProjectionSearchMatch>,
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
                if !self.filtered_indices.is_empty() {
                    let position = self.list_state.selected().unwrap_or(0).saturating_sub(10);
                    self.list_state.select(Some(position));
                }
                return PickerSearchAction::None;
            }
            KeyCode::PageDown => {
                if !self.filtered_indices.is_empty() {
                    let position = self
                        .list_state
                        .selected()
                        .unwrap_or(0)
                        .saturating_add(10)
                        .min(self.filtered_indices.len().saturating_sub(1));
                    self.list_state.select(Some(position));
                }
                return PickerSearchAction::None;
            }
            _ => {}
        }

        let previous_query = self.query.lines().join("\n");
        if self.query.input(key) {
            let query = self.query.lines().join("\n");
            if query != previous_query {
                self.filtered_indices = filter_indices(&query, labels);
                self.select_preferred(None);
                self.preview_scroll = 0;
            }
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

struct ArbitrarySourcePickerState {
    request_slot_id: usize,
    request_function: &'static Function,
    constructor: &'static Function,
    labels: Vec<String>,
    choices: Vec<ArbitrarySourceChoice>,
    search: PickerSearchState,
}

impl ArbitrarySourcePickerState {
    fn new(
        request_slot_id: usize,
        request_function: &'static Function,
        constructor: &'static Function,
        choices: Vec<ArbitrarySourceChoice>,
        labels: Vec<String>,
    ) -> Self {
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        Self {
            request_slot_id,
            request_function,
            constructor,
            labels,
            choices,
            search,
        }
    }

    fn selected_choice(&self) -> Option<ArbitrarySourceChoice> {
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
enum ArbitrarySourceChoice {
    ExistingSlot { slot_id: usize },
    CreateNew,
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
    InvokeDefaultProducer {
        input_shape_name: String,
        function_label: String,
    },
    InvokeArbitraryProducer {
        input_shape_name: String,
        function_label: String,
    },
    CreateNew,
    CreateNewValue,
    CreateNewVariant {
        variant_index: usize,
        variant_name: String,
    },
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

fn reflected_field_default_is_materializable(field: &ShapeFieldInfo) -> bool {
    field.has_default
        && field
            .default_value_label
            .as_deref()
            .is_some_and(|label| label != "<default>")
}

struct GeneralValueEditorState {
    slot_id: usize,
    shape_name: String,
    shape: &'static facet::Shape,
    textarea: TextArea<'static>,
    validation_error: Option<String>,
}
impl GeneralValueEditorState {
    fn new(
        slot_id: usize,
        shape_name: String,
        shape: &'static facet::Shape,
        input: String,
    ) -> Self {
        Self {
            slot_id,
            shape_name,
            shape,
            textarea: build_text_area(&input),
            validation_error: None,
        }
    }
}
struct BooleanValuePickerState {
    slot_id: usize,
    shape_name: String,
    labels: Vec<String>,
    search: PickerSearchState,
}
impl BooleanValuePickerState {
    fn new(slot_id: usize, shape_name: String, selected: Option<usize>) -> Self {
        let labels = vec!["false".to_string(), "true".to_string()];
        let mut search = PickerSearchState::new();
        search.reset(&labels, selected);
        Self {
            slot_id,
            shape_name,
            labels,
            search,
        }
    }
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
    labels: Vec<String>,
    actions: Vec<LinkAction>,
    search: PickerSearchState,
}

impl LinkActionPickerState {
    fn new(owner_slot_id: usize, field_index: usize, selected_slot_id: usize) -> Self {
        let labels = vec!["Move".to_string(), "Clone".to_string()];
        let actions = vec![LinkAction::Move, LinkAction::Clone];
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        Self {
            owner_slot_id,
            field_index,
            selected_slot_id,
            labels,
            actions,
            search,
        }
    }

    fn selected_action(&self) -> Option<LinkAction> {
        let index = self.search.selected_filtered_index()?;
        self.actions.get(index).copied()
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
enum LinkAction {
    Move,
    Clone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PartitionPickerTarget {
    ShapeFilter { edit_index: Option<usize> },
    SlotKindFilter { edit_index: Option<usize> },
}

struct FilterKindPickerState {
    labels: Vec<String>,
    search: PickerSearchState,
}

impl FilterKindPickerState {
    fn new() -> Self {
        let labels = vec![
            "filter shape".to_string(),
            "filter value".to_string(),
            "filter slot kind".to_string(),
        ];
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        Self { labels, search }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueFilterSource {
    Existing,
    Literal,
}

impl ValueFilterSource {
    fn label(self) -> &'static str {
        match self {
            Self::Existing => "choose from existing",
            Self::Literal => "literal",
        }
    }
}

struct ValueFilterEditorState {
    edit_index: Option<usize>,
    draft: ValueFilterView,
    source: ValueFilterSource,
    active_row: usize,
    literal_input: TextArea<'static>,
}

impl ValueFilterEditorState {
    fn new(edit_index: Option<usize>, draft: ValueFilterView) -> Self {
        let literal_input = build_text_area(&draft.value);
        Self {
            edit_index,
            draft,
            source: ValueFilterSource::Existing,
            active_row: 0,
            literal_input,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueFilterChoiceTarget {
    FieldShape,
    FieldName,
    Operator,
    ExistingValue,
}

struct ValueFilterChoicePickerState {
    target: ValueFilterChoiceTarget,
    labels: Vec<String>,
    search: PickerSearchState,
    worker: Option<Nucleo<usize>>,
}

impl ValueFilterChoicePickerState {
    const ASYNC_THRESHOLD: usize = 5_000;
    const MAX_VISIBLE_MATCHES: usize = 2_048;

    fn new(target: ValueFilterChoiceTarget, labels: Vec<String>) -> Self {
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        let worker = (labels.len() >= Self::ASYNC_THRESHOLD).then(|| {
            let mut worker = Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
            let injector = worker.injector();
            for index in 0..labels.len() {
                injector.push(index, |index, columns: &mut [Utf32String]| {
                    columns[0] = labels[*index].as_str().into();
                });
            }
            worker
                .pattern
                .reparse(0, "", CaseMatching::Smart, Normalization::Smart, false);
            worker.tick(0);
            worker
        });
        if worker.is_some() {
            search.filtered_indices = (0..labels.len().min(Self::MAX_VISIBLE_MATCHES)).collect();
            search.list_state.select(Some(0));
        }
        Self {
            target,
            labels,
            search,
            worker,
        }
    }

    fn refresh_worker(&mut self) {
        let Some(worker) = self.worker.as_mut() else {
            return;
        };
        let status = worker.tick(1);
        if !status.changed && status.running {
            return;
        }
        let snapshot = worker.snapshot();
        self.search.filtered_indices = snapshot
            .matched_items(
                0..snapshot
                    .matched_item_count()
                    .min(Self::MAX_VISIBLE_MATCHES as u32),
            )
            .map(|item| *item.data)
            .collect();
        self.search.select_preferred(None);
    }

    fn handle_key(&mut self, key: KeyEvent) -> PickerSearchAction {
        if self.worker.is_none() {
            return self.search.handle_key(key, &self.labels);
        }
        match key.code {
            KeyCode::Esc => return PickerSearchAction::Cancel,
            KeyCode::Enter => return PickerSearchAction::Submit,
            KeyCode::Up => self.search.list_state.select_previous(),
            KeyCode::Down => self.search.list_state.select_next(),
            KeyCode::Home if !self.search.filtered_indices.is_empty() => {
                self.search.list_state.select(Some(0));
            }
            KeyCode::End if !self.search.filtered_indices.is_empty() => {
                self.search
                    .list_state
                    .select(Some(self.search.filtered_indices.len().saturating_sub(1)));
            }
            KeyCode::PageUp if !self.search.filtered_indices.is_empty() => {
                let position = self
                    .search
                    .list_state
                    .selected()
                    .unwrap_or(0)
                    .saturating_sub(10);
                self.search.list_state.select(Some(position));
            }
            KeyCode::PageDown if !self.search.filtered_indices.is_empty() => {
                let position = self
                    .search
                    .list_state
                    .selected()
                    .unwrap_or(0)
                    .saturating_add(10)
                    .min(self.search.filtered_indices.len().saturating_sub(1));
                self.search.list_state.select(Some(position));
            }
            _ => {
                let previous_query = self.search.query.lines().concat();
                if self.search.query.input(key) {
                    let query = self.search.query.lines().concat();
                    if query != previous_query {
                        let append = query.starts_with(&previous_query);
                        if let Some(worker) = self.worker.as_mut() {
                            worker.pattern.reparse(
                                0,
                                &query,
                                CaseMatching::Smart,
                                Normalization::Smart,
                                append,
                            );
                            worker.tick(0);
                        }
                        self.search.list_state.select(Some(0));
                    }
                }
            }
        }
        PickerSearchAction::None
    }
}

struct PartitionPickerState {
    target: PartitionPickerTarget,
    labels: Vec<String>,
    included_indices: BTreeSet<usize>,
    selected_indices: BTreeSet<usize>,
    range_anchor: Option<usize>,
    search: PickerSearchState,
}

impl PartitionPickerState {
    fn new(target: PartitionPickerTarget, labels: Vec<String>) -> Self {
        let mut search = PickerSearchState::new();
        search.reset(&labels, Some(0));
        let selected_indices = search.selected_filtered_index().into_iter().collect();
        Self {
            target,
            labels,
            included_indices: BTreeSet::new(),
            selected_indices,
            range_anchor: search.selected_filtered_index(),
            search,
        }
    }

    fn with_included_labels(
        target: PartitionPickerTarget,
        labels: Vec<String>,
        included_labels: &BTreeSet<String>,
    ) -> Self {
        let mut picker = Self::new(target, labels);
        picker.included_indices = picker
            .labels
            .iter()
            .enumerate()
            .filter_map(|(index, label)| included_labels.contains(label).then_some(index))
            .collect();
        picker
    }

    fn current_index(&self) -> Option<usize> {
        self.search.selected_filtered_index()
    }

    fn reset_selection_to_current(&mut self) {
        self.selected_indices.clear();
        if let Some(index) = self.current_index() {
            self.selected_indices.insert(index);
            self.range_anchor = Some(index);
        }
    }

    fn extend_selection_to_current(&mut self) {
        let Some(anchor) = self.range_anchor else {
            self.reset_selection_to_current();
            return;
        };
        let Some(current) = self.current_index() else {
            return;
        };
        let Some(anchor_position) = self
            .search
            .filtered_indices
            .iter()
            .position(|index| *index == anchor)
        else {
            self.reset_selection_to_current();
            return;
        };
        let Some(current_position) = self
            .search
            .filtered_indices
            .iter()
            .position(|index| *index == current)
        else {
            return;
        };
        let start = anchor_position.min(current_position);
        let end = anchor_position.max(current_position);
        self.selected_indices = self.search.filtered_indices[start..=end]
            .iter()
            .copied()
            .collect();
    }

    fn move_selected(&mut self, include: bool) {
        for index in &self.selected_indices {
            if include {
                self.included_indices.insert(*index);
            } else {
                self.included_indices.remove(index);
            }
        }
    }

    fn toggle_selected(&mut self) {
        let include = self
            .current_index()
            .is_some_and(|index| !self.included_indices.contains(&index));
        self.move_selected(include);
    }

    fn select_all_filtered(&mut self) {
        self.selected_indices = self.search.filtered_indices.iter().copied().collect();
    }

    fn included_labels(&self) -> BTreeSet<String> {
        self.included_indices
            .iter()
            .filter_map(|index| self.labels.get(*index).cloned())
            .collect()
    }
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
    value_state: SlotSnapshotValueState,
}

#[derive(Clone, Debug)]
enum SlotSnapshotValueState {
    Building(SlotBody),
    ResolvedValue { json: String, value: Value },
}

#[derive(Debug)]
enum SlotValueState {
    Building(SlotBody),
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
    value_state: SlotValueState,
    result_slot_ids: Vec<usize>,
    created_for: Option<SlotCreatedFor>,
    produced_by_slot_id: Option<usize>,
    display_cache: Option<Vec<SlotDisplayRow>>,
}

impl ObjectSlot {
    fn from_snapshot(id: usize, snapshot: SlotSnapshot) -> Self {
        Self {
            id,
            name: snapshot.name,
            kind: SlotKind::Owned,
            shape_name: snapshot.shape_name,
            value_state: match snapshot.value_state {
                SlotSnapshotValueState::Building(body) => SlotValueState::Building(body),
                SlotSnapshotValueState::ResolvedValue { json, value } => {
                    SlotValueState::ResolvedValue { json, value }
                }
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        }
    }

    fn new(id: usize) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::Owned,
            shape_name: None,
            value_state: SlotValueState::Building(SlotBody::Unset),
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        }
    }

    fn new_result(id: usize, shape_name: String, pending: PendingInvocationState) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::Owned,
            shape_name: Some(shape_name),
            value_state: SlotValueState::Pending(pending),
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        }
    }

    fn new_resolved_result(id: usize, shape_name: String, json: String, value: Value) -> Self {
        Self {
            id,
            name: None,
            kind: SlotKind::Owned,
            shape_name: Some(shape_name),
            value_state: SlotValueState::ResolvedValue { json, value },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        }
    }

    fn new_scalar(id: usize, shape_name: String, shape: &'static facet::Shape) -> Self {
        let mut slot = Self::new(id);
        slot.shape_name = Some(shape_name);
        slot.value_state = SlotValueState::Building(SlotBody::Value { shape, value: None });
        slot
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
            value_state: SlotValueState::Building(SlotBody::Unset),
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        }
    }

    fn apply_shape_choice(&mut self, choice: &KnownShapeInfo) {
        self.shape_name = Some(choice.label.clone());
        if is_general_value_shape(choice.thing.shape) {
            self.value_state = SlotValueState::Building(SlotBody::Value {
                shape: choice.thing.shape,
                value: None,
            });
            return;
        }

        let variants = shape_variants_for_thing(choice.thing)
            .into_iter()
            .map(ObjectVariantState::new)
            .collect::<Vec<_>>();
        if !variants.is_empty() {
            self.value_state = SlotValueState::Building(SlotBody::Enum {
                variants,
                selected_variant: None,
                fields: Vec::new(),
            });
            return;
        }

        let fields = shape_fields_for_thing(choice.thing)
            .into_iter()
            .map(ObjectFieldState::new)
            .collect::<Vec<_>>();
        self.value_state = SlotValueState::Building(SlotBody::Struct { fields });
    }

    fn building_body(&self) -> Option<&SlotBody> {
        match &self.value_state {
            SlotValueState::Building(body) => Some(body),
            _ => None,
        }
    }

    fn building_body_mut(&mut self) -> Option<&mut SlotBody> {
        match &mut self.value_state {
            SlotValueState::Building(body) => Some(body),
            _ => None,
        }
    }

    fn variant_picker_seed(&self) -> Option<(String, Vec<ShapeVariantInfo>, Option<usize>)> {
        let shape_name = self.shape_name.clone()?;
        let SlotBody::Enum {
            variants,
            selected_variant,
            ..
        } = self.building_body()?
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
        } = self.building_body_mut()?
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
        let Some(body) = self.building_body() else {
            return SlotFocusTarget::RuntimeValue;
        };
        match body {
            SlotBody::Value { .. } => SlotFocusTarget::RuntimeValue,
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
        match self.building_body()? {
            SlotBody::Struct { fields } | SlotBody::Enum { fields, .. } => fields.get(field_index),
            SlotBody::Value { .. } | SlotBody::Unset => None,
        }
    }

    fn field_mut(&mut self, field_index: usize) -> Option<&mut ObjectFieldState> {
        match self.building_body_mut()? {
            SlotBody::Struct { fields } | SlotBody::Enum { fields, .. } => {
                fields.get_mut(field_index)
            }
            SlotBody::Value { .. } | SlotBody::Unset => None,
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
    Value {
        shape: &'static facet::Shape,
        value: Option<Value>,
    },
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

    fn value_spans(
        &self,
        accent: Color,
        linked_style: Style,
        linked_label: String,
    ) -> Vec<Span<'static>> {
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
            FieldValueState::Linked { .. } => spans.push(Span::styled(linked_label, linked_style)),
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

fn materialized_field_rows(fields: &[MaterializedFieldState]) -> Vec<SlotDisplayRow> {
    let mut rows = Vec::with_capacity(fields.len() * 2);
    for (index, field) in fields.iter().enumerate() {
        let accent = field_group_color(index);
        rows.push(focusable_spans_row(
            SlotFocusTarget::FieldType(index),
            vec![
                Span::styled(
                    "type ",
                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    field.info.field_shape_name.clone(),
                    Style::default().fg(accent).add_modifier(Modifier::DIM),
                ),
            ],
        ));
        rows.push(focusable_spans_row(
            SlotFocusTarget::FieldValue(index),
            vec![
                Span::styled(
                    format!("{}: ", field.info.field_name),
                    Style::default().fg(accent),
                ),
                Span::styled(
                    json_value_summary(&field.value),
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
            ],
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

fn runtime_state_rows(runtime_state: &SlotValueState) -> Vec<SlotDisplayRow> {
    match runtime_state {
        SlotValueState::Pending(_) => {
            vec![SlotDisplayRow::Static(Line::from(
                "  pending invocation...",
            ))]
        }
        SlotValueState::Failed { message } => vec![SlotDisplayRow::Static(Line::from(vec![
            Span::raw("  "),
            Span::styled("failed", unset_style()),
            Span::raw(format!(": {message}")),
        ]))],
        SlotValueState::ResolvedValue { value, .. } => vec![focusable_spans_row(
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
        SlotValueState::Building(_) => Vec::new(),
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

fn render_projection_search_matches(
    matches: &[ProjectionSearchMatch],
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

fn shape_accepts_text_input(shape: &facet::Shape) -> bool {
    shape.is_transparent()
        || shape.proxy.is_some()
        || shape
            .format_proxies
            .iter()
            .any(|proxy| proxy.format == "json")
}

fn is_general_value_shape(shape: &'static facet::Shape) -> bool {
    if shape.scalar_type().is_some() || shape.is_transparent() || describe_shape(shape) == "Uuid" {
        return true;
    }

    if matches!(shape.ty, facet::Type::User(facet::UserType::Enum(_))) {
        return false;
    }

    shape_accepts_text_input(shape)
}

fn scalar_value_input(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
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
    use ratatui::crossterm::event::Event;
    use ratatui::crossterm::event::KeyCode;
    use ratatui::crossterm::event::KeyEvent;
    use ratatui::crossterm::event::KeyModifiers;
    use std::borrow::Cow;
    use std::collections::BTreeSet;
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

    #[derive(Debug, Clone, Facet)]
    #[repr(C)]
    struct DummyCowOwner {
        value: Cow<'static, DummyInvokeOutput>,
    }

    #[derive(Debug, Clone, Facet, Default)]
    #[repr(C)]
    struct DummyCowProducerRequest;

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

    impl IntoFuture for DummyCowProducerRequest {
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
    cloud_terrastodon_registry::register_thing!(DummyCowOwner);
    cloud_terrastodon_registry::register_thing!(Cow<'static, DummyInvokeOutput>);
    cloud_terrastodon_registry::register_thing!(DummyCowProducerRequest);
    cloud_terrastodon_registry::register_into_future!(DummyInvokeRequest => DummyInvokeOutput);
    cloud_terrastodon_registry::register_into_future!(
        DummyCowProducerRequest => DummyInvokeOutput
    );
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
    fn proxy_scalar_shapes_use_value_slots() {
        let mut app = ObjectBrowserApp::default();
        let choice = app
            .shape_choices
            .iter()
            .find(|choice| choice.label == "EntraUserId")
            .cloned()
            .expect("EntraUserId should be registered");
        assert!(super::is_general_value_shape(choice.thing.shape));

        let principal_choice = app
            .shape_choices
            .iter()
            .find(|choice| choice.label == "PrincipalId")
            .cloned()
            .expect("PrincipalId should be registered");
        assert!(!super::is_general_value_shape(principal_choice.thing.shape));
        let unknown_variant_index = super::shape_variants_for_thing(principal_choice.thing)
            .iter()
            .position(|variant| variant.variant_name == "Unknown")
            .expect("PrincipalId::Unknown should be registered");
        let mut principal_slot = ObjectSlot::new(3);
        principal_slot.apply_shape_choice(&principal_choice);
        assert!(matches!(
            &principal_slot.value_state,
            super::SlotValueState::Building(SlotBody::Enum { .. })
        ));
        principal_slot
            .select_variant(unknown_variant_index)
            .expect("PrincipalId::Unknown should be selectable");
        app.object_slots.push(principal_slot);
        assert!(app.is_general_value_field(3, 0));

        let mut slot = ObjectSlot::new(1);
        slot.apply_shape_choice(&choice);
        assert!(matches!(
            slot.value_state,
            super::SlotValueState::Building(SlotBody::Value { .. })
        ));
        app.object_slots.push(slot);
        app.activate_runtime_value(1);
        assert_eq!(app.mode, UiMode::GeneralValueEditor);

        let mut legacy_slot = ObjectSlot::new(2);
        legacy_slot.shape_name = Some(choice.label.clone());
        legacy_slot.value_state = super::SlotValueState::Building(SlotBody::Struct {
            fields: super::shape_fields_for_thing(choice.thing)
                .into_iter()
                .map(super::ObjectFieldState::new)
                .collect(),
        });
        app.general_value_editor = None;
        app.mode = UiMode::Pool;
        app.object_slots.push(legacy_slot);
        app.active_slot_index = 2;
        app.activate_field_value(0);
        assert_eq!(app.mode, UiMode::GeneralValueEditor);
        assert!(matches!(
            app.slot_by_id(2).map(|slot| &slot.value_state),
            Some(super::SlotValueState::Building(SlotBody::Value { .. }))
        ));
    }

    #[test]
    fn cow_fields_discover_producers_for_the_inner_shape() {
        let mut app = ObjectBrowserApp::default();
        let owner_index = app
            .shape_choices
            .iter()
            .position(|shape| shape.label == "DummyCowOwner")
            .expect("DummyCowOwner should be registered");

        app.activate_current_row();
        app.shape_picker.open(Some(owner_index));
        app.shape_picker.search.list_state.select(Some(owner_index));
        app.apply_shape_selection();

        app.active_row_index = 2;
        app.activate_current_row();

        let picker = app
            .field_picker
            .as_ref()
            .expect("the Cow field should open its object picker");
        assert!(picker.choices.iter().any(|choice| matches!(
            choice,
            FieldPickerChoice::InvokeDefaultProducer { input_shape_name, .. }
                if input_shape_name == "DummyCowProducerRequest"
        )));
        assert!(matches!(
            picker.selected_choice(),
            Some(FieldPickerChoice::InvokeDefaultProducer { input_shape_name, .. })
                if input_shape_name == "DummyCowProducerRequest"
        ));
        assert_eq!(
            app.default_json_for_shape("DummyCowProducerRequest")
                .expect("unit request should support default construction"),
            serde_json::json!({})
        );
    }

    #[test]
    fn creating_field_object_keeps_owner_focused_until_field_activation() {
        let mut app = ObjectBrowserApp::default();
        let owner_choice = app
            .shape_choices
            .iter()
            .find(|choice| choice.label == "EntraGroupsForMemberRequest")
            .cloned()
            .expect("EntraGroupsForMemberRequest should be registered");
        let principal_choice = app
            .shape_choices
            .iter()
            .find(|choice| choice.label == "PrincipalId")
            .cloned()
            .expect("PrincipalId should be registered");
        let field_index = super::shape_fields_for_thing(owner_choice.thing)
            .iter()
            .position(|field| field.field_name == "principal_id")
            .expect("request should have a principal_id field");
        let user_variant_index = super::shape_variants_for_thing(principal_choice.thing)
            .iter()
            .position(|variant| variant.variant_name == "UserId")
            .expect("PrincipalId::UserId should be registered");

        let mut owner_slot = ObjectSlot::new(1);
        owner_slot.apply_shape_choice(&owner_choice);
        app.object_slots.push(owner_slot);
        app.active_slot_index = 0;
        app.active_row_index = app
            .focus_row_for_slot_target(1, SlotFocusTarget::FieldValue(field_index))
            .expect("principal_id field should be focusable");

        app.create_field_object(1, field_index, "PrincipalId", Some(user_variant_index));

        assert_eq!(app.current_slot_id(), Some(1));
        assert!(matches!(
            app.slot_field(1, field_index)
                .map(|field| field.value_state),
            Some(super::FieldValueState::Linked { slot_id: 2 })
        ));
        let value_spans = app
            .slot_display_rows(1)
            .into_iter()
            .find_map(|row| match row {
                super::SlotDisplayRow::Focusable {
                    target: SlotFocusTarget::FieldValue(index),
                    spans,
                } if index == field_index => Some(spans),
                _ => None,
            })
            .expect("principal_id value row should be rendered");
        assert_eq!(value_spans[1].style.fg, Some(ratatui::style::Color::Red));

        app.activate_field_value(field_index);
        assert_eq!(app.current_slot_id(), Some(2));
    }

    #[test]
    fn proxied_enum_slots_serialize_the_inner_payload() {
        let mut app = ObjectBrowserApp::default();
        let principal_choice = app
            .shape_choices
            .iter()
            .find(|choice| choice.label == "PrincipalId")
            .cloned()
            .expect("PrincipalId should be registered");
        let user_choice = app
            .shape_choices
            .iter()
            .find(|choice| choice.label == "EntraUserId")
            .cloned()
            .expect("EntraUserId should be registered");
        let user_variant_index = super::shape_variants_for_thing(principal_choice.thing)
            .iter()
            .position(|variant| variant.variant_name == "UserId")
            .expect("PrincipalId::UserId should be registered");

        let mut principal_slot = ObjectSlot::new(1);
        principal_slot.apply_shape_choice(&principal_choice);
        principal_slot
            .select_variant(user_variant_index)
            .expect("PrincipalId::UserId should be selectable");
        let mut user_slot = ObjectSlot::new_scalar(2, user_choice.label, user_choice.thing.shape);
        user_slot.value_state = super::SlotValueState::Building(SlotBody::Value {
            shape: user_choice.thing.shape,
            value: Some(serde_json::json!("22bd3607-20b4-41fc-bf14-000000000000")),
        });
        app.object_slots.push(principal_slot);
        app.object_slots.push(user_slot);
        app.set_field_link(1, 0, 2);

        assert_eq!(
            app.slot_json_value(1)
                .expect("principal id should serialize"),
            serde_json::json!("22bd3607-20b4-41fc-bf14-000000000000")
        );
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
        assert!(matches!(
            app.object_slots[0].value_state,
            super::SlotValueState::Building(SlotBody::Enum { .. })
        ));
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
        for variant_name in ["Default", "Id", "Alias"] {
            assert!(picker.choices.iter().any(|choice| matches!(
                choice,
                FieldPickerChoice::CreateNewVariant {
                    variant_name: candidate,
                    ..
                } if candidate == variant_name
            )));
        }

        let default_choice_index = picker
            .choices
            .iter()
            .position(|choice| {
                matches!(
                    choice,
                    FieldPickerChoice::CreateNewVariant { variant_name, .. }
                        if variant_name == "Default"
                )
            })
            .expect("Default variant choice should be present");
        app.field_picker
            .as_mut()
            .expect("field picker should remain open")
            .search
            .list_state
            .select(Some(default_choice_index));
        app.apply_field_picker_selection();

        let linked_slot_id = match app.slot_field(2, 0).map(|field| field.value_state) {
            Some(super::FieldValueState::Linked { slot_id }) => slot_id,
            other => panic!("expected linked Default variant, got {other:?}"),
        };
        let (_, variants, selected_variant) = app
            .slot_variant_picker_seed(linked_slot_id)
            .expect("linked AzureTenantArgument should retain enum state");
        assert_eq!(
            selected_variant.map(|index| variants[index].variant_name),
            Some("Default")
        );
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
        assert!(picker.choices.iter().any(|choice| matches!(
            choice,
            FieldPickerChoice::InvokeDefaultProducer { input_shape_name, .. }
                if input_shape_name == "AzureTenantIdResolveRequest"
        )));
        assert!(picker.choices.iter().any(|choice| matches!(
            choice,
            FieldPickerChoice::InvokeArbitraryProducer { input_shape_name, .. }
                if input_shape_name == "AzureTenantIdResolveRequest"
        )));
        let create_index = picker
            .choices
            .iter()
            .position(|choice| {
                matches!(
                    choice,
                    FieldPickerChoice::CreateProducer { input_shape_name, .. }
                        if input_shape_name == "AzureTenantIdResolveRequest"
                )
            })
            .expect("create producer choice should be selectable");
        let default_index = picker
            .choices
            .iter()
            .position(|choice| {
                matches!(
                    choice,
                    FieldPickerChoice::InvokeDefaultProducer { input_shape_name, .. }
                        if input_shape_name == "AzureTenantIdResolveRequest"
                )
            })
            .expect("default producer choice should be selectable");
        assert!(
            default_index < create_index,
            "default producer should precede create producer: {:?}",
            picker.labels
        );
        assert!(matches!(
            picker.selected_choice(),
            Some(FieldPickerChoice::InvokeDefaultProducer { input_shape_name, .. })
                if input_shape_name == "AzureTenantIdResolveRequest"
        ));
        assert_eq!(
            app.default_json_for_shape("AzureTenantIdResolveRequest")
                .expect("the request should support reflected default construction"),
            serde_json::json!({ "tenant": "Default" })
        );

        let arbitrary_request_choice = picker
            .choices
            .iter()
            .position(|choice| {
                matches!(
                    choice,
                    FieldPickerChoice::InvokeArbitraryProducer { input_shape_name, .. }
                        if input_shape_name == "AzureTenantIdResolveRequest"
                )
            })
            .expect("arbitrary request shortcut should be selectable");
        app.field_picker
            .as_mut()
            .expect("field picker should remain open")
            .search
            .list_state
            .select(Some(arbitrary_request_choice));
        app.apply_field_picker_selection();
        assert_eq!(app.mode, UiMode::ArbitrarySourcePicker);
        assert!(app.object_slots.iter().any(|slot| {
            slot.shape_name.as_deref() == Some("AzureTenantIdResolveRequest")
                && slot.created_for.is_some_and(|created_for| {
                    created_for.owner_slot_id == 1 && created_for.field_index == 0
                })
        }));
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
    fn materialized_request_cards_render_reflected_fields_without_a_builder() {
        let mut app = ObjectBrowserApp::default();
        let shape = app
            .shape_choices
            .iter()
            .find(|shape| shape.label == "AzureTenantIdResolveRequest")
            .cloned()
            .expect("AzureTenantIdResolveRequest should be registered");
        let value = serde_json::json!({ "tenant": "Default" });
        let mut slot = ObjectSlot::new(1);
        slot.apply_shape_choice(&shape);
        slot.value_state = super::SlotValueState::ResolvedValue {
            json: serde_json::to_string(&value).expect("json"),
            value,
        };
        app.object_slots.push(slot);

        assert!(app.slot_body(1).is_none());
        let fields = app.materialized_fields(1);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].info.field_name, "tenant");
        assert_eq!(fields[0].value, serde_json::json!("Default"));
        assert!(
            app.slot_focus_targets(1)
                .contains(&SlotFocusTarget::FieldValue(0))
        );

        let rendered = app
            .slot_display_rows(1)
            .iter()
            .filter_map(|row| match row {
                super::SlotDisplayRow::Static(line) => Some(
                    line.spans
                        .iter()
                        .map(|span| span.content.as_ref())
                        .collect::<String>(),
                ),
                super::SlotDisplayRow::Focusable { spans, .. } => Some(
                    spans
                        .iter()
                        .map(|span| span.content.as_ref())
                        .collect::<String>(),
                ),
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("--- fields ---"), "{rendered}");
        assert!(rendered.contains("tenant: \"Default\""), "{rendered}");
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

        assert_eq!(app.current_slot_id(), Some(1));
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
        assert_eq!(app.current_slot_id(), Some(2));
        assert_eq!(
            app.active_row_index,
            app.focus_row_for_slot_target(2, SlotFocusTarget::FieldValue(0))
                .expect("the request field should remain focused after cloning")
        );
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
        assert_eq!(app.current_slot_id(), Some(2));
        assert_eq!(
            app.active_row_index,
            app.focus_row_for_slot_target(2, SlotFocusTarget::FieldValue(0))
                .expect("the request field should remain focused after moving")
        );
    }

    #[test]
    fn move_or_clone_picker_supports_fuzzy_search() {
        let mut app = ObjectBrowserApp::default();
        app.open_link_action_picker(2, 0, 1);

        app.handle_link_action_picker_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        app.handle_link_action_picker_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));

        let picker = app
            .link_action_picker
            .as_ref()
            .expect("the move/clone picker should remain open");
        assert_eq!(picker.search.filtered_indices, vec![1]);
        assert_eq!(picker.selected_action(), Some(super::LinkAction::Clone));
    }

    #[test]
    fn picker_search_selects_the_top_match_after_each_query_edit() {
        let labels = vec!["alpha".to_string(), "beta alpha".to_string()];
        let mut search = super::PickerSearchState::new();
        search.reset(&labels, Some(1));
        assert_eq!(search.selected_filtered_index(), Some(1));

        search.handle_key(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            &labels,
        );

        assert_eq!(search.filtered_indices, vec![0, 1]);
        assert_eq!(search.list_state.selected(), Some(0));
        assert_eq!(search.selected_filtered_index(), Some(0));

        let mut picker = super::PartitionPickerState::new(
            super::PartitionPickerTarget::ShapeFilter { edit_index: None },
            labels,
        );
        picker.search.list_state.select(Some(1));
        picker.reset_selection_to_current();
        let mut app = ObjectBrowserApp {
            partition_picker: Some(picker),
            mode: UiMode::PartitionPicker,
            ..ObjectBrowserApp::default()
        };

        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));

        let picker = app
            .partition_picker
            .as_ref()
            .expect("partition picker should remain open");
        assert_eq!(picker.search.list_state.selected(), Some(0));
        assert_eq!(picker.selected_indices, BTreeSet::from([0]));
    }

    #[test]
    fn shift_semicolon_selects_add_then_shape_filter_can_be_confirmed() {
        let mut app = ObjectBrowserApp::default();
        for (id, shape_name) in [
            (1, "AzureTenantArgument"),
            (2, "AzureTenantIdResolveRequest"),
        ] {
            let shape = app
                .shape_choices
                .iter()
                .find(|shape| shape.label == shape_name)
                .cloned()
                .expect("shape should be registered");
            let mut slot = ObjectSlot::new(id);
            slot.apply_shape_choice(&shape);
            app.object_slots.push(slot);
        }

        app.handle_event(&Event::Key(KeyEvent::new(
            KeyCode::Char(':'),
            KeyModifiers::SHIFT,
        )));
        assert_eq!(app.mode, UiMode::Pool);
        assert_eq!(app.pool_surface, super::PoolSurface::Breadcrumbs);
        assert_eq!(
            app.active_breadcrumb_index,
            app.breadcrumb_count().saturating_sub(1)
        );
        app.handle_pool_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.mode, UiMode::FilterKindPicker);
        app.handle_filter_kind_picker_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.mode, UiMode::PartitionPicker);
        let request_index = app
            .partition_picker
            .as_ref()
            .expect("partition picker should open")
            .labels
            .iter()
            .position(|label| label == "AzureTenantIdResolveRequest")
            .expect("request shape should be listed");
        let picker = app.partition_picker.as_mut().expect("picker");
        picker.search.list_state.select(Some(request_index));
        picker.reset_selection_to_current();
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(app.mode, UiMode::Pool);
        assert_eq!(
            app.breadcrumb_filters
                .first()
                .and_then(|filter| match filter {
                    super::BreadcrumbFilter::Shape(filter) => {
                        Some(filter.included_shapes.clone())
                    }
                    super::BreadcrumbFilter::Value(_) | super::BreadcrumbFilter::SlotKind(_) =>
                        None,
                }),
            Some(BTreeSet::from(["AzureTenantIdResolveRequest".to_string()]))
        );
        assert_eq!(app.total_slot_count(), 2, "matching slot plus +New Slot");
        assert_eq!(app.pool_entry_at(0), Some(super::PoolEntry::RealSlot(2)));
        assert_eq!(app.pool_entry_at(1), Some(super::PoolEntry::NewSlot));
    }

    #[test]
    fn partition_picker_supports_range_and_bulk_selection() {
        let mut app = ObjectBrowserApp::default();
        app.partition_picker = Some(super::PartitionPickerState::new(
            super::PartitionPickerTarget::ShapeFilter { edit_index: None },
            vec!["A".into(), "B".into(), "C".into()],
        ));
        app.mode = UiMode::PartitionPicker;

        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
        let picker = app.partition_picker.as_ref().expect("picker");
        assert_eq!(picker.selected_indices, BTreeSet::from([1, 2]));

        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(
            app.partition_picker
                .as_ref()
                .expect("picker")
                .included_indices,
            BTreeSet::from([1, 2])
        );
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
        assert_eq!(
            app.partition_picker
                .as_ref()
                .expect("picker")
                .selected_indices,
            BTreeSet::from([0, 1, 2])
        );
    }

    #[test]
    fn partition_picker_supports_page_navigation() {
        let mut app = ObjectBrowserApp::default();
        app.partition_picker = Some(super::PartitionPickerState::new(
            super::PartitionPickerTarget::ShapeFilter { edit_index: None },
            (0..25).map(|index| format!("Shape {index}")).collect(),
        ));
        app.mode = UiMode::PartitionPicker;

        app.handle_partition_picker_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        assert_eq!(
            app.partition_picker
                .as_ref()
                .expect("picker")
                .current_index(),
            Some(10)
        );
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        assert_eq!(
            app.partition_picker
                .as_ref()
                .expect("picker")
                .current_index(),
            Some(24)
        );
        app.handle_partition_picker_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::SHIFT));
        let picker = app.partition_picker.as_ref().expect("picker");
        assert_eq!(picker.current_index(), Some(14));
        assert_eq!(picker.selected_indices, (14..=24).collect::<BTreeSet<_>>());
    }

    #[test]
    fn empty_shape_filter_submission_includes_the_active_item() {
        let mut app = ObjectBrowserApp::default();
        app.partition_picker = Some(super::PartitionPickerState::new(
            super::PartitionPickerTarget::ShapeFilter { edit_index: None },
            vec!["A".to_string(), "B".to_string()],
        ));
        app.mode = UiMode::PartitionPicker;

        app.handle_partition_picker_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(matches!(
            app.breadcrumb_filters.as_slice(),
            [super::BreadcrumbFilter::Shape(filter)]
                if filter.included_shapes == BTreeSet::from(["A".to_string()])
        ));
    }

    #[test]
    fn delete_removes_the_selected_filter_breadcrumb() {
        let mut app = ObjectBrowserApp::default();
        app.breadcrumb_filters = vec![
            super::BreadcrumbFilter::Shape(super::ShapeFilterView {
                included_shapes: BTreeSet::from(["A".to_string()]),
            }),
            super::BreadcrumbFilter::Shape(super::ShapeFilterView {
                included_shapes: BTreeSet::from(["B".to_string()]),
            }),
        ];
        app.pool_surface = super::PoolSurface::Breadcrumbs;
        app.active_breadcrumb_index = 1;

        app.handle_pool_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));

        assert_eq!(app.breadcrumb_filters.len(), 1);
        assert!(matches!(
            app.breadcrumb_filters.first(),
            Some(super::BreadcrumbFilter::Shape(filter))
                if filter.included_shapes.contains("B")
        ));
    }

    #[test]
    fn tabs_preserve_navigation_and_filters() {
        let mut app = ObjectBrowserApp::default();
        app.breadcrumb_filters = vec![super::BreadcrumbFilter::Shape(super::ShapeFilterView {
            included_shapes: BTreeSet::from(["RoleAssignment".to_string()]),
        })];
        app.pool_surface = super::PoolSurface::Breadcrumbs;
        app.active_breadcrumb_index = 1;

        app.handle_event(&Event::Key(KeyEvent::new(
            KeyCode::Char('}'),
            KeyModifiers::SHIFT,
        )));

        assert_eq!(app.active_tab_index, 1);
        assert_eq!(app.tabs.len(), 2);
        assert!(app.breadcrumb_filters.is_empty());
        assert_eq!(app.breadcrumb_count(), 2);

        app.handle_event(&Event::Key(KeyEvent::new(
            KeyCode::Char('{'),
            KeyModifiers::SHIFT,
        )));
        assert_eq!(app.active_tab_index, 0);
        assert_eq!(app.breadcrumb_filters.len(), 1);

        app.handle_event(&Event::Key(KeyEvent::new(
            KeyCode::Char('}'),
            KeyModifiers::SHIFT,
        )));
        assert_eq!(app.active_tab_index, 1);
        assert_eq!(app.tabs.len(), 2);
        assert!(app.breadcrumb_filters.is_empty());
    }

    #[test]
    fn value_filter_editor_supports_page_and_edge_navigation() {
        let mut app = ObjectBrowserApp::default();
        app.open_value_filter_editor(
            None,
            super::ValueFilterView {
                field_shape: "*".to_string(),
                field_name: "*".to_string(),
                operator: super::ValueFilterOperator::Equals,
                value: String::new(),
            },
        );

        app.handle_value_filter_editor_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        assert_eq!(
            app.value_filter_editor.as_ref().expect("editor").active_row,
            5
        );
        app.handle_value_filter_editor_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(
            app.value_filter_editor.as_ref().expect("editor").active_row,
            0
        );
        app.handle_value_filter_editor_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(
            app.value_filter_editor.as_ref().expect("editor").active_row,
            5
        );
        app.handle_value_filter_editor_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
        assert_eq!(
            app.value_filter_editor.as_ref().expect("editor").active_row,
            0
        );
    }

    #[test]
    fn large_value_choice_picker_uses_the_nucleo_worker() {
        let labels = (0..6_000)
            .map(|index| format!("value-{index:04}"))
            .collect::<Vec<_>>();
        let mut picker = super::ValueFilterChoicePickerState::new(
            super::ValueFilterChoiceTarget::ExistingValue,
            labels,
        );
        assert!(picker.worker.is_some());
        assert!(
            picker.search.filtered_indices.len()
                <= super::ValueFilterChoicePickerState::MAX_VISIBLE_MATCHES
        );

        for character in "value-5999".chars() {
            picker.handle_key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
        }
        for _ in 0..20 {
            picker.refresh_worker();
            if picker
                .search
                .selected_filtered_index()
                .is_some_and(|index| picker.labels[index] == "value-5999")
            {
                break;
            }
        }
        assert!(
            picker
                .search
                .filtered_indices
                .iter()
                .any(|index| picker.labels[*index] == "value-5999")
        );
    }

    #[test]
    fn shape_filter_breadcrumbs_are_additive_and_editable() {
        let mut app = ObjectBrowserApp::default();
        let shape = app
            .shape_choices
            .iter()
            .find(|shape| shape.label == "RoleAssignment")
            .cloned()
            .expect("RoleAssignment should be registered");
        let mut slot = ObjectSlot::new(1);
        slot.apply_shape_choice(&shape);
        app.object_slots.push(slot);
        app.breadcrumb_filters = vec![super::BreadcrumbFilter::Shape(super::ShapeFilterView {
            included_shapes: BTreeSet::from(["RoleAssignment".to_string()]),
        })];
        app.open_shape_filter_picker(None);
        let picker = app.partition_picker.as_mut().expect("new shape filter");
        let role_assignment_index = picker
            .labels
            .iter()
            .position(|label| label == "RoleAssignment")
            .expect("RoleAssignment should be available");
        picker.search.list_state.select(Some(role_assignment_index));
        picker.reset_selection_to_current();
        picker.move_selected(true);
        app.apply_partition_picker_selection();
        assert_eq!(app.breadcrumb_filters.len(), 2);

        app.pool_surface = super::PoolSurface::Breadcrumbs;
        app.active_breadcrumb_index = app.projection_stack.len() + 1;

        app.activate_current_breadcrumb();

        assert_eq!(app.mode, UiMode::PartitionPicker);
        let picker = app.partition_picker.as_ref().expect("shape editor");
        assert_eq!(
            picker.target,
            super::PartitionPickerTarget::ShapeFilter {
                edit_index: Some(0)
            }
        );
        let role_assignment_index = picker
            .labels
            .iter()
            .position(|label| label == "RoleAssignment")
            .expect("selected shape should remain available while editing");
        assert!(picker.included_indices.contains(&role_assignment_index));
        assert_eq!(app.breadcrumb_filters.len(), 2);
    }

    #[test]
    fn add_breadcrumb_can_open_the_value_filter_editor() {
        let mut app = ObjectBrowserApp::default();
        app.pool_surface = super::PoolSurface::Breadcrumbs;
        app.active_breadcrumb_index = app.breadcrumb_count().saturating_sub(1);

        app.activate_current_breadcrumb();
        assert_eq!(app.mode, UiMode::FilterKindPicker);
        app.handle_filter_kind_picker_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.handle_filter_kind_picker_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert_eq!(app.mode, UiMode::ValueFilterEditor);
        let editor = app.value_filter_editor.as_ref().expect("value editor");
        assert_eq!(editor.draft.field_shape, "*");
        assert_eq!(editor.draft.field_name, "*");
        assert_eq!(editor.source, super::ValueFilterSource::Existing);
    }

    #[test]
    fn value_filter_picklists_resolve_function_output_list_shapes() {
        let mut app = ObjectBrowserApp::default();
        let value = serde_json::json!([
            {
                "displayName": "Ada",
                "userPrincipalName": "ada@example.com"
            },
            {
                "displayName": "Dominic",
                "userPrincipalName": "dominic.phillips@agr.gc.ca"
            }
        ]);
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some("List<EntraUser>".to_string()),
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        });
        app.activate_runtime_value(1);
        app.open_value_filter_editor(
            None,
            super::ValueFilterView {
                field_shape: "*".to_string(),
                field_name: "*".to_string(),
                operator: super::ValueFilterOperator::Equals,
                value: String::new(),
            },
        );

        app.open_value_filter_choice(super::ValueFilterChoiceTarget::FieldShape);
        let shape_picker = app
            .value_filter_choice_picker
            .as_ref()
            .expect("field shape picker");
        assert!(
            shape_picker.labels.iter().any(|label| label != "*"),
            "{:?}",
            shape_picker.labels
        );

        app.value_filter_choice_picker = None;
        app.mode = UiMode::ValueFilterEditor;
        app.open_value_filter_choice(super::ValueFilterChoiceTarget::FieldName);
        let name_picker = app
            .value_filter_choice_picker
            .as_ref()
            .expect("field name picker");
        assert!(
            name_picker.labels.iter().any(|label| label != "*"),
            "{:?}",
            name_picker.labels
        );

        app.value_filter_choice_picker = None;
        app.value_filter_editor = None;
        app.mode = UiMode::Pool;
        app.breadcrumb_filters = vec![super::BreadcrumbFilter::Value(super::ValueFilterView {
            field_shape: "*".to_string(),
            field_name: "userPrincipalName".to_string(),
            operator: super::ValueFilterOperator::Equals,
            value: "dominic.phillips@agr.gc.ca".to_string(),
        })];
        app.projection_cache.borrow_mut().clear();

        assert_eq!(app.total_slot_count(), 3);
        assert!(matches!(
            app.pool_entry_at(0),
            Some(super::PoolEntry::Projection(super::ProjectionSlot { path, .. }))
                if path == vec![super::JsonPathSegment::Index(1)]
        ));
        assert!((0..app.total_slot_count()).any(|index| matches!(
            app.pool_entry_at(index),
            Some(super::PoolEntry::Projection(super::ProjectionSlot { path, .. }))
                if path == vec![
                    super::JsonPathSegment::Index(1),
                    super::JsonPathSegment::Field("displayName".to_string())
                ]
        )));
        for index in 0..app.total_slot_count() {
            let _ = app.pool_entry_at(index);
        }
        let cached_path_count = app.projection_cache.borrow().filtered_paths.len();
        assert!(cached_path_count > 0);
        for index in 0..app.total_slot_count() {
            let _ = app.pool_entry_at(index);
        }
        assert_eq!(
            app.projection_cache.borrow().filtered_paths.len(),
            cached_path_count,
            "revisiting visible filtered slots should reuse indexed paths"
        );
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
        assert_eq!(app.current_slot_id(), Some(1));
        assert_eq!(
            app.slot_focus_targets(1).get(app.active_row_index),
            Some(&SlotFocusTarget::Result(0))
        );
        assert!(matches!(
            app.slot_by_id(result_slot_id).map(|slot| &slot.value_state),
            Some(super::SlotValueState::Pending(_))
        ));

        tokio::task::yield_now().await;
        app.advance_pending_invocations();

        let resolved_json = app
            .slot_by_id(result_slot_id)
            .and_then(|slot| match &slot.value_state {
                super::SlotValueState::ResolvedValue { json, .. } => Some(json.clone()),
                _ => None,
            })
            .expect("result slot should resolve");
        assert!(resolved_json.contains("\"message\":\"done\""));
    }

    #[tokio::test]
    async fn pending_producer_result_is_linked_into_its_created_field() {
        let mut app = ObjectBrowserApp::default();
        let owner_choice = app
            .shape_choices
            .iter()
            .find(|shape| shape.label == "EntraGroupsForMemberRequest")
            .cloned()
            .expect("EntraGroupsForMemberRequest should be registered");
        let field_index = super::shape_fields_for_thing(owner_choice.thing)
            .iter()
            .position(|field| field.field_name == "tenant_id")
            .expect("request should have a tenant_id field");
        let source_choice = app
            .shape_choices
            .iter()
            .find(|shape| shape.label == "DummyInvokeRequest")
            .cloned()
            .expect("DummyInvokeRequest should be registered");

        let mut owner_slot = ObjectSlot::new(1);
        owner_slot.apply_shape_choice(&owner_choice);
        let mut source_slot = ObjectSlot::new(2);
        source_slot.apply_shape_choice(&source_choice);
        source_slot.created_for = Some(super::SlotCreatedFor {
            owner_slot_id: 1,
            field_index,
            field_name: "tenant_id",
        });
        app.object_slots.push(owner_slot);
        app.object_slots.push(source_slot);

        let function = app
            .applicable_functions_for_slot(2)
            .into_iter()
            .find(|function| describe_shape(function.output_shape) == "DummyInvokeOutput")
            .expect("DummyInvokeOutput constructor should be available");
        app.invoke_registered_function(2, function);
        assert_eq!(app.current_slot_id(), Some(1));
        let result_slot_id = app
            .slot_by_id(2)
            .and_then(|slot| slot.result_slot_ids.first().copied())
            .expect("invocation should create a result slot");

        assert!(matches!(
            app.slot_field(1, field_index).map(|field| field.value_state),
            Some(super::FieldValueState::Linked { slot_id }) if slot_id == result_slot_id
        ));
        let rendered = app
            .slot_display_rows(1)
            .into_iter()
            .find_map(|row| match row {
                super::SlotDisplayRow::Focusable {
                    target: SlotFocusTarget::FieldValue(index),
                    spans,
                } if index == field_index => Some(
                    spans
                        .iter()
                        .map(|span| span.content.as_ref())
                        .collect::<String>(),
                ),
                _ => None,
            })
            .expect("tenant_id value row should be rendered");
        assert_eq!(rendered, "tenant_id: pending");

        tokio::task::yield_now().await;
        app.advance_pending_invocations();

        let rendered = app
            .slot_display_rows(1)
            .into_iter()
            .find_map(|row| match row {
                super::SlotDisplayRow::Focusable {
                    target: SlotFocusTarget::FieldValue(index),
                    spans,
                } if index == field_index => Some(
                    spans
                        .iter()
                        .map(|span| span.content.as_ref())
                        .collect::<String>(),
                ),
                _ => None,
            })
            .expect("tenant_id value row should still be rendered");
        assert_eq!(rendered, format!("tenant_id: slot {result_slot_id}"));
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
            slot.value_state = super::SlotValueState::ResolvedValue {
                json: "[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]"
                    .to_string(),
                value: serde_json::from_str(
                    "[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]",
                )
                .unwrap(),
            };
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
        assert_eq!(app.current_slot_id(), Some(1));
        assert_eq!(
            app.slot_focus_targets(1).get(app.active_row_index),
            Some(&SlotFocusTarget::Result(0))
        );
        let resolved_value = app
            .slot_by_id(result_slot_id)
            .and_then(|slot| match &slot.value_state {
                super::SlotValueState::ResolvedValue { value, .. } => Some(value.clone()),
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
        assert_eq!(
            app.request_output_shape_names(1),
            vec!["AzureTenantId".to_string()]
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
            .position(|shape| shape.label.contains("AzureTenantIdResolveRequest"))
            .expect("AzureTenantIdResolveRequest should be registered");
        app.shape_picker.open(Some(request_index));
        app.shape_picker
            .search
            .list_state
            .select(Some(request_index));
        app.apply_shape_selection();

        app.activate_slot_action(1, super::SlotAction::InvokeArbitrary);
        assert_eq!(app.mode, UiMode::ArbitrarySourcePicker);
        assert!(matches!(
            app.arbitrary_source_picker
                .as_ref()
                .and_then(|picker| picker.selected_choice()),
            Some(super::ArbitrarySourceChoice::CreateNew)
        ));
        app.apply_arbitrary_source_picker_selection();

        let arbitrary_slot = app
            .object_slots
            .iter()
            .find(|slot| slot.shape_name.as_deref() == Some("ArbitraryBytes"))
            .expect("the shortcut should create an explicit ArbitraryBytes slot");
        let arbitrary_slot_id = arbitrary_slot.id;
        assert!(matches!(
            arbitrary_slot.value_state,
            super::SlotValueState::ResolvedValue { .. }
        ));
        assert!(
            app.slot_body(arbitrary_slot_id).is_none(),
            "a materialized slot must not retain builder state"
        );
        let materialized_fields = app.materialized_fields(arbitrary_slot_id);
        assert_eq!(materialized_fields.len(), 1);
        assert_eq!(materialized_fields[0].info.field_name, "0");
        assert!(materialized_fields[0].value.is_array());
        let result_slot_id = arbitrary_slot
            .result_slot_ids
            .first()
            .copied()
            .unwrap_or_else(|| {
                panic!(
                    "arbitrary source should create a result slot: {}",
                    app.status_message
                )
            });
        assert_eq!(
            app.slot_by_id(result_slot_id)
                .and_then(|slot| slot.produced_by_slot_id),
            Some(arbitrary_slot.id)
        );
        assert_eq!(
            arbitrary_slot.produced_by_slot_id,
            Some(1),
            "the generated ArbitraryBytes slot should link back to its request"
        );
        assert_eq!(
            app.slot_by_id(1)
                .and_then(|slot| slot.result_slot_ids.first().copied()),
            Some(arbitrary_slot.id),
            "the request should link forward to its generated ArbitraryBytes slot"
        );
        let resolved_value = app
            .slot_by_id(result_slot_id)
            .and_then(|slot| match &slot.value_state {
                super::SlotValueState::ResolvedValue { value, .. } => Some(value.clone()),
                _ => None,
            })
            .expect("fake result slot should resolve immediately");
        assert_ne!(
            resolved_value,
            serde_json::Value::String("00000000-0000-4000-8000-000000000000".to_string())
        );
    }

    #[test]
    fn invoke_arbitrary_can_reuse_an_existing_arbitrary_bytes_slot() {
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

        let source_slot_id = app
            .create_random_arbitrary_bytes_slot()
            .expect("ArbitraryBytes should be constructible");
        app.activate_slot_action(1, super::SlotAction::InvokeArbitrary);
        assert!(matches!(
            app.arbitrary_source_picker
                .as_ref()
                .and_then(|picker| picker.selected_choice()),
            Some(super::ArbitrarySourceChoice::ExistingSlot { slot_id })
                if slot_id == source_slot_id
        ));
        app.apply_arbitrary_source_picker_selection();

        assert_eq!(
            app.object_slots
                .iter()
                .filter(|slot| slot.shape_name.as_deref() == Some("ArbitraryBytes"))
                .count(),
            1
        );
        assert_eq!(
            app.slot_by_id(source_slot_id)
                .map(|slot| slot.result_slot_ids.len()),
            Some(1)
        );
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

        let mail_index = match app.slot_by_id(1).map(|slot| &slot.value_state) {
            Some(super::SlotValueState::Building(SlotBody::Struct { fields })) => fields
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
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
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
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
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
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
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
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
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

        app.pool_surface = super::PoolSurface::Breadcrumbs;
        app.handle_event(&Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(!app.show_hotkey_help);
        assert_eq!(app.pool_surface, super::PoolSurface::Breadcrumbs);
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
    fn shifted_home_end_and_page_keys_navigate_across_slots() {
        let mut app = ObjectBrowserApp::default();
        app.last_visible_slot_count = 3;

        for _ in 0..5 {
            app.append_slot();
        }

        app.active_slot_index = 2;
        app.handle_pool_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::SHIFT));
        assert_eq!(app.active_slot_index, 4);

        app.handle_pool_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::SHIFT));
        assert_eq!(app.active_slot_index, 2);

        app.handle_pool_key(KeyEvent::new(KeyCode::End, KeyModifiers::SHIFT));
        assert_eq!(
            app.active_slot_index,
            app.total_slot_count().saturating_sub(1)
        );

        app.handle_pool_key(KeyEvent::new(KeyCode::Home, KeyModifiers::SHIFT));
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
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
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

    #[test]
    fn projected_slots_support_type_search_and_value_filters() {
        let mut app = ObjectBrowserApp::default();
        let principal_id = "11111111-2222-4333-8444-555555555555";
        let value = serde_json::json!({
            "id": "/providers/Microsoft.Authorization/roleAssignments/a",
            "scope": "/subscriptions/s",
            "role_definition_id": "/providers/Microsoft.Authorization/roleDefinitions/r",
            "principal_id": principal_id,
        });
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some("RoleAssignment".to_string()),
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        });
        app.activate_runtime_value(1);

        app.start_slot_search(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));

        assert_eq!(app.mode, UiMode::SlotSearch);
        let search = app
            .projection_search
            .as_ref()
            .expect("projection search should start");
        assert!(!search.filtered_matches.is_empty());
        assert!(search.filtered_matches.iter().any(|matched| {
            super::spans_plain_text(&matched.spans)
                .to_lowercase()
                .contains("principal")
        }));

        app.cancel_slot_search();
        app.breadcrumb_filters = vec![super::BreadcrumbFilter::Value(super::ValueFilterView {
            field_shape: "PrincipalId".to_string(),
            field_name: "*".to_string(),
            operator: super::ValueFilterOperator::Equals,
            value: principal_id.to_string(),
        })];
        app.projection_cache.borrow_mut().clear();

        assert_eq!(app.total_slot_count(), 5);
        assert!(matches!(
            app.pool_entry_at(0),
            Some(super::PoolEntry::Projection(super::ProjectionSlot { path, .. }))
                if path.is_empty()
        ));
        assert!((0..app.total_slot_count()).any(|index| matches!(
            app.pool_entry_at(index),
            Some(super::PoolEntry::Projection(super::ProjectionSlot { path, .. }))
                if path == vec![super::JsonPathSegment::Field("principal_id".to_string())]
        )));
        assert_eq!(
            app.existing_value_filter_choices("PrincipalId", "*"),
            vec![principal_id.to_string()]
        );
    }

    #[test]
    fn shape_filter_discovery_and_indexing_stay_virtual_for_large_projections() {
        let mut app = ObjectBrowserApp::default();
        let root_shape_name = app
            .shape_choices
            .iter()
            .find(|shape| shape.label == "RoleDefinitionsAndAssignments")
            .map(|shape| shape.label.clone())
            .expect("the role collection shape should be registered");
        let role_assignments = (0..10_000)
            .map(|index| {
                (
                    format!("assignment-{index:05}"),
                    serde_json::json!({ "id": format!("assignment-{index:05}") }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let value = serde_json::json!({
            "role_assignments": role_assignments,
            "role_definitions": {},
        });
        app.object_slots.push(super::ObjectSlot {
            id: 1,
            name: None,
            kind: super::SlotKind::Owned,
            shape_name: Some(root_shape_name),
            value_state: super::SlotValueState::ResolvedValue {
                json: serde_json::to_string(&value).expect("json"),
                value,
            },
            result_slot_ids: Vec::new(),
            created_for: None,
            produced_by_slot_id: None,
            display_cache: None,
        });
        app.activate_runtime_value(1);
        app.projection_cache.borrow_mut().clear();

        app.open_shape_filter_picker(None);

        let picker = app
            .partition_picker
            .as_ref()
            .expect("shape filter picker should open");
        assert!(picker.labels.iter().any(|label| label == "RoleAssignment"));
        assert!(
            !picker
                .labels
                .iter()
                .any(|label| label.starts_with("AzureApplicationGateway")),
            "string-proxied IDs must not expose unrelated enum internals: {:?}",
            picker.labels
        );
        assert!(
            app.projection_cache.borrow().descendant_counts.is_empty(),
            "shape discovery should inspect Facet shapes, not flatten values"
        );

        app.partition_picker = None;
        app.mode = UiMode::Pool;
        app.breadcrumb_filters = vec![super::BreadcrumbFilter::Shape(super::ShapeFilterView {
            included_shapes: BTreeSet::from(["RoleAssignment".to_string()]),
        })];
        app.projection_cache.borrow_mut().clear();

        assert_eq!(app.total_slot_count(), 10_000);
        assert!(
            app.projection_cache
                .borrow()
                .filtered_descendant_counts
                .len()
                <= 4,
            "a homogeneous map should cache containers, not all 10,000 entries"
        );
        assert!(matches!(
            app.pool_entry_at(9_999),
            Some(super::PoolEntry::Projection(super::ProjectionSlot { path, .. }))
                if path == vec![
                    super::JsonPathSegment::Field("role_assignments".to_string()),
                    super::JsonPathSegment::Key("assignment-09999".to_string()),
                ]
        ));
    }
}
