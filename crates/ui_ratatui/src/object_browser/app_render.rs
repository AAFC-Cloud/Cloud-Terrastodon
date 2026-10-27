use super::app::BrowserMode;
use super::app::ObjectBrowserApp;
use super::render::CardLayoutAxis;
use super::render::CardWindowRenderer;
use crate::object_explorer::CardAddress;
use crate::object_explorer::CardWindow;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::Modifier;
use ratatui::prelude::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use std::num::NonZeroUsize;

impl ObjectBrowserApp {
    /// Derive the next bounded engine request from terminal geometry.
    pub(crate) fn configure_viewport(&mut self, area: Rect) {
        let pool_height = area.height.max(1) as usize;
        let (main_axis, breadth) = match self.axis {
            CardLayoutAxis::Horizontal => (area.width, self.card_width),
            CardLayoutAxis::Vertical => (area.height, self.card_height),
        };
        self.last_card_main_axis = main_axis;
        let cards = match self.axis {
            CardLayoutAxis::Horizontal => (area.width as usize / breadth.max(1) as usize).max(1),
            CardLayoutAxis::Vertical => (pool_height / breadth.max(1) as usize).max(1),
        }
        .min(128);
        self.max_cards = NonZeroUsize::new(cards).unwrap();
        let card_height = match self.axis {
            CardLayoutAxis::Horizontal => pool_height,
            CardLayoutAxis::Vertical => pool_height / cards,
        };
        self.max_relationship_rows = card_height.saturating_sub(4).clamp(1, 64);
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let sections = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);
        let selected = self.selected_address();
        let tab = self.controller.active_tab_header();
        let (tab_ordinal, tab_count) = self.controller.active_tab_ordinal().unwrap_or((0, 0));
        frame.render_widget(
            Paragraph::new(format!(
                "slot {} - Tab {} of {} - {}",
                tab.slot(),
                tab_ordinal,
                tab_count,
                tab.name()
            ))
            .alignment(ratatui::layout::Alignment::Center),
            sections[0],
        );
        frame.render_widget(
            Paragraph::new(breadcrumb_header(tab, self.breadcrumb_focus)),
            sections[1],
        );

        let pool = Block::default().borders(Borders::ALL).title("Object Pool");
        let inner = pool.inner(sections[2]);
        frame.render_widget(pool, sections[2]);
        self.configure_viewport(inner);
        self.draw_pool(frame, inner, &selected);

