use crate::Choice;
use crate::PickError;
use crate::PickResult;
use cloud_terrastodon_command::InvalidatableCache;
use compact_str::CompactString;
use nucleo::Nucleo;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use ratatui::Terminal;
use ratatui::crossterm::event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::EnterAlternateScreen;
use ratatui::crossterm::terminal::enable_raw_mode;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::prelude::CrosstermBackend;
use ratatui::restore;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::List;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::io::BufWriter;
use std::io::stderr;
use std::pin::Pin;
use std::sync::Arc;
use tui_textarea::CursorMove;
use tui_textarea::TextArea;

/// Used to supply choices to a [`PickerTui`] after a reload command from the user.
/// This will always invalidate the cache before supplying new choices.
/// The initial choices for the [`PickerTui`] should be created without using this type to avoid cache invalidation on first load.
pub trait ChoiceSupplier<T> {
    fn supply_choices(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = eyre::Result<Vec<Choice<T>>>> + Send + 'static>>;
}
impl<T, C, O, E> ChoiceSupplier<T> for C
where
    C: Clone + InvalidatableCache + IntoFuture<Output = eyre::Result<O>, IntoFuture = Pin<Box<dyn Future<Output = eyre::Result<O>> + Send>>>,
    O: IntoIterator<Item = E> + 'static,
    E: Into<Choice<T>>,
{
    fn supply_choices(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = eyre::Result<Vec<Choice<T>>>> + Send + 'static>> {
        let invalidation = self.invalidate_cache();
        let fut = self.clone().into_future();
        Box::pin(async move {
            invalidation.await?;
            Ok(fut.await?.into_iter().map(Into::into).collect())
        })
    }
}

pub struct PickerTui<T> {
    /// The list of items being chosen from
    pub choices: Vec<Choice<T>>,
    /// The function to reload choices
    pub reload_command: Option<Box<dyn ChoiceSupplier<T>>>,
    /// The current query
    pub query_text_area: TextArea<'static>,
    /// The previous query, used to determine if the query has changed
    pub previous_query: Option<String>,
    /// The header text to indicate to the user what is being chosen
    pub header: Option<String>,
    /// Determines if the query should be pushed to the search engine
    pub query_changed: bool,
    /// If there is zero or one options, automatically accept the choice
    pub auto_accept: bool,
}

impl<T, I> From<I> for PickerTui<T>
where
    I: IntoIterator<Item = Choice<T>>,
{
    /// This has stronger type inference than [`PickerTui::new`]
    fn from(choices: I) -> Self {
        Self::new(choices)
    }
}

type Key = CompactString;

impl<T> PickerTui<T> {
    pub fn new<E: Into<Choice<T>>>(choices: impl IntoIterator<Item = E>) -> Self {
        let rtn = Self {
            choices: choices.into_iter().map(Into::into).collect(),
            reload_command: None,
            query_text_area: Self::build_text_area(""),
            previous_query: Default::default(),
            header: Default::default(),
            query_changed: false,
            auto_accept: true,
        };
        #[cfg(debug_assertions)]
        {
            if rtn.choices.iter().any(|c| c.key.contains('\t')) {
                tracing::warn!(
                    "Warning: Some choice keys contain tab characters, which may render poorly in the TUI"
                );
                println!("Press Enter to continue...");
                let _: Result<_, _> = std::io::stdin().read_line(&mut String::new());
            }
        }
        rtn
    }

