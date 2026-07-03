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

fn validate(textarea: &mut TextArea) -> bool {
    if let Err(err) = textarea.lines()[0].parse::<f64>() {
        textarea.set_style(Style::default().fg(Color::LightRed));
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Color::LightRed)
                .title(format!("ERROR: {}", err)),
        );
        false
    } else {
        textarea.set_style(Style::default().fg(Color::LightGreen));
        textarea.set_block(
            Block::default()
                .border_style(Color::LightGreen)
                .borders(Borders::ALL)
                .title("OK"),
        );
        true
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_text("Enter a valid float (e.g. 1.56)");
    let layout = Layout::default().constraints([Constraint::Length(3), Constraint::Min(1)]);
    let mut is_valid = validate(&mut textarea);

    loop {
        term.draw(|f| {
            let chunks = layout.split(f.area());
            f.render_widget(&textarea, chunks[0]);
        })?;

        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            Input {
                key: Key::Enter, ..
            } if is_valid => break,
            Input {
                key: Key::Char('m'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => {}
            input => {
                // TextArea::input returns if the input modified its text
                if textarea.input(input) {
                    is_valid = validate(&mut textarea);
                }
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

    println!("Input: {:?}", textarea.lines()[0]);
    Ok(())
}