        let status = if self.controller.is_scanning() {
            format!("{} | scanning", self.status)
        } else if let Some(search) = &self.row_search {
            format!(
                "Search: {} ({} match{}) | Up/Down/PgUp/PgDn: navigate | Enter: activate | Esc: cancel",
                search.query(),
                search.matches().len(),
                if search.matches().len() == 1 {
                    ""
                } else {
                    "es"
                }
            )
        } else {
            self.status.clone()
        };
        frame.render_widget(Paragraph::new(status), sections[3]);
        self.draw_dialog(frame, area);
    }

    fn draw_pool(&self, frame: &mut Frame<'_>, area: Rect, selected: &CardAddress) {
        let visible_rows = self.row_search.as_ref().map(|search| search.matches());
        let query_total = self
            .controller
            .active_state()
            .map(|state| state.query_total())
            .unwrap_or_default();
        if let Some(window) = self.controller.window() {
            let display_window = if selected == &CardAddress::NewSlot && window.has_after() {
                Some(CardWindow::single(
                    crate::object_explorer::CardSnapshot::new_slot(),
                ))
            } else {
                (!window.has_after()).then(|| window.including_new_slot())
            };
            let window = display_window.as_ref().unwrap_or(window);
            CardWindowRenderer::new(
                window,
                selected,
                self.controller
                    .active_state()
                    .and_then(|state| state.focused_row()),
                self.axis,
                self.focused_card_fill,
                self.card_breadth(),
            )
            .with_viewport_anchor(
                self.controller
                    .active_state()
                    .map(|state| state.viewport_anchor()),
            )
            .with_query_total(query_total)
            .with_visible_rows(visible_rows)
            .draw(frame, area);
        } else if self.controller.is_scanning() && selected != &CardAddress::NewSlot {
            frame.render_widget(Paragraph::new("Scanning object addresses…"), area);
        } else {
            // A completed query can exclude the current selection. Keep the
            // create-object card available without changing focus while drawing.
            let card_area = if selected == &CardAddress::NewSlot {
                area
            } else {
                let sections =
                    Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
                frame.render_widget(
                    Paragraph::new("No matching objects. Shift+End: create an object."),
                    sections[0],
                );
                sections[1]
            };
            let window = CardWindow::single(crate::object_explorer::CardSnapshot::new_slot());
            CardWindowRenderer::new(
                &window,
                selected,
                None,
                self.axis,
                self.focused_card_fill,
                self.card_breadth(),
            )
            .with_query_total(query_total)
            .with_visible_rows(visible_rows)
            .draw(frame, card_area);
        }
    }

    fn card_breadth(&self) -> u16 {
        match self.axis {
            CardLayoutAxis::Horizontal => self.card_width,
            CardLayoutAxis::Vertical => self.card_height,
        }
    }

    fn draw_dialog(&self, frame: &mut Frame<'_>, area: Rect) {
        match self.mode {
            BrowserMode::Pool => {}
            BrowserMode::RowSearch => {}
            BrowserMode::Variant => self.draw_variant_picker(frame, centered(area, 70, 70)),
            BrowserMode::Value => self.draw_value_picker(frame, centered(area, 88, 76)),
            BrowserMode::LinkAction => self.draw_link_action_picker(frame, centered(area, 88, 70)),
            BrowserMode::Text => self.draw_text_editor(frame, centered(area, 72, 30)),
            BrowserMode::BreadcrumbValue => {
                self.draw_breadcrumb_value_editor(frame, centered(area, 72, 30))
            }
            BrowserMode::NestedPicker => {}
            BrowserMode::TabName => self.draw_tab_name_editor(frame, centered(area, 72, 30)),
        }
    }

    fn draw_variant_picker(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let Some(picker) = &self.variant_picker else {
            return;
        };
        let max_rows = area.height.saturating_sub(5) as usize;
        let lines = picker
            .matches()
            .enumerate()
            .take(max_rows)
            .map(|(row, (_, label))| picker_line(label, row == picker.selected_index()))
            .chain(std::iter::once(Line::from(format!(
                "Search: {}",
                picker.query()
            ))))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Pick Variant")),
            area,
        );
    }

    fn draw_value_picker(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let Some(picker) = &self.value_picker else {
            return;
        };
        let max_rows = area.height.saturating_sub(6) as usize;
        let start = window_start(picker.selected_index(), picker.rows().len(), max_rows);
        let mut lines = picker
            .rows()
            .iter()
            .enumerate()
            .skip(start)
            .take(max_rows)
            .map(|(index, row)| picker_line(row.label(), index == picker.selected_index()))
            .collect::<Vec<_>>();
        if picker.has_before() {
            lines.insert(0, Line::from("… earlier compatible values"));
        }
        if picker.has_after() {
            lines.push(Line::from("… more compatible values"));
        }
        lines.push(Line::from(format!("Search: {}", picker.query())));
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Pick Object for {}", picker.field_name())),
            ),
            area,
        );
    }

    fn draw_link_action_picker(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let Some(picker) = &self.link_action_picker else {
            return;
        };
        let columns = Layout::horizontal([Constraint::Percentage(36), Constraint::Percentage(64)])
            .split(area);
        let choices = picker
            .consequences()
            .iter()
            .enumerate()
            .map(|(index, consequence)| {
                picker_line(
                    &format!("{:?}", consequence.action()),
                    index == picker.selected_index(),
                )
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(choices).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Move or Clone"),
            ),
            columns[0],
        );
        let consequence = picker
            .selected()
            .map(|choice| choice.description())
            .unwrap_or("No valid transfer operation");
        frame.render_widget(
            Paragraph::new(consequence)
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title("Consequence")),
            columns[1],
        );
    }

    fn draw_text_editor(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let Some(editor) = &self.text_editor else {
            return;
        };
        frame.render_widget(
            Paragraph::new(format!("> {}", editor.text)).block(
                Block::default().borders(Borders::ALL).title(format!(
                    "Set {}",
                    cloud_terrastodon_registry::describe_shape(editor.shape)
                )),
            ),
            area,
        );
    }

    fn draw_breadcrumb_value_editor(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let Some(editor) = &self.breadcrumb_value_editor else {
            return;
        };
        frame.render_widget(
            Paragraph::new(format!("> {}", editor.text)).block(
                Block::default().borders(Borders::ALL).title(format!(
                    "Filter {} ({})",
                    editor.field_name, editor.field_shape
                )),
            ),
            area,
        );
    }

    fn draw_tab_name_editor(&self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let Some(name) = &self.tab_name_editor else {
            return;
        };
        frame.render_widget(
            Paragraph::new(format!("> {name}"))
                .block(Block::default().borders(Borders::ALL).title("Rename Tab")),
            area,
        );
    }
}

fn breadcrumb_header(
    tab: &crate::object_explorer::TabHeaderSnapshot,
    focus: Option<super::breadcrumb_bar_focus::BreadcrumbBarFocus>,
) -> Line<'static> {
    let mut spans = vec![Span::raw("Everything")];
    if tab.first_visible_breadcrumb() > 0 {
        spans.push(Span::raw(" > "));
        spans.push(Span::raw(format!(
            "… {} earlier",
            tab.first_visible_breadcrumb()
        )));
    }
    for (visible_index, label) in tab.breadcrumb_labels().iter().enumerate() {
        let index = tab.first_visible_breadcrumb() + visible_index;
        spans.push(Span::raw(" > "));
        spans.push(breadcrumb_header_item(
            label.clone(),
            focus.is_some_and(|focus| focus.position() == index),
        ));
    }
    spans.push(Span::raw(" > "));
    spans.push(breadcrumb_header_item(
        "+Add Breadcrumb".to_owned(),
        focus.is_some_and(|focus| focus.is_add(tab.breadcrumb_count())),
    ));
    Line::from(spans)
}

