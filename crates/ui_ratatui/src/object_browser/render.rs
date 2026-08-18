use crate::object_explorer::CardAddress;
use crate::object_explorer::CardRowContent;
use crate::object_explorer::CardRowKey;
use crate::object_explorer::CardRowSnapshot;
use crate::object_explorer::CardSnapshot;
use crate::object_explorer::CardWindow;
use crate::object_explorer::FieldBindingSnapshot;
use crate::object_explorer::QueryTotal;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Margin;
use ratatui::layout::Rect;
use ratatui::prelude::Color;
use ratatui::prelude::Modifier;
use ratatui::prelude::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Scrollbar;
use ratatui::widgets::ScrollbarOrientation;
use ratatui::widgets::ScrollbarState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CardLayoutAxis {
    Horizontal,
    Vertical,
}

/// Pure Ratatui adapter for a bounded engine-owned card snapshot window.
///
/// It cannot inspect Arena, RuntimeValue, Facet values, or an unbounded result
/// stream. Consequently, one draw performs work proportional only to the
/// snapshots already admitted by the engine's frame budget.
pub(crate) struct CardWindowRenderer<'window> {
    window: &'window CardWindow,
    selected_card: &'window CardAddress,
    selected_row: Option<&'window CardRowKey>,
    axis: CardLayoutAxis,
    focused_card_fill: bool,
    card_breadth: u16,
    query_total: QueryTotal,
    visible_rows: Option<&'window [CardRowKey]>,
}

impl<'window> CardWindowRenderer<'window> {
    pub(crate) const fn new(
        window: &'window CardWindow,
        selected_card: &'window CardAddress,
        selected_row: Option<&'window CardRowKey>,
        axis: CardLayoutAxis,
        focused_card_fill: bool,
        card_breadth: u16,
    ) -> Self {
        Self {
            window,
            selected_card,
            selected_row,
            axis,
            focused_card_fill,
            card_breadth,
            query_total: QueryTotal::Unknown,
            visible_rows: None,
        }
    }

    pub(crate) const fn with_query_total(mut self, query_total: QueryTotal) -> Self {
        self.query_total = query_total;
        self
    }

    pub(crate) const fn with_visible_rows(
        mut self,
        visible_rows: Option<&'window [CardRowKey]>,
    ) -> Self {
        self.visible_rows = visible_rows;
        self
    }

