use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use std::io;
use tui_textarea::Input;
use tui_textarea::Key;
use tui_textarea::TextArea;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_mask_char('\u{2022}'); //U+2022 BULLET (•)
    textarea.set_placeholder_text("Please enter your password");
    let constraints = [Constraint::Length(3), Constraint::Min(1)];
    let layout = Layout::default().constraints(constraints);
    textarea.set_style(Style::default().fg(Color::LightGreen));
    textarea.set_block(Block::default().borders(Borders::ALL).title("Password"));

    loop {
        term.draw(|f| {
            let chunks = layout.split(f.area());
            f.render_widget(&textarea, chunks[0]);
        })?;

        match crossterm::event::read()?.into() {
            Input {
                key: Key::Esc | Key::Enter,
                ..
            } => break,
            input => {
                if textarea.input(input) {
                    // When the input modified its text, validate the text content
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    term.show_cursor()?;

    println!("Input: {:?}", textarea.lines()[0]);
    Ok(())
}