fn breadcrumb_header_item(label: String, focused: bool) -> Span<'static> {
    if focused {
        Span::styled(
            format!("[{label}]"),
            Style::default().add_modifier(Modifier::REVERSED),
        )
    } else {
        Span::raw(label)
    }
}

fn picker_line(label: &str, selected: bool) -> Line<'static> {
    let text = format!("{}{}", if selected { "> " } else { "  " }, label);
    if selected {
        Line::from(Span::styled(
            text,
            Style::default().add_modifier(Modifier::REVERSED),
        ))
    } else {
        Line::from(text)
    }
}

fn window_start(selected: usize, count: usize, capacity: usize) -> usize {
    if capacity == 0 || count <= capacity {
        0
    } else {
        selected
            .saturating_sub(capacity / 2)
            .min(count.saturating_sub(capacity))
    }
}

fn centered(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - height_percent) / 2),
        Constraint::Percentage(height_percent),
        Constraint::Percentage((100 - height_percent) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - width_percent) / 2),
        Constraint::Percentage(width_percent),
        Constraint::Percentage((100 - width_percent) / 2),
    ])
    .split(vertical[1])[1]
}

fn display_card_address(address: &CardAddress) -> String {
    match address {
        CardAddress::Value(address) => address.to_string(),
        CardAddress::NewSlot => "new slot".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_explorer::ArenaQueryContext;
    use crate::object_explorer::ExplorerEngine;
    use crate::object_explorer::SlotId;
    use crate::object_explorer::ValueAddress;
    use crossterm::event::Event;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use ratatui::Terminal;
    use ratatui::TerminalOptions;
    use ratatui::Viewport;
    use ratatui::backend::TestBackend;

    #[tokio::test]
    async fn browser_draws_below_log_history_with_footer_in_the_viewport() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            app.tick().await.unwrap();
            let mut lines = vec![" ".repeat(120); 30];
            lines[0] = format!("{:<120}", " INFO Previous request completed");
            lines[1] = format!("{:<120}", "    at request.rs:42");
            lines[2] = format!("{:<120}", " WARN Earlier diagnostic remains in history");
            let backend = TestBackend::with_lines(lines);
            let history = backend.buffer().content()[..4 * 120].to_vec();
            let mut terminal = Terminal::with_options(
                backend,
                TerminalOptions {
                    viewport: Viewport::Fixed(Rect::new(0, 4, 120, 26)),
                },
            )
            .unwrap();

            terminal.draw(|frame| app.draw(frame)).unwrap();
            assert_eq!(&terminal.backend().buffer().content()[..4 * 120], history);
            let footer = (0..120)
                .map(|x| terminal.backend().buffer()[(x, 29)].symbol())
                .collect::<String>();
            assert!(footer.contains("Left/Right: cards"));
            assert!(footer.contains("Ctrl-O: draw slot"));

            app.status = "Invocation failed; scroll up for full error.".to_owned();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            assert_eq!(&terminal.backend().buffer().content()[..4 * 120], history);
            let footer = (0..120)
                .map(|x| terminal.backend().buffer()[(x, 29)].symbol())
                .collect::<String>();
            assert!(footer.contains("Invocation failed; scroll up for full error."));
            app.close().await.unwrap();
        };
        tokio::join!(engine.run(inbox), client);
    }

    #[tokio::test]
    async fn missing_projection_shows_empty_results_and_keeps_creation_available() {
        let engine = ExplorerEngine::empty();
        let (context, inbox) = ArenaQueryContext::channel(16);
        let client = async move {
            let mut app = ObjectBrowserApp::bootstrap(context).await.unwrap();
            let missing = ValueAddress::root(SlotId::new(1_000));
            app.controller
                .create_projection_tab("missing object", missing.clone())
                .await
                .unwrap();
            app.controller
                .refresh_window(16, NonZeroUsize::new(3).unwrap(), 3)
                .await
                .unwrap();
            assert!(!app.controller.is_scanning());
            assert!(app.controller.window().is_none());

            let mut terminal = Terminal::new(TestBackend::new(100, 16)).unwrap();
            terminal.draw(|frame| app.draw(frame)).unwrap();
            let rendered = terminal.backend().to_string();
            assert!(rendered.contains("No matching objects."));
            assert!(rendered.contains("new object"));
            assert!(!rendered.contains("Scanning object addresses"));
            assert_eq!(app.selected_address(), CardAddress::Value(missing));

            app.handle_event(&Event::Key(KeyEvent::new(
                KeyCode::End,
                KeyModifiers::SHIFT,
            )))
            .await
            .unwrap();
            assert_eq!(app.selected_address(), CardAddress::NewSlot);
            terminal.draw(|frame| app.draw(frame)).unwrap();
            assert!(terminal.backend().to_string().contains("new object"));
            app.close().await.unwrap();
        };
        tokio::join!(engine.run(inbox), client);
    }
}
