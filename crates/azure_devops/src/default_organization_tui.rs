use crate::prelude::get_default_organization_url;
use crate::prelude::set_default_organization_url;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::crossterm::event::{self};
use ratatui::layout::Alignment;
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
use std::borrow::Cow;
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

pub struct AzureDevOpsDefaultOrganizationUrlTui {
    text_input: TextArea<'static>,
    focus: NavigationTarget,
    current_value: Option<String>,
    validation: eyre::Result<()>,
}

impl Default for AzureDevOpsDefaultOrganizationUrlTui {
    fn default() -> Self {
        Self {
            text_input: TextArea::new(vec![]),
            focus: NavigationTarget::TextInput,
            current_value: None,
            validation: Ok(()),
        }
    }
}

impl AzureDevOpsDefaultOrganizationUrlTui {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_validation(&mut self) {
        let content = self.text_input.lines().join("");
        // must not be empty
        if content.is_empty() {
            self.validation = Err(eyre::eyre!("Input cannot be empty"));
            return;
        }
        // must start with https://
        if !content.starts_with("https://") {
            self.validation = Err(eyre::eyre!("Input must start with 'https://'"));
            return;
        }

        // must contain a slug
        if content
            .trim_start_matches("https://")
            .trim_end_matches('/')
            .rsplit_once('/')
            .is_none()
        {
            self.validation = Err(eyre::eyre!(
                "Missing org name, expected input in format 'https://dev.azure.com/myorg/'"
            ));
            return;
        }
        self.validation = Ok(());
    }

    pub async fn run(mut self) -> eyre::Result<DefaultOrganizationTuiResponse> {
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
        self.text_input.move_cursor(tui_textarea::CursorMove::End);

        const RECOMMENDED_URL: &str = "https://dev.azure.com/aafc";

        let mut terminal = ratatui::init();
        terminal.clear()?;

        let rtn;

        loop {
            // handle keyboard input
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Esc => {
                        // Exit the TUI (cancel)
                        rtn = DefaultOrganizationTuiResponse::Cancel;
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
                            self.update_validation();
                            if self.validation.is_err() {
                                // invalid input, do not submit
                                self.focus = NavigationTarget::TextInput;
                                continue;
                            }
                            let text = self.text_input.lines().join("");
                            let (base_url, organization_name) =
                                text.trim_end_matches('/').rsplit_once('/').ok_or_else(|| {
                                    eyre::eyre!(
                                        "Expected input in format 'https://dev.azure.com/myorg/'"
                                    )
                                })?;
                            let url =
                                AzureDevOpsOrganizationUrl::try_new(base_url, organization_name)?;
                            rtn = DefaultOrganizationTuiResponse::SetDefaultOrganizationUrl(url);
                            break;
                        }
                        NavigationTarget::CancelButton => {
                            rtn = DefaultOrganizationTuiResponse::Cancel;
                            break;
                        }
                        NavigationTarget::TextInput => {
                            self.focus = self.focus.next();
                        }
                        NavigationTarget::RecommendedUrl => {
                            self.text_input = TextArea::new(vec![RECOMMENDED_URL.to_string()]);
                            self.text_input.move_cursor(tui_textarea::CursorMove::End);
                            self.focus = NavigationTarget::TextInput;
                            self.update_validation();
                        }
                        NavigationTarget::CurrentValue => {
                            if let Some(ref v) = self.current_value {
                                self.text_input = TextArea::new(vec![v.clone()]);
                            } else {
                                self.text_input = TextArea::new(vec!["".to_string()]);
                            }
                            self.text_input.move_cursor(tui_textarea::CursorMove::End);
                            self.focus = NavigationTarget::TextInput;
                            self.update_validation();
                        }
                    },
                    _ if self.focus == NavigationTarget::TextInput => {
                        let changed = self.text_input.input(key);
                        if changed {
                            self.update_validation();
                        }
                    }
                    _ => {}
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
                let recommended = Paragraph::new(Text::from(Span::raw(RECOMMENDED_URL))).block(
                    Block::new()
                        .title("Recommended url (Enter to use)")
                        .borders(Borders::ALL),
                );
                let recommended = if self.focus == NavigationTarget::RecommendedUrl {
                    recommended.style(Style::default().fg(Color::Yellow))
                } else {
                    recommended
                };
                frame.render_widget(recommended, chunks[1]);

                // Text input (textarea handles its own rendering)
                self.text_input.set_block(
                    Block::new()
                        .title(match self.validation.as_ref() {
                            Ok(()) => Cow::Borrowed("Custom url (editable)"),
                            Err(e) => Cow::Owned(format!("Custom url (invalid: {e})")),
                        })
                        .borders(Borders::ALL)
                        .style(
                            match (
                                self.focus == NavigationTarget::TextInput,
                                self.validation.is_ok(),
                            ) {
                                (_, false) => Style::default().fg(Color::Red),
                                (true, true) => Style::default().fg(Color::Yellow),
                                (false, true) => Style::default(),
                            },
                        ),
                );
                // render textarea into its area
                self.text_input.render(chunks[2], frame.buffer_mut());

                // Buttons layout
                let btn_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[3]);

                // Submit button (no title; centered text inside)
                {
                    let focused = self.focus == NavigationTarget::SubmitButton;
                    let mut submit_block = Block::new().borders(Borders::ALL);
                    if focused {
                        submit_block =
                            submit_block.style(Style::default().fg(Color::White).bg(Color::Green));
                    }
                    frame.render_widget(submit_block.clone(), btn_chunks[0]);
                    let inner = submit_block.inner(btn_chunks[0]);
                    let mut submit_label =
                        Paragraph::new(Line::from("Submit")).alignment(Alignment::Center);
                    if focused {
                        submit_label =
                            submit_label.style(Style::default().fg(Color::White).bg(Color::Green));
                    }
                    frame.render_widget(submit_label, inner);
                }

                // Cancel button (no title; centered text inside)
                {
                    let focused = self.focus == NavigationTarget::CancelButton;
                    let mut cancel_block = Block::new().borders(Borders::ALL);
                    if focused {
                        cancel_block =
                            cancel_block.style(Style::default().fg(Color::White).bg(Color::Red));
                    }
                    frame.render_widget(cancel_block.clone(), btn_chunks[1]);
                    let inner = cancel_block.inner(btn_chunks[1]);
                    let mut cancel_label =
                        Paragraph::new(Line::from("Cancel")).alignment(Alignment::Center);
                    if focused {
                        cancel_label =
                            cancel_label.style(Style::default().fg(Color::White).bg(Color::Red));
                    }
                    frame.render_widget(cancel_label, inner);
                }
            })?;
        }

        ratatui::restore();
        Ok(rtn)
    }
}

#[must_use = "To perform any changes, the handle fn must be called"]
pub enum DefaultOrganizationTuiResponse {
    SetDefaultOrganizationUrl(AzureDevOpsOrganizationUrl),
    Cancel,
}
impl DefaultOrganizationTuiResponse {
    pub async fn handle(self) -> eyre::Result<()> {
        match self {
            DefaultOrganizationTuiResponse::SetDefaultOrganizationUrl(url) => {
                set_default_organization_url(url).await?;
                Ok(())
            }
            DefaultOrganizationTuiResponse::Cancel => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    #[test_log::test(tokio::test)]
    #[ignore]
    pub async fn it_works() -> eyre::Result<()> {
        let tui = super::AzureDevOpsDefaultOrganizationUrlTui::new();
        tui.run().await?.handle().await?;
        Ok(())
    }
}
