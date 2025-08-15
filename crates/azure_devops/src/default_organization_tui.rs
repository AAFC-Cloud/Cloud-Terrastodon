use crate::prelude::get_default_organization_url;
use crate::prelude::set_default_organization_url;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::{self};
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use tui_textarea::TextArea;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTarget {
    CurrentValue,
    RecommendedUrl,
    TextInput,
    SubmitButton,
    CancelButton,
}

impl NavigationTarget {
    fn next(self) -> Self {
        use NavigationTarget::*;
        match self {
            CurrentValue => RecommendedUrl,
            RecommendedUrl => TextInput,
            TextInput => SubmitButton,
            SubmitButton => CancelButton,
            CancelButton => CurrentValue,
        }
    }

    fn prev(self) -> Self {
        use NavigationTarget::*;
        match self {
            CurrentValue => CancelButton,
            RecommendedUrl => CurrentValue,
            TextInput => RecommendedUrl,
            SubmitButton => TextInput,
            CancelButton => SubmitButton,
        }
    }
}

pub struct DefaultOrganizationTui {
    text_input: TextArea<'static>,
    focus: NavigationTarget,
    current_value: Option<String>,
}

impl Default for DefaultOrganizationTui {
    fn default() -> Self {
        Self {
            text_input: TextArea::new(vec![]),
            focus: NavigationTarget::TextInput,
            current_value: None,
        }
    }
}

impl DefaultOrganizationTui {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self) -> eyre::Result<()> {
        // fetch current configured value (may be None)
        self.current_value = get_default_organization_url()
            .await
            .map(|s| s.to_string())
            .ok();

        // set initial text input to current value if present
        if let Some(ref v) = self.current_value {
            self.text_input = TextArea::new(vec![v.clone()]);
        } else {
            self.text_input = TextArea::new(vec!["".to_string()]);
        }
        self.text_input.set_block(
            Block::new()
                .title("Default Azure DevOps Organization Url")
                .borders(Borders::ALL),
        );

        const RECOMMENDED_URL: &str = "https://dev.azure.com/aafc";

        let mut terminal = ratatui::init();
        terminal.clear()?;

        loop {
            // handle keyboard input
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Esc => {
                        // Exit the TUI (cancel)
                        break;
                    }
                    KeyCode::Down | KeyCode::Tab => {
                        self.focus = self.focus.next();
                    }
                    KeyCode::Up | KeyCode::BackTab => {
                        self.focus = self.focus.prev();
                    }
                    KeyCode::Enter => match self.focus {
                        NavigationTarget::SubmitButton => {
                            // set_default_organization_url( self.text_input.lines().join("")).await?;
                            break;
                        }
                        NavigationTarget::CancelButton => {
                            break;
                        }
                        NavigationTarget::TextInput => {
                            // allow textarea to handle Enter (insert newline)
                            let _ = self.text_input.input(key);
                        }
                        _ => {}
                    },
                    _ => {
                        // forward input to textarea only if it has focus
                        if self.focus == NavigationTarget::TextInput {
                            let _changed = self.text_input.input(key);
                        }
                    }
                }
            }

            // draw frame
            terminal.draw(|frame| {
                let size = frame.area();

                let outer = Block::new()
                    .title(Line::from("Azure DevOps Configuration").centered())
                    .borders(Borders::ALL)
                    .border_set(ratatui::symbols::border::DOUBLE)
                    .style(Style::default().fg(Color::Blue));

                let inner = outer.inner(size);
                frame.render_widget(outer, size);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3), // current value
                        Constraint::Length(3), // recommended url
                        Constraint::Length(5), // text input
                        Constraint::Length(3), // buttons
                        Constraint::Min(0),
                    ])
                    .split(inner);

                // Current value display
                let current_text = match &self.current_value {
                    Some(v) if !v.is_empty() => Span::raw(v.clone()),
                    _ => Span::styled("(not set)", Style::default().fg(Color::DarkGray)),
                };
                let mut current_block = Paragraph::new(Text::from(current_text))
                    .block(Block::new().title("Current value").borders(Borders::ALL));
                if self.focus == NavigationTarget::CurrentValue {
                    current_block = current_block.style(Style::default().fg(Color::Yellow));
                }
                frame.render_widget(current_block, chunks[0]);

                // Recommended URL
                let recommended = Paragraph::new(Text::from(Span::raw(RECOMMENDED_URL)))
                    .block(Block::new().title("Recommended url").borders(Borders::ALL));
                let recommended = if self.focus == NavigationTarget::RecommendedUrl {
                    recommended.style(Style::default().fg(Color::Yellow))
                } else {
                    recommended
                };
                frame.render_widget(recommended, chunks[1]);

                // Text input (textarea handles its own rendering)
                self.text_input.set_block(
                    Block::new()
                        .title("Custom url (editable)")
                        .borders(Borders::ALL),
                );
                // visually emphasize when focused
                if self.focus == NavigationTarget::TextInput {
                    // There's no direct style on TextArea, but its inner Paragraph will show cursor
                    // We can draw a highlighted border by rendering an extra block behind it
                    let focused_block = Block::new()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Yellow));
                    frame.render_widget(focused_block, chunks[2]);
                }
                // render textarea into its area
                self.text_input.render(chunks[2], frame.buffer_mut());

                // Buttons layout
                let btn_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[3]);

                let submit_block = Block::new().title(" Submit ").borders(Borders::ALL);
                let submit_block = if self.focus == NavigationTarget::SubmitButton {
                    submit_block.style(Style::default().fg(Color::White).bg(Color::Green))
                } else {
                    submit_block
                };
                frame.render_widget(submit_block, btn_chunks[0]);

                let cancel_block = Block::new().title(" Cancel ").borders(Borders::ALL);
                let cancel_block = if self.focus == NavigationTarget::CancelButton {
                    cancel_block.style(Style::default().fg(Color::White).bg(Color::Red))
                } else {
                    cancel_block
                };
                frame.render_widget(cancel_block, btn_chunks[1]);
            })?;
        }

        ratatui::restore();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    #[ignore]
    pub async fn it_works() -> eyre::Result<()> {
        let tui = super::DefaultOrganizationTui::new();
        tui.run().await?;
        Ok(())
    }
}
