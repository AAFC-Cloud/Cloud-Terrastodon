use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyEventKind;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

pub struct MessageBoxTui {
    message: Text<'static>,
    title: Option<Line<'static>>,
}
impl MessageBoxTui {
    pub fn new(message: impl Into<Text<'static>>) -> Self {
        Self {
            message: message.into(),
            title: None,
        }
    }

    #[allow(dead_code)]
    pub fn title(mut self, title: impl Into<Line<'static>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Run the message box until the user acknowledges it.
    ///
    /// Keys: Enter / Space close the dialog (Esc / q also allowed as shortcut).
    pub fn run(self) -> eyre::Result<()> {
        let mut terminal = ratatui::init();
        terminal.clear()?;

        // Precompute lines count for sizing hints
        let msg_line_count = self.message.lines.len();

        loop {
            // Input handling
            if event::poll(std::time::Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }

            // Draw
            terminal.draw(|f| {
                let area = f.area();

                // Centered layout: outer block full screen, inner vertical layout
                let outer_block = Block::new()
                    .borders(Borders::ALL)
                    .title(self.title.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::Blue));
                let inner = outer_block.inner(area);
                f.render_widget(outer_block, area);

                // dynamic button row height (3 lines) + message height
                let constraints = [
                    Constraint::Min((msg_line_count as u16).max(1)),
                    Constraint::Length(3),
                ];
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(constraints)
                    .split(inner);

                // Message paragraph, centered horizontally
                let msg_para = Paragraph::new(self.message.clone()).alignment(Alignment::Left);
                f.render_widget(msg_para.block(Block::new()), chunks[0]);

                // Single OK button
                let btn_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .split(chunks[1]);

                let ok_block = Block::new()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White).bg(Color::Green));
                f.render_widget(ok_block.clone(), btn_chunks[0]);
                let inner_btn = ok_block.inner(btn_chunks[0]);
                let label = Paragraph::new(Line::from(" OK "))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White).bg(Color::Green));
                f.render_widget(label, inner_btn);
            })?;
        }

        ratatui::restore();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::MessageBoxTui;

    #[test]
    #[ignore]
    fn manual_run() -> eyre::Result<()> {
        MessageBoxTui::new("Hello world from MessageBoxTui. Press Enter to dismiss.")
            .title("Info")
            .run()?;
        Ok(())
    }
}