    pub(crate) fn draw(&self, frame: &mut Frame<'_>, area: Rect) {
        if area.is_empty() || self.window.cards().is_empty() {
            return;
        }
        let show_card_scrollbar = self.axis == CardLayoutAxis::Horizontal
            && !self.focused_card_fill
            && (self.window.has_before() || self.window.has_after())
            && area.height >= 2;
        let (cards_area, card_scrollbar_area) = if show_card_scrollbar {
            let sections =
                Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
            (sections[0], Some(sections[1]))
        } else {
            (area, None)
        };
        let (cards, visible_start) = self.visible_cards(cards_area);
        if cards.is_empty() {
            return;
        }
        let constraints = vec![Constraint::Fill(1); cards.len()];
        let areas = match self.axis {
            CardLayoutAxis::Horizontal => Layout::horizontal(constraints).split(cards_area),
            CardLayoutAxis::Vertical => Layout::vertical(constraints).split(cards_area),
        };
        for (card, area) in cards.iter().copied().zip(areas.iter().copied()) {
            self.draw_card(frame, area, card);
        }
        if let Some(scrollbar_area) = card_scrollbar_area {
            let position = self.window.start_ordinal().saturating_add(visible_start);
            let has_new_slot = cards
                .iter()
                .any(|card| card.address() == &CardAddress::NewSlot);
            let minimum_total = position
                .saturating_add(cards.len())
                .saturating_add(usize::from(self.window.has_after()));
            let total_items = match self.query_total {
                QueryTotal::Exact(total) => total.saturating_add(usize::from(has_new_slot)),
                QueryTotal::Scanning(progress) => progress.matched,
                QueryTotal::Unknown => 0,
            }
            .max(minimum_total);
            let scroll_positions = total_items.saturating_sub(cards.len()).saturating_add(1);
            let mut state = ScrollbarState::new(scroll_positions)
                .position(position)
                .viewport_content_length(cards.len());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom).thumb_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
        }
    }

    fn visible_cards(&self, area: Rect) -> (Vec<&CardSnapshot>, usize) {
        if self.focused_card_fill {
            return (
                self.window
                    .cards()
                    .iter()
                    .find(|card| card.address() == self.selected_card)
                    .into_iter()
                    .collect(),
                0,
            );
        }

        let cards = self.window.cards();
        let main_axis = match self.axis {
            CardLayoutAxis::Horizontal => area.width,
            CardLayoutAxis::Vertical => area.height,
        };
        let capacity = (main_axis as usize / self.card_breadth.max(1) as usize).max(1);
        if cards.len() <= capacity {
            return (cards.iter().collect(), 0);
        }

        let selected = cards
            .iter()
            .position(|card| card.address() == self.selected_card);
        let start = selected
            .map(|index| index.saturating_sub(capacity - 1))
            .unwrap_or_default()
            .min(cards.len() - capacity);
        (cards[start..start + capacity].iter().collect(), start)
    }

    fn draw_card(&self, frame: &mut Frame<'_>, area: Rect, card: &CardSnapshot) {
        let selected = card.address() == self.selected_card;
        let card_area = if area.height >= 3 {
            let sections = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);
            if selected {
                let marker_style = Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD);
                frame.render_widget(
                    Paragraph::new(Span::styled("vvvvv", marker_style))
                        .alignment(Alignment::Center),
                    sections[0],
                );
                frame.render_widget(
                    Paragraph::new(Span::styled("^^^^^", marker_style))
                        .alignment(Alignment::Center),
                    sections[2],
                );
            }
            sections[1]
        } else {
            area
        };
        let title = match card.address() {
            CardAddress::NewSlot => "new slot".to_owned(),
            CardAddress::Value(_) => {
                let ownership = if card.owned_slot().is_some() {
                    "owned"
                } else {
                    "projection"
                };
                format!("{} [{ownership}]", display_address(card.address()))
            }
        };
        let block =
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(card_border_color(card)).add_modifier(
                    if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ));
        let (lines, focused_line, selected_line) = if card.address() == &CardAddress::NewSlot {
            (vec![new_slot_line(selected)], None, None)
        } else {
            let mut relationship_index = 0;
            let mut lines = Vec::new();
            let mut focused_line = None;
            let mut selected_line = None;
            for row in card.rows() {
                if selected
                    && self
                        .visible_rows
                        .is_some_and(|visible| !visible.contains(row.key()))
                {
                    continue;
                }
                let accent = matches!(
                    row.key(),
                    CardRowKey::Field(_) | CardRowKey::Element(_) | CardRowKey::MapValue(_)
                )
                .then(|| {
                    let color = field_group_color(relationship_index);
                    relationship_index += 1;
                    color
                });
                let row_selected = selected && self.selected_row == Some(row.key());
                if let (Some(type_name), Some(value)) = (row.type_name(), row.value_display()) {
                    if row_selected {
                        selected_line = Some(lines.len());
                    }
                    lines.push(typed_row_line(row, type_name, row_selected, accent));
                    lines.push(value_display_line(value, row_selected, accent));
                    if row_selected {
                        focused_line = Some(lines.len().saturating_sub(1));
                    }
                } else {
                    if let Some(type_name) = row.type_name() {
                        lines.push(type_line(type_name, accent.unwrap_or(Color::White)));
                    }
                    if row_selected {
                        selected_line = Some(lines.len());
                        focused_line = Some(lines.len());
                    }
                    lines.push(self.render_row(row, selected, accent));
                }
            }
            if lines.is_empty() && self.visible_rows.is_some() {
                lines.push(Line::from(Span::styled(
                    "  No matching rows",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )));
            }
            if !card.relationships_complete() {
                lines.push(Line::from(Span::styled(
                    "  … more relationships",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )));
            }
            (lines, focused_line, selected_line)
        };
        let line_count = lines.len();
        let viewport_height = block.inner(card_area).height as usize;
        let max_offset = line_count.saturating_sub(viewport_height);
        let scroll_offset = focused_line
            .map(|line| line.saturating_sub(viewport_height.saturating_sub(1)))
            .unwrap_or_default()
            .min(max_offset);
        frame.render_widget(
            Paragraph::new(lines)
                .scroll((scroll_offset.min(u16::MAX as usize) as u16, 0))
                .block(block),
            card_area,
        );
        if selected
            && let Some(line) = selected_line
            && line >= scroll_offset
            && line.saturating_sub(scroll_offset) < viewport_height
        {
            let marker_area = Rect::new(
                card_area.left(),
                card_area
                    .top()
                    .saturating_add(1)
                    .saturating_add((line - scroll_offset) as u16),
                1,
                1,
            );
            frame.render_widget(
                Paragraph::new(Span::styled(">", row_marker_style())),
                marker_area,
            );
        }
        if selected && viewport_height > 0 && line_count > viewport_height {
            let scrollbar_area = card_area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            });
            let scroll_positions = line_count.saturating_sub(viewport_height).saturating_add(1);
            let mut state = ScrollbarState::new(scroll_positions)
                .position(scroll_offset)
                .viewport_content_length(viewport_height);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .thumb_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_stateful_widget(scrollbar, scrollbar_area, &mut state);
        }
    }

    fn render_row(
        &self,
        row: &CardRowSnapshot,
        card_selected: bool,
        accent: Option<Color>,
    ) -> Line<'static> {
        let row_selected = card_selected && self.selected_row == Some(row.key());
        let accent = accent.unwrap_or(Color::White);
        let mut spans = Vec::new();
        spans.push(Span::styled(
            row.label().to_owned(),
            row_label_style(row.key(), accent),
        ));
        spans.push(Span::styled(": ", Style::default().fg(Color::DarkGray)));
        spans.extend(content_spans(row, accent));
        Line::from(spans).style(if row_selected {
            Style::default().bg(Color::Blue)
        } else {
            Style::default()
        })
    }
}