    pub async fn new_reloadable<C, O, E>(command: C) -> eyre::Result<Self>
    where
        C: 'static + Clone + InvalidatableCache + IntoFuture<Output = eyre::Result<O>, IntoFuture = Pin<Box<dyn Future<Output = eyre::Result<O>> + Send>>>,
        O: IntoIterator<Item = E> + 'static,
        E: Into<Choice<T>>,
    {
        let choices = command.clone().await?;
        let mut this = Self::new(choices);
        this.reload_command = Some(Box::new(command));
        Ok(this)
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

    pub fn set_auto_accept(mut self, auto_accept: bool) -> Self {
        self.auto_accept = auto_accept;
        self
    }

    pub fn set_query(mut self, query: impl Into<String>) -> Self {
        self.query_text_area = Self::build_text_area(query.into().as_str());
        self.previous_query = None;
        self.query_changed = true;
        self
    }

    pub fn pick_one(self) -> PickResult<T> {
        match self.pick_inner(false) {
            Ok(mut items) => Ok(items.pop().unwrap()),
            Err(e) => Err(e),
        }
    }
    pub fn pick_many(self) -> PickResult<Vec<T>> {
        self.pick_inner(true)
    }

    pub fn pick_inner(mut self, many: bool) -> PickResult<Vec<T>> {
        // Short circuit if applicable
        match (self.choices.len(), self.auto_accept) {
            (0, _) => return Err(PickError::NoChoicesProvided),
            (1, true) => {
                let choice = self.choices.remove(0);
                return Ok(vec![choice.value]);
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

        // Enter ratatui using stderr
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            hook(info);
        }));
        enable_raw_mode()?;
        execute!(stderr(), EnterAlternateScreen)?;
        // https://blog.orhun.dev/stdout-vs-stderr/
        let backend = CrosstermBackend::new(BufWriter::new(stderr()));
        let mut terminal = Terminal::new(backend)?;
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
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                        if (!many || marked_for_return.is_empty())
                            && let Some(selected_index) = list_state.selected()
                        {
                            let selected_key = search_results_keys.swap_remove(selected_index);
                            marked_for_return.insert(selected_key);
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
                    KeyCode::BackTab if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.query_changed = self.query_text_area.delete_word()
                    }
                    _ => {
                        // Send the key to the search box
                        self.query_changed = self.query_text_area.input(key);
                    }
                }
            }

            if self.query_changed {
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
                list_state.select_first();
                self.query_changed = false;
            }

            // Tick the search engine and update the results
            let status = nucleo.tick(10);
            should_rebuild_search_results_display |= status.changed;

            if should_rebuild_search_results_display {
                // Get the current results from the search engine
                let snapshot = nucleo.snapshot();
                let items = snapshot.matched_items(..);

                // Clear the previous search results
                search_results_keys.clear();

                // Push the new search results
                let mut search_results_display: Vec<Text> = Default::default();
                for item in items {
                    let key: Key = item.data.clone();

                    let mut text = Text::from(key.to_string());
                    if many {
                        if marked_for_return.contains(&key) {
                            text.lines[0].spans.insert(0, Span::from("● ").red());
                            for line in text.lines.iter_mut().skip(1) {
                                line.spans.insert(0, Span::from("  "));
                            }
                        } else if !marked_for_return.is_empty() {
                            for line in text.lines.iter_mut() {
                                line.spans.insert(0, Span::from("  "));
                            }
                        };
                    }

                    search_results_keys.push(key);
                    search_results_display.push(text);
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
            return Err(PickError::Cancelled);
        }

        let mut rtn: Vec<T> = Vec::with_capacity(marked_for_return.len());
        for key in marked_for_return {
            if let Some(value) = choice_map.remove(&key) {
                rtn.push(value);
            }
        }
        Ok(rtn)
    }
}

#[cfg(test)]
mod test {
    use super::PickerTui;
    use crate::Choice;

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
    pub fn it_works22() -> eyre::Result<()> {
        let items = vec![
            Thingy {
                name: format!("{:<25} bruh", "dog"),
                value: 1,
            },
            Thingy {
                name: format!("{:<25} bruh", "cat"),
                value: 2,
            },
            Thingy {
                name: format!("{:<25} bruh", "house"),
                value: 3,
            },
            Thingy {
                name: format!("{:<25} bruh", "pickle"),
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

    #[test]
    #[ignore]
    pub fn it_works4() -> eyre::Result<()> {
        let results = PickerTui::<usize>::new([
            Choice {
                key: "one\none".into(),
                value: 1,
            },
            Choice {
                key: "two\ntwo".into(),
                value: 2,
            },
            Choice {
                key: "three\nthree".into(),
                value: 3,
            },
        ])
        .set_header("Select some numbers")
        .pick_many()?;
        dbg!(results);
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn it_works_many() -> eyre::Result<()> {
        // create 100,000 items
        let items = (0..100_000).map(|i| Choice {
            key: format!("Item {}", i),
            value: i,
        });
        let results = PickerTui::<usize>::new(items)
            .set_header("Select some items")
            .pick_many()?;
        dbg!(results);
        Ok(())
    }
}
