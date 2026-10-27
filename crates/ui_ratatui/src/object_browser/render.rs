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
use ratatui::widgets::Wrap;

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
    viewport_anchor: Option<&'window CardAddress>,
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
            viewport_anchor: None,
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

    pub(crate) const fn with_viewport_anchor(
        mut self,
        viewport_anchor: Option<&'window CardAddress>,
    ) -> Self {
        self.viewport_anchor = viewport_anchor;
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
        let (cards, visible_start) = self.visible_cards(area);
        if cards.is_empty() {
            return;
        }
        let show_card_scrollbar = self.axis == CardLayoutAxis::Horizontal
            && !self.focused_card_fill
            && (self.window.has_before()
                || self.window.has_after()
                || cards.len() < self.window.cards().len())
            && area.height >= 2;
        let (cards_area, card_scrollbar_area) = if show_card_scrollbar {
            let sections =
                Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
            (sections[0], Some(sections[1]))
        } else {
            (area, None)
        };
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
            let has_new_slot = self
                .window
                .cards()
                .iter()
                .any(|card| card.address() == &CardAddress::NewSlot);
            let minimum_total = position
                .saturating_add(cards.len())
                .saturating_add(usize::from(
                    self.window.has_after()
                        || visible_start + cards.len() < self.window.cards().len(),
                ));
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
        let mut start = self
            .viewport_anchor
            .and_then(|anchor| cards.iter().position(|card| card.address() == anchor))
            .unwrap_or_default()
            .min(cards.len() - capacity);
        // Query windows can backfill at the end of the pool. Keep the logical
        // viewport anchored inside those snapshots, shifting only to reveal a
        // selection that is outside it (including the appended new-slot card).
        if let Some(selected) = selected {
            if selected < start {
                start = selected;
            } else if selected >= start + capacity {
                start = selected + 1 - capacity;
            }
        }
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
            let mut section = None;
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
                let row_section = match row.key() {
                    CardRowKey::Field(_) => Some("fields"),
                    CardRowKey::Element(_) => Some("elements"),
                    CardRowKey::MapValue(_) => Some("entries"),
                    CardRowKey::Action(_) => Some("actions"),
                    _ => None,
                };
                if let Some(title) = row_section {
                    if section != row_section {
                        lines.push(section_line(title, block.inner(card_area).width));
                    } else if accent.is_some() {
                        lines.push(Line::default());
                    }
                }
                section = row_section;
                if row_selected {
                    selected_line = Some(lines.len());
                }
                if let Some(accent) = accent {
                    lines.extend(field_lines(row, row_selected, accent));
                } else {
                    lines.push(self.render_row(row, selected));
                }
                if row_selected {
                    focused_line = Some(lines.len().saturating_sub(1));
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
        let content_area = block.inner(card_area);
        // Selection refers to semantic rows, while Paragraph scrolling refers
        // to wrapped terminal lines. Measure with the same wrapper used below
        // so wide graphemes, word boundaries, and explicit line breaks agree.
        let mut wrapped_lines = Vec::new();
        let mut line_count = 0;
        let mut wrapped_selected_line = None;
        let mut wrapped_focused_line = None;
        for (index, line) in lines.into_iter().enumerate() {
            if selected_line == Some(index) {
                wrapped_selected_line = Some(line_count);
            }
            for line in split_explicit_lines(line) {
                line_count += Paragraph::new(line.clone())
                    .wrap(Wrap { trim: false })
                    .line_count(content_area.width);
                wrapped_lines.push(line);
            }
            if focused_line == Some(index) {
                wrapped_focused_line = Some(line_count.saturating_sub(1));
            }
        }
        let viewport_height = content_area.height as usize;
        let max_offset = line_count.saturating_sub(viewport_height);
        let scroll_offset = wrapped_focused_line
            .map(|line| line.saturating_sub(viewport_height.saturating_sub(1)))
            .unwrap_or_default()
            .min(max_offset)
            // When the field block exceeds the viewport, keep its heading and
            // selection marker visible rather than scrolling to only its value.
            .min(wrapped_selected_line.unwrap_or(usize::MAX))
            .min(u16::MAX as usize);
        frame.render_widget(
            Paragraph::new(wrapped_lines)
                .wrap(Wrap { trim: false })
                .scroll((scroll_offset as u16, 0))
                .block(block),
            card_area,
        );
        if selected
            && let Some(line) = wrapped_selected_line
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

    fn render_row(&self, row: &CardRowSnapshot, card_selected: bool) -> Line<'static> {
        let row_selected = card_selected && self.selected_row == Some(row.key());
        let accent = Color::White;
        let mut spans = Vec::new();
        if !matches!(row.key(), CardRowKey::Shape | CardRowKey::Action(_)) {
            spans.push(Span::styled(
                row.label().to_owned(),
                row_label_style(row.key(), accent),
            ));
            spans.push(Span::styled(": ", Style::default().fg(Color::DarkGray)));
        }
        spans.extend(content_spans(row, accent));
        Line::from(spans).style(if row_selected {
            Style::default().bg(Color::Blue)
        } else {
            Style::default()
        })
    }
}