fn type_line(type_name: &str, accent: Color) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "type ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        ),
        Span::styled(
            type_name.to_owned(),
            Style::default().fg(accent).add_modifier(Modifier::DIM),
        ),
    ])
}

fn typed_row_line(
    row: &CardRowSnapshot,
    type_name: &str,
    row_selected: bool,
    accent: Option<Color>,
) -> Line<'static> {
    let accent = accent.unwrap_or(Color::White);
    Line::from(vec![
        Span::styled(row.label().to_owned(), row_label_style(row.key(), accent)),
        Span::styled(": ", Style::default().fg(Color::DarkGray)),
        Span::styled(type_name.to_owned(), Style::default().fg(accent)),
    ])
    .style(if row_selected {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    })
}

fn value_display_line(value: &str, row_selected: bool, accent: Option<Color>) -> Line<'static> {
    Line::from(vec![Span::styled(
        value.to_owned(),
        Style::default()
            .fg(accent.unwrap_or(Color::Green))
            .add_modifier(Modifier::BOLD),
    )])
    .style(if row_selected {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    })
}

fn row_marker_style() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

fn new_slot_line(selected: bool) -> Line<'static> {
    Line::from(vec![
        if selected {
            Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("  ")
        },
        Span::styled(
            "+ create object",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn card_border_color(card: &CardSnapshot) -> Color {
    if card.address() == &CardAddress::NewSlot {
        return Color::DarkGray;
    }
    if let Some(status) = card
        .rows()
        .iter()
        .find_map(|row| (row.key() == &CardRowKey::Status).then(|| display_content(row.content())))
    {
        return match status.as_str() {
            "Building" | "Pending" => Color::Yellow,
            "Failed" => Color::Red,
            "Cancelled" | "Consumed" | "Tombstone" => Color::DarkGray,
            _ => Color::Green,
        };
    }
    if card.owned_slot().is_some() {
        Color::Green
    } else {
        Color::Magenta
    }
}

fn row_label_style(key: &CardRowKey, accent: Color) -> Style {
    if matches!(
        key,
        CardRowKey::Field(_) | CardRowKey::Element(_) | CardRowKey::MapValue(_)
    ) {
        Style::default().fg(accent)
    } else {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM)
    }
}

fn content_spans(row: &CardRowSnapshot, accent: Color) -> Vec<Span<'static>> {
    let (text, style) = match row.content() {
        CardRowContent::Text(text) => {
            let style = match row.key() {
                CardRowKey::Shape | CardRowKey::Variant | CardRowKey::Value => Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                CardRowKey::Status => match text.as_str() {
                    "Building" | "Pending" => Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    "Failed" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "Cancelled" | "Consumed" | "Tombstone" => Style::default().fg(Color::DarkGray),
                    _ => Style::default().fg(Color::Green),
                },
                _ => Style::default().fg(Color::White),
            };
            (text.clone(), style)
        }
        CardRowContent::Address(address) => (
            address.to_string(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
        CardRowContent::Binding(binding) => {
            let style = match binding {
                FieldBindingSnapshot::Unset => {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                }
                FieldBindingSnapshot::PendingProducer => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                _ => Style::default().fg(accent).add_modifier(Modifier::BOLD),
            };
            (display_content(row.content()), style)
        }
        CardRowContent::RootAction(_) => (
            display_content(row.content()),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    };
    vec![Span::styled(text, style)]
}

fn field_group_color(index: usize) -> Color {
    match index % 4 {
        0 => Color::Blue,
        1 => Color::Green,
        2 => Color::Yellow,
        _ => Color::Magenta,
    }
}

fn display_address(address: &CardAddress) -> String {
    match address {
        CardAddress::Value(address) => address.to_string(),
        CardAddress::NewSlot => "new slot".to_owned(),
    }
}

fn display_content(content: &CardRowContent) -> String {
    match content {
        CardRowContent::Text(text) => text.clone(),
        CardRowContent::Address(address) => address.to_string(),
        CardRowContent::Binding(binding) => match binding {
            FieldBindingSnapshot::Unset => "unset".to_owned(),
            FieldBindingSnapshot::Default => "<default>".to_owned(),
            FieldBindingSnapshot::InlineOwned { shape } => format!("inline {shape}"),
            FieldBindingSnapshot::CloneFrom(address) => format!("clone {address}"),
            FieldBindingSnapshot::MoveFrom(slot) => format!("move slot {slot}"),
            FieldBindingSnapshot::BorrowFrom(address) => format!("borrow {address}"),
            FieldBindingSnapshot::PendingProducer => "pending producer".to_owned(),
        },
        CardRowContent::RootAction(action) => action.label(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::Arena;
    use crate::object_explorer::ArenaAddressSource;
    use crate::object_explorer::CardRowKey;
    use crate::object_explorer::ValueAddress;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::num::NonZeroUsize;

    #[derive(Clone, Debug, Facet)]
    #[repr(C)]
    struct RenderThing {
        age: usize,
        name: String,
    }

    fn runtime<T>(value: T) -> RuntimeValue
    where
        T: Facet<'static> + Send + 'static,
    {
        RuntimeValue::from_box(Box::new(value)).expect("test value is representable")
    }

    #[test]
    fn card_window_rendering_uses_only_bounded_snapshots_and_semantic_rows() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(RenderThing {
                age: 42,
                name: "Ada".to_owned(),
            }))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(3).unwrap(), 2).unwrap();
        let selected = CardAddress::Value(ValueAddress::root(root));
        let selected_row = CardRowKey::Field("name".to_owned());
        let backend = TestBackend::new(90, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    Some(&selected_row),
                    CardLayoutAxis::Horizontal,
                    false,
                    30,
                )
                .draw(frame, frame.area());
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("slot 0 [owned]"));
        assert!(rendered.contains("shape: RenderThing"));
        assert!(rendered.contains("age: usize"));
        assert!(rendered.contains("42"));
        assert!(rendered.contains("name: String"));
        assert!(rendered.contains(">name: String"));
        assert!(rendered.contains("Ada"));
        assert!(!rendered.contains("age (usize)"));
        assert!(rendered.contains("vvvvv"));
        assert!(rendered.contains("^^^^^"));
        assert!(rendered.contains("slot 0.age [projection]"));
        assert!(rendered.contains("slot 0.name [projection]"));
        assert_eq!(window.cards().len(), 3);

        let age_row = window.cards()[0]
            .rows()
            .iter()
            .find(|row| row.key() == &CardRowKey::Field("age".to_owned()))
            .unwrap();
        let renderer = CardWindowRenderer::new(
            &window,
            &selected,
            Some(&selected_row),
            CardLayoutAxis::Horizontal,
            false,
            30,
        );
        let type_line = type_line(age_row.type_name().unwrap(), Color::Blue);
        let type_span = type_line
            .spans
            .iter()
            .find(|span| span.content == "usize")
            .expect("ready field type has its own semantic span");
        assert_eq!(type_span.style.fg, Some(Color::Blue));
        assert!(type_span.style.add_modifier.contains(Modifier::DIM));
        let line = renderer.render_row(age_row, true, Some(Color::Blue));
        let value_span = line
            .spans
            .iter()
            .find(|span| span.content == "slot 0.age")
            .expect("field value has its own semantic span");
        assert_eq!(value_span.style.fg, Some(Color::Blue));
        assert!(value_span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn focused_and_vertical_rendering_do_not_request_additional_cards() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime((0_usize..1_000_000).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(8).unwrap(), 3).unwrap();
        let selected = CardAddress::Value(ValueAddress::root(root));
        let backend = TestBackend::new(50, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    None,
                    CardLayoutAxis::Vertical,
                    true,
                    10,
                )
                .draw(frame, frame.area());
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("slot 0 [owned]"));
        assert!(!rendered.contains("slot 0[0] [projection]"));
        assert_eq!(window.cards().len(), 8);
        assert!(window.has_after());
    }

    #[test]
    fn vertical_rendering_keeps_the_selected_card_in_the_visible_capacity() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..32).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(8).unwrap(), 1).unwrap();
        let selected = window.cards().last().unwrap().address().clone();
        let backend = TestBackend::new(50, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    None,
                    CardLayoutAxis::Vertical,
                    false,
                    7,
                )
                .draw(frame, frame.area());
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains(&display_address(&selected)));
        assert!(!rendered.contains("slot 0 [owned]"));
        assert!(rendered.contains("vvvvv"));
        assert!(rendered.contains("^^^^^"));
    }

    #[test]
    fn cards_share_the_space_remaining_after_capacity_is_chosen() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime(RenderThing {
                age: 42,
                name: "Ada".to_owned(),
            }))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(2).unwrap(), 1).unwrap();
        let selected = window.cards()[0].address().clone();
        let backend = TestBackend::new(81, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    None,
                    CardLayoutAxis::Horizontal,
                    false,
                    38,
                )
                .draw(frame, frame.area());
            })
            .unwrap();

        assert_eq!(
            terminal
                .backend()
                .buffer()
                .cell((80, 1))
                .expect("last terminal cell exists")
                .symbol(),
            "┐",
            "the final card grows through the remainder instead of leaving a blank gutter"
        );
    }

    fn horizontal_scrollbar_line(window: &CardWindow, selected: &CardAddress) -> String {
        let backend = TestBackend::new(48, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    window,
                    selected,
                    None,
                    CardLayoutAxis::Horizontal,
                    false,
                    12,
                )
                .draw(frame, frame.area());
            })
            .unwrap();
        terminal
            .backend()
            .to_string()
            .lines()
            .last()
            .unwrap_or_default()
            .to_owned()
    }

    fn horizontal_scrollbar_line_with_total(
        window: &CardWindow,
        selected: &CardAddress,
        total: usize,
    ) -> String {
        let backend = TestBackend::new(48, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    window,
                    selected,
                    None,
                    CardLayoutAxis::Horizontal,
                    false,
                    12,
                )
                .with_query_total(QueryTotal::Exact(total))
                .draw(frame, frame.area());
            })
            .unwrap();
        terminal
            .backend()
            .to_string()
            .lines()
            .last()
            .unwrap_or_default()
            .to_owned()
    }

    #[test]
    fn horizontal_scrollbar_thumb_size_does_not_change_with_position() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..20).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let full_window = CardWindow::first(&source, NonZeroUsize::new(20).unwrap(), 1).unwrap();
        let first = CardWindow::from_cards(full_window.cards()[..10].to_vec(), 0, false, true);
        let second = CardWindow::from_cards(full_window.cards()[10..].to_vec(), 10, true, false);
        let first_selected = first.cards()[0].address().clone();
        let second_selected = second.cards()[0].address().clone();
        let first_line = horizontal_scrollbar_line_with_total(&first, &first_selected, 21);
        let second_line = horizontal_scrollbar_line_with_total(&second, &second_selected, 21);
        let first_thumb = first_line
            .chars()
            .filter(|character| *character == '█')
            .count();
        let second_thumb = second_line
            .chars()
            .filter(|character| *character == '█')
            .count();

        assert_eq!(
            first_thumb, second_thumb,
            "the horizontal thumb size must be derived from total content, not its current position"
        );
    }
    #[test]
    fn horizontal_scrollbar_stays_put_when_selection_stays_in_the_viewport() {
        let mut arena = Arena::default();
        arena
            .insert_ready(runtime((0_usize..20).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(20).unwrap(), 1).unwrap();
        let first = window.cards()[0].address().clone();
        let second = window.cards()[1].address().clone();

        assert_eq!(
            horizontal_scrollbar_line(&window, &first),
            horizontal_scrollbar_line(&window, &second),
            "selection movement inside the rendered card viewport must not move its scrollbar"
        );
    }

    #[test]
    fn selected_rows_scroll_inside_the_card_and_both_scrollbars_are_visible() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime((0_usize..9).collect::<Vec<_>>()))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(1).unwrap(), 10).unwrap();
        let selected = CardAddress::Value(ValueAddress::root(root));
        let selected_row = CardRowKey::Element(8);
        let backend = TestBackend::new(48, 12);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    Some(&selected_row),
                    CardLayoutAxis::Horizontal,
                    false,
                    38,
                )
                .draw(frame, frame.area());
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains(">[8]: usize"));
        assert!(rendered.contains("8"));
        assert!(!rendered.contains("[0]: usize"));
        assert!(
            rendered.lines().last().unwrap_or_default().contains('◄')
                && rendered.lines().last().unwrap_or_default().contains('►'),
            "a continuation window renders the card-axis scrollbar"
        );
        assert!(
            (1_u16..10).any(|y| {
                terminal
                    .backend()
                    .buffer()
                    .cell((47, y))
                    .is_some_and(|cell| matches!(cell.symbol(), "█" | "║"))
            }),
            "overflowing selected-card rows render a vertical scrollbar"
        );
        assert!(
            terminal
                .backend()
                .buffer()
                .cell((47, 8))
                .is_some_and(|cell| matches!(cell.symbol(), "█" | "║")),
            "a bottom viewport must place the vertical thumb at the bottom of its track"
        );
    }
}
