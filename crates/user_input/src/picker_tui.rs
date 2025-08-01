use crate::Choice;
use compact_str::CompactString;
use nucleo::Nucleo;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use ratatui::crossterm::event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::sync::Arc;
use tui_textarea::CursorMove;
use tui_textarea::TextArea;

pub struct PickerTui<T: Send + Sync + 'static> {
    /// The list of items being chosen from
    pub choices: Vec<Choice<T>>,
    /// The current query
    pub query_text_area: TextArea<'static>,
    /// The previous query, used to determine if the query has changed
    pub previous_query: Option<String>,
    /// The header text to indicate to the user what is being chosen
    pub header: Option<String>,
    /// Determines if the query should be pushed to the search engine
    pub query_changed: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PickResponse<T> {
    Some(T),
    Cancelled,
}
#[derive(Debug, Eq, PartialEq)]
pub enum PickManyResponse<T> {
    Some(Vec<T>),
    Cancelled,
}

type Key = CompactString;

impl<T: Send + Sync + 'static> PickerTui<T> {
    pub fn new<E: Into<Choice<T>>>(
        choices: impl IntoIterator<Item = E>,
    ) -> Self {
        Self {
            choices: choices.into_iter().map(Into::into).collect(),
            query_text_area: Self::build_text_area(""),
            previous_query: Default::default(),
            header: Default::default(),
            query_changed: false,
        }
    }

    fn build_text_area(query: &str) -> TextArea<'static> {
        let mut text_area = TextArea::new(vec![query.to_string()]);
        text_area.move_cursor(CursorMove::End);
        text_area.set_block(Block::bordered());
        text_area
    }

    pub fn set_header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn set_query(mut self, query: impl Into<String>) -> Self {
        self.query_text_area = Self::build_text_area(query.into().as_str());
        self.previous_query = None;
        self.query_changed = true;
        self
    }

    pub fn pick_one(self) -> eyre::Result<PickResponse<T>> {
        match self.pick_inner(false)? {
            PickManyResponse::Some(mut items) => Ok(PickResponse::Some(items.pop().unwrap())),
            PickManyResponse::Cancelled => Ok(PickResponse::Cancelled),
        }
    }
    pub fn pick_many(self) -> eyre::Result<PickManyResponse<T>> {
        self.pick_inner(true)
    }

    fn pick_inner(mut self, many: bool) -> eyre::Result<PickManyResponse<T>> {
        // Short circuit if applicable
        match self.choices.len() {
            0 => return Ok(PickManyResponse::Cancelled),
            1 => {
                let choice = self.choices.remove(0);
                return Ok(PickManyResponse::Some(vec![choice.value]));
            }
            _ => {}
        }

        // Prepare the search engine
        let mut nucleo: Nucleo<Key> =
            Nucleo::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);

        // Build our lookup table and inject the keys into the search engine
        let mut choice_map: FxHashMap<Key, T> =
            FxHashMap::with_capacity_and_hasher(self.choices.len(), FxBuildHasher);
        for choice in self.choices {
            let key: Key = choice.key.into();
            choice_map.insert(key.clone(), choice.value);
            nucleo.injector().push(key, |x, cols| {
                cols[0] = x.as_str().into();
            });
        }

        // Track what items we will return
        let mut marked_for_return: FxHashSet<Key> = Default::default();

        // Enter ratatui
        let mut terminal = ratatui::init();
        terminal.clear()?;

        // Set up the search results state
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut search_results_list: List = Default::default();
        let mut search_results_keys: Vec<Key> = Vec::new();

        // Main event loop
        loop {
            // Handle keyboard input
            let mut should_rebuild_search_results_display = false;
            if event::poll(std::time::Duration::from_millis(100))?
                && let event::Event::Key(key) = event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Esc => {
                        // Send cancellation
                        marked_for_return.clear();
                        break;
                    }
                    KeyCode::Up => {
                        list_state.select_previous();
                    }
                    KeyCode::Down => {
                        list_state.select_next();
                    }
                    KeyCode::Tab => {
                        // Toggle selection if multiple selection is allowed
                        if many
                            && let Some(selected_item) = list_state
                                .selected()
                                .and_then(|index| search_results_keys.get(index))
                        {
                            if marked_for_return.contains(selected_item) {
                                marked_for_return.remove(selected_item);
                            } else {
                                marked_for_return.insert(selected_item.clone());
                            }
                            should_rebuild_search_results_display = true;
                            list_state.select_next();
                        }
                    }
                    KeyCode::Enter => {
                        // Submit the selected item if no items marked, submit existing marked items otherwise (do not mark the selected item)
                        if !many || marked_for_return.is_empty() {
                            if let Some(selected_index) = list_state.selected() {
                                let selected_key = search_results_keys.swap_remove(selected_index);
                                marked_for_return.insert(selected_key);
                            }
                        }
                        break;
                    }
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Select all items
                        marked_for_return.extend(search_results_keys.iter().cloned());
                        should_rebuild_search_results_display = true;
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Toggle all items
                        marked_for_return = search_results_keys
                            .iter()
                            .filter(|key| !marked_for_return.contains(*key))
                            .cloned()
                            .collect::<FxHashSet<_>>();
                        should_rebuild_search_results_display = true;
                    }
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Deselect all items
                        marked_for_return.clear();
                        should_rebuild_search_results_display = true;
                    }
                    KeyCode::PageUp => {
                        // Move the selection up by one page
                        if let Some(selected) = list_state.selected() {
                            let new_index = selected.saturating_sub(10);
                            list_state.select(Some(new_index));
                        }
                    }
                    KeyCode::PageDown => {
                        if let Some(selected) = list_state.selected() {
                            let new_index = selected.saturating_add(10);
                            if new_index < search_results_keys.len() {
                                list_state.select(Some(new_index));
                            }
                        }
                    }
                    KeyCode::Home => {
                        // Move the selection to the top
                        list_state.select(Some(0));
                    }
                    KeyCode::End => {
                        // Move the selection to the bottom
                        list_state.select(Some(search_results_keys.len().saturating_sub(1)));
                    }
                    _ => {
                        // Send the key to the search box
                }
            }

                let new_query = self.query_text_area.lines().join("\n");
                nucleo.pattern.reparse(
                    0,
                    &new_query,
                    CaseMatching::Smart,
                    Normalization::Smart,
                    match &self.previous_query {
                        None => false,
                        Some(previous_query) => new_query.starts_with(previous_query),
                    },
                );
                self.previous_query = Some(new_query);
            }

            // Tick the search engine and update the results
            let status = nucleo.tick(10).clone();
            should_rebuild_search_results_display |= status.changed;

            if should_rebuild_search_results_display {
                // Get the current results from the search engine
                let snapshot = nucleo.snapshot();
                let items = snapshot.matched_items(..);

                // Clear the previous search results
                search_results_keys.clear();

                // Push the new search results
                let mut search_results_display: Vec<Line> = Default::default();
                for item in items {
                    let key: Key = item.data.clone();

                    let mut line = Line::default();
                    if many && marked_for_return.contains(&key) {
                        line.push_span(Span::from("● ").red());
                    } else if many && !marked_for_return.is_empty() {
                        line.push_span(Span::from("  "));
                    }
                    line.push_span(Span::from(key.clone()));

                    search_results_keys.push(key);
                    search_results_display.push(line);
                }
                let counts_title = if many {
                    format!(
                        "{} items marked for return of {} items matching query of {} items total",
                        marked_for_return.len(),
                        search_results_keys.len(),
                        choice_map.len()
                    )
                } else {
                    format!(
                        "{} items matching query of {} items total",
                        search_results_keys.len(),
                        choice_map.len()
                    )
                };
                self.query_text_area
                    .set_block(Block::bordered().title(counts_title));
                search_results_list = List::new(search_results_display)
                    .block({
                        let mut block = Block::bordered();
                        if let Some(header) = &self.header {
                            block = block.title(header.as_str());
                        }
                        block
                    })
                    .highlight_style(Style::new().bg(Color::Blue).fg(Color::Yellow));
            }

            // Draw to the terminal
            terminal.draw(|f| {
                let area = f.area();
                let buf = f.buffer_mut();

                let [list_area, searchbox_area] =
                    Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

                // Draw search results area
                StatefulWidget::render(&search_results_list, list_area, buf, &mut list_state);

                // Draw search box area
                if self.query_text_area.is_empty() {
                    Paragraph::new("Type to search".gray())
                        .block(Block::bordered())
                        .render(searchbox_area, buf);
                } else {
                    self.query_text_area.render(searchbox_area, buf);
                }
            })?;
        }
        ratatui::restore();

        if marked_for_return.is_empty() {
            return Ok(PickManyResponse::Cancelled);
        }

        let mut rtn: Vec<T> = Vec::with_capacity(marked_for_return.len());
        for key in marked_for_return {
            if let Some(value) = choice_map.remove(&key) {
                rtn.push(value);
            }
        }
        Ok(PickManyResponse::Some(rtn))
    }
}

#[cfg(test)]
mod test {
    use super::PickerTui;

    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Thingy {
        name: String,
        value: u32,
    }
    impl std::fmt::Display for Thingy {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    #[test]
    #[ignore]
    pub fn it_works() -> eyre::Result<()> {
        let items = vec!["dog", "cat", "house", "pickle", "mouse"];
        let results = PickerTui::new(items)
            .set_header("Select an item")
            .pick_one()?;
        dbg!(results);
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn it_works2() -> eyre::Result<()> {
        let items = vec![
            Thingy {
                name: "dog".into(),
                value: 1,
            },
            Thingy {
                name: "cat".into(),
                value: 2,
            },
            Thingy {
                name: "house".into(),
                value: 3,
            },
            Thingy {
                name: "pickle".into(),
                value: 4,
            },
        ];
        let results = PickerTui::new(items)
            .set_header("Select an item")
            .pick_many()?;
        dbg!(results);
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn it_works3() -> eyre::Result<()> {
        let results = PickerTui::new(1..10_000_000)
            .set_header("Select some numbers")
            .set_query("100")
            .pick_many()?;
        dbg!(results);
        Ok(())
    }
}
