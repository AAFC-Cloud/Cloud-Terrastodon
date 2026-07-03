use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
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
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .title("Crossterm Popup Example"),
    );

    let area = Rect {
        width: 40,
        height: 5,
        x: 5,
        y: 5,
    };
    textarea.set_style(Style::default().fg(Color::Yellow));
    textarea.set_placeholder_style(Style::default());
    textarea.set_placeholder_text("prompt message");
    loop {
        term.draw(|f| {
            f.render_widget(&textarea, area);
        })?;
        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            input => {
                textarea.input(input);
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    println!("Lines: {:?}", textarea.lines());
    Ok(())
}