/// Ratatui wraps Lines, but a Span containing a newline does not create a new
/// Line. Split only explicit breaks here, retaining each span and row's style;
/// Paragraph remains responsible for word and grapheme-aware wrapping.
fn split_explicit_lines(line: Line<'static>) -> Vec<Line<'static>> {
    if !line.spans.iter().any(|span| span.content.contains('\n')) {
        return vec![line];
    }
    let mut result = Vec::new();
    let mut current = Line::default().style(line.style);
    current.alignment = line.alignment;
    for span in line.spans {
        let mut parts = span.content.split('\n').peekable();
        while let Some(part) = parts.next() {
            let has_break = parts.peek().is_some();
            let part = if has_break {
                part.strip_suffix('\r').unwrap_or(part)
            } else {
                part
            };
            current
                .spans
                .push(Span::styled(part.to_owned(), span.style));
            if has_break {
                result.push(current);
                current = Line::default().style(line.style);
                current.alignment = line.alignment;
            }
        }
    }
    result.push(current);
    result
}

fn type_line(type_name: &str, accent: Color) -> Line<'static> {
    Line::from(vec![
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

fn section_line(title: &str, width: u16) -> Line<'static> {
    let padding = (width as usize).saturating_sub(title.len() + 2);
    Line::from(Span::styled(
        format!(
            "{} {title} {}",
            "─".repeat(padding / 2),
            "─".repeat(padding - padding / 2),
        ),
        Style::default().fg(Color::DarkGray),
    ))
}

/// Expand one selectable relationship into a name/type/value block. The value
/// preview takes precedence over its address, while bindings retain their state
/// colors (for example red for unset inputs).
fn field_lines(row: &CardRowSnapshot, row_selected: bool, accent: Color) -> Vec<Line<'static>> {
    let (kind, name) = match row.key() {
        CardRowKey::Field(_) => ("field ", format!("{:?}", row.label())),
        CardRowKey::Element(_) => ("element ", row.label().to_owned()),
        CardRowKey::MapValue(_) => ("entry ", row.label().to_owned()),
        _ => unreachable!("only relationship rows have field blocks"),
    };
    let mut lines = vec![Line::from(vec![
        Span::styled(kind, Style::default().fg(Color::DarkGray)),
        Span::styled(name, row_label_style(row.key(), accent)),
    ])];
    if let Some(type_name) = row.type_name() {
        lines.push(type_line(type_name, accent));
    }
    let mut value = vec![Span::styled("value ", Style::default().fg(Color::DarkGray))];
    if let Some(preview) = row.value_display() {
        value.push(Span::styled(
            preview.to_owned(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));
    } else {
        value.extend(content_spans(row, accent));
    }
    lines.push(Line::from(value));
    let style = if row_selected {
        Style::default().bg(Color::Blue)
    } else {
        Style::default()
    };
    lines.into_iter().map(|line| line.style(style)).collect()
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
            // Full error reports can contain color-eyre's terminal escapes.
            // Cards use Ratatui styles; retain the original report for logging.
            let text = if text.contains('\u{1b}') {
                strip_ansi_escapes::strip_str(text)
            } else {
                text.clone()
            };
            (text, style)
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
        // Blue is reserved for selection backgrounds.
        0 => Color::Cyan,
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
    use crate::object_explorer::RootRevision;
    use crate::object_explorer::SlotId;
    use crate::object_explorer::ValueAddress;
    use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
    use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
    use cloud_terrastodon_registry::RuntimeValue;
    use facet::Facet;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::borrow::Cow;
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
    fn auth_cards_render_the_active_variant_and_nested_policy_preview() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(AzureDevOpsAuthContext::AzureCli(
                AuthContext::Unresolved,
            )))
            .unwrap();
        arena.insert_ready(runtime(AuthContext::None)).unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(3).unwrap(), 8).unwrap();
        let selected = CardAddress::Value(ValueAddress::root(root));
        let mut terminal = Terminal::new(TestBackend::new(120, 14)).unwrap();
        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    None,
                    CardLayoutAxis::Horizontal,
                    false,
                    40,
                )
                .draw(frame, frame.area());
            })
            .unwrap();
        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("variant: AzureCli"));
        assert!(rendered.contains("variant: Unresolved"));
        assert!(rendered.contains("variant: None"));
        assert!(rendered.contains("value Unresolved"));
    }

    #[test]
    fn tail_viewport_reveals_new_slot_without_evicting_visible_real_cards_early() {
        let mut arena = Arena::default();
        for value in 0_usize..3 {
            arena.insert_ready(runtime(value)).unwrap();
        }
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(3).unwrap(), 1)
            .unwrap()
            .including_new_slot();
        assert!(!window.has_before());
        assert!(!window.has_after());

        for (selected, anchor, expected_start) in [
            (0, 0, 0), // First real card must not be evicted to reserve NewSlot.
            (2, 0, 0), // Last real card remains at the right edge.
            (3, 0, 1), // Revealing NewSlot moves just one card.
            (2, 1, 1), // Moving left within the tail keeps the viewport stable.
            (1, 1, 1),
            (0, 1, 0), // Crossing the left edge reveals only the previous card.
        ] {
            let renderer = CardWindowRenderer::new(
                &window,
                window.cards()[selected].address(),
                None,
                CardLayoutAxis::Horizontal,
                false,
                30,
            )
            .with_viewport_anchor(Some(window.cards()[anchor].address()));
            let (visible, start) = renderer.visible_cards(Rect::new(0, 0, 90, 12));
            assert_eq!(
                start, expected_start,
                "selected {selected}, anchor {anchor}"
            );
            assert_eq!(
                visible
                    .iter()
                    .map(|card| card.address())
                    .collect::<Vec<_>>(),
                window.cards()[expected_start..expected_start + 3]
                    .iter()
                    .map(CardSnapshot::address)
                    .collect::<Vec<_>>(),
            );
        }

        let renderer = CardWindowRenderer::new(
            &window,
            &CardAddress::NewSlot,
            None,
            CardLayoutAxis::Horizontal,
            false,
            30,
        )
        .with_viewport_anchor(Some(window.cards()[2].address()));
        let (single, start) = renderer.visible_cards(Rect::new(0, 0, 30, 12));
        assert_eq!(start, 3, "a one-card viewport reveals NewSlot");
        assert_eq!(single[0].address(), &CardAddress::NewSlot);
        let (spare_capacity, start) = renderer.visible_cards(Rect::new(0, 0, 150, 12));
        assert_eq!(start, 0);
        assert_eq!(spare_capacity.len(), 4, "spare capacity retains every card");

        let mut scrollbar_lines = Vec::new();
        for (selected, anchor) in [(0, 0), (3, 1)] {
            let mut terminal = Terminal::new(TestBackend::new(90, 12)).unwrap();
            terminal
                .draw(|frame| {
                    CardWindowRenderer::new(
                        &window,
                        window.cards()[selected].address(),
                        None,
                        CardLayoutAxis::Horizontal,
                        false,
                        30,
                    )
                    .with_viewport_anchor(Some(window.cards()[anchor].address()))
                    .with_query_total(QueryTotal::Exact(3))
                    .draw(frame, frame.area());
                })
                .unwrap();
            scrollbar_lines.push(
                terminal
                    .backend()
                    .to_string()
                    .lines()
                    .last()
                    .unwrap()
                    .to_owned(),
            );
        }
        assert!(
            scrollbar_lines
                .iter()
                .all(|line| line.contains('◄') && line.contains('►'))
        );
        assert_eq!(
            scrollbar_lines[0].matches('█').count(),
            scrollbar_lines[1].matches('█').count(),
            "NewSlot counts toward the scrollbar even before it is visible",
        );
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
        let backend = TestBackend::new(90, 16);
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
        assert!(rendered.contains("RenderThing"));
        assert!(!rendered.contains("shape:"));
        assert!(rendered.contains("─ fields ─"));
        assert!(rendered.contains("field \"age\""));
        assert!(rendered.contains("type usize"));
        assert!(rendered.contains("value 42"));
        assert!(rendered.contains(">field \"name\""));
        assert!(rendered.contains("type String"));
        assert!(rendered.contains("value Ada"));
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
        let accent = field_group_color(0);
        let type_line = type_line(age_row.type_name().unwrap(), accent);
        let type_span = type_line
            .spans
            .iter()
            .find(|span| span.content == "usize")
            .expect("ready field type has its own semantic span");
        assert_eq!(type_span.style.fg, Some(accent));
        assert!(type_span.style.add_modifier.contains(Modifier::DIM));
        let lines = field_lines(age_row, true, accent);
        let value_span = lines[2]
            .spans
            .iter()
            .find(|span| span.content == "42")
            .expect("field value has its own semantic span");
        assert_eq!(value_span.style.fg, Some(accent));
        assert_ne!(value_span.style.fg, lines[2].style.bg);
        assert!(value_span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn request_card_separates_fields_and_actions_and_filters_whole_sections() -> eyre::Result<()> {
        CancellationToken::new().with_scope(|cancellation| {
            let mut arena = Arena::default();
            let root = arena
                .insert_ready(runtime(AzureDevOpsProjectsListRequest {
                    cancellation: cancellation.clone(),
                    org_url: Cow::Owned("example-org".parse().unwrap()),
                    auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
                }))
                .unwrap();
            let source = ArenaAddressSource::new(&arena);
            let window = CardWindow::first(&source, NonZeroUsize::new(1).unwrap(), 8).unwrap();
            let selected = CardAddress::Value(ValueAddress::root(root));
            let action_keys = window.cards()[0]
                .rows()
                .iter()
                .filter(|row| matches!(row.key(), CardRowKey::Action(_)))
                .map(|row| row.key().clone())
                .collect::<Vec<_>>();

            for visible in [None, Some(action_keys.as_slice()), Some(&[][..])] {
                let mut terminal = Terminal::new(TestBackend::new(85, 24)).unwrap();
                terminal
                    .draw(|frame| {
                        CardWindowRenderer::new(
                            &window,
                            &selected,
                            action_keys.last(),
                            CardLayoutAxis::Horizontal,
                            true,
                            85,
                        )
                        .with_visible_rows(visible)
                        .draw(frame, frame.area());
                    })
                    .unwrap();
                let rendered = terminal.backend().to_string();
                if visible.is_some_and(|keys| keys.is_empty()) {
                    assert!(rendered.contains("No matching rows"));
                    assert!(!rendered.contains("─ actions ─"));
                } else {
                    assert!(rendered.contains("─ actions ─"));
                    assert!(rendered.contains("Self(Clone) -> List<AzureDevOpsProject>"));
                    assert!(rendered.contains("Self(Drop) -> List<AzureDevOpsProject>"));
                    assert!(rendered.contains("Self -> List<AzureDevOpsProject>(Arbitrary)"));
                    assert!(!rendered.contains("action:"));
                }
                if visible.is_none() {
                    assert!(rendered.contains("─ fields ─"));
                    assert!(rendered.contains("field \"cancellation\""));
                    assert!(rendered.contains("value slot 0.cancellation"));
                    assert!(rendered.contains("field \"org_url\""));
                    assert!(rendered.contains("type Cow<AzureDevOpsOrganizationUrl>"));
                    assert!(rendered.contains("value https://dev.azure.com/example-org"));
                    assert!(rendered.contains("field \"auth_context\""));
                } else {
                    assert!(!rendered.contains("─ fields ─"));
                }
            }
            Ok(())
        })
    }

    #[test]
    fn short_cards_keep_the_selected_field_heading_and_unset_binding_visible() {
        let field = CardRowSnapshot::new(
            CardRowKey::Field("name".into()),
            "name",
            CardRowContent::Binding(FieldBindingSnapshot::Unset),
        )
        .with_type_name("String");
        let card = CardSnapshot::owned(
            SlotId::new(0),
            "Request",
            RootRevision::default(),
            vec![
                CardRowSnapshot::new(
                    CardRowKey::Shape,
                    "shape",
                    CardRowContent::Text("Request".into()),
                ),
                field.clone(),
                CardRowSnapshot::new(
                    CardRowKey::Status,
                    "status",
                    CardRowContent::Text("Building".into()),
                ),
            ],
            true,
        );
        let selected = card.address().clone();
        let window = CardWindow::single(card);

        for height in 5..=7 {
            let mut terminal = Terminal::new(TestBackend::new(48, height)).unwrap();
            terminal
                .draw(|frame| {
                    CardWindowRenderer::new(
                        &window,
                        &selected,
                        Some(field.key()),
                        CardLayoutAxis::Horizontal,
                        true,
                        48,
                    )
                    .draw(frame, frame.area());
                })
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(">field \"name\""));
            if height == 7 {
                assert!(rendered.contains("type String"));
                assert!(rendered.contains("value unset"));
            }
        }
        let lines = field_lines(&field, true, field_group_color(0));
        let unset = lines[2]
            .spans
            .iter()
            .find(|span| span.content == "unset")
            .unwrap();
        assert_eq!(unset.style.fg, Some(Color::Red));
    }

    #[test]
    fn failure_details_wrap_and_keep_explicit_breaks_and_styles() {
        let card = CardSnapshot::owned(
            SlotId::new(0),
            "Failed",
            RootRevision::default(),
            vec![
                CardRowSnapshot::new(
                    CardRowKey::Status,
                    "status",
                    CardRowContent::Text("Failed".into()),
                ),
                CardRowSnapshot::new(
                    CardRowKey::Value,
                    "detail",
                    CardRowContent::Text(
                        "\u{1b}[91mThe required organization URL could not be constructed.\u{1b}[0m\r\nCause: \u{1b}[1méquipe 🦀\u{1b}[0m needs an input value.\n\nRetry after supplying the missing organization."
                            .into(),
                    ),
                ),
            ],
            true,
        );
        let selected = card.address().clone();
        let window = CardWindow::single(card);
        for width in [24, 40] {
            let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
            terminal
                .draw(|frame| {
                    CardWindowRenderer::new(
                        &window,
                        &selected,
                        Some(&CardRowKey::Value),
                        CardLayoutAxis::Horizontal,
                        true,
                        width,
                    )
                    .draw(frame, frame.area());
                })
                .unwrap();
            let buffer = terminal.backend().buffer();
            let content = (2..22)
                .map(|y| {
                    (1..width - 1)
                        .map(|x| buffer.cell((x, y)).unwrap().symbol())
                        .collect::<String>()
                })
                .collect::<Vec<_>>();
            let words = content
                .iter()
                .flat_map(|line| line.split_whitespace())
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(
                words,
                "status: Failed detail: The required organization URL could not be constructed. Cause: équipe 🦀 needs an input value. Retry after supplying the missing organization."
            );
            let cause_y = content
                .iter()
                .position(|line| line.starts_with("Cause:"))
                .unwrap() as u16
                + 2;
            let cause = buffer.cell((1, cause_y)).unwrap();
            assert_eq!(cause.fg, Color::Green);
            assert_eq!(cause.bg, Color::Blue);
            assert!(cause.modifier.contains(Modifier::BOLD));
            let retry = content
                .iter()
                .position(|line| line.starts_with("Retry"))
                .unwrap();
            assert!(
                content[retry - 1].trim().is_empty(),
                "the empty error-detail line is retained"
            );
            assert_eq!(buffer.cell((0, 3)).unwrap().symbol(), ">");
        }
    }

    #[test]
    fn selection_and_scrollbar_follow_visual_lines_after_wrapped_content() {
        let selected_row = CardRowKey::Field("next".into());
        let card = CardSnapshot::owned(
            SlotId::new(0),
            "Request",
            RootRevision::default(),
            vec![
                CardRowSnapshot::new(
                    CardRowKey::Shape,
                    "shape",
                    CardRowContent::Text("Request".into()),
                ),
                CardRowSnapshot::new(
                    CardRowKey::Value,
                    "detail",
                    CardRowContent::Text(
                        "A long explanation with many words before the selected field.\nAnother line with a wide character 🦀 and more explanation."
                            .into(),
                    ),
                ),
                CardRowSnapshot::new(
                    selected_row.clone(),
                    "next",
                    CardRowContent::Binding(FieldBindingSnapshot::Unset),
                )
                .with_type_name("String"),
            ],
            true,
        );
        let selected = card.address().clone();
        let window = CardWindow::single(card);
        for width in [18, 30] {
            let mut terminal = Terminal::new(TestBackend::new(width, 10)).unwrap();
            terminal
                .draw(|frame| {
                    CardWindowRenderer::new(
                        &window,
                        &selected,
                        Some(&selected_row),
                        CardLayoutAxis::Horizontal,
                        true,
                        width,
                    )
                    .draw(frame, frame.area());
                })
                .unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains(">field \"next\""));
            assert!(rendered.contains("type String"));
            assert!(rendered.contains("value unset"));
            let buffer = terminal.backend().buffer();
            let marker_y = (2..8)
                .find(|y| buffer.cell((0, *y)).unwrap().symbol() == ">")
                .unwrap();
            assert_eq!(buffer.cell((1, marker_y)).unwrap().symbol(), "f");
            assert_eq!(buffer.cell((1, marker_y)).unwrap().bg, Color::Blue);
            assert_eq!(
                buffer.cell((width - 1, 7)).unwrap().symbol(),
                "█",
                "the viewport ends on the final wrapped line, so the thumb reaches the bottom"
            );
        }
    }

    #[test]
    fn owned_organization_url_slot_renders_its_value() {
        let mut arena = Arena::default();
        let root = arena
            .insert_ready(runtime(
                "example-org".parse::<AzureDevOpsOrganizationUrl>().unwrap(),
            ))
            .unwrap();
        let source = ArenaAddressSource::new(&arena);
        let window = CardWindow::first(&source, NonZeroUsize::new(1).unwrap(), 8).unwrap();
        let selected = CardAddress::Value(ValueAddress::root(root));
        let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();

        terminal
            .draw(|frame| {
                CardWindowRenderer::new(
                    &window,
                    &selected,
                    Some(&CardRowKey::Shape),
                    CardLayoutAxis::Horizontal,
                    true,
                    60,
                )
                .draw(frame, frame.area());
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        assert!(rendered.contains("slot 0 [owned]"));
        assert!(rendered.contains("AzureDevOpsOrganizationUrl"));
        assert!(rendered.contains("value: https://dev.azure.com/example-org"));
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
        assert!(rendered.contains(">element [8]"));
        assert!(rendered.contains("value 8"));
        assert!(!rendered.contains("element [0]"));
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
