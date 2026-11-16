use ratatui::Terminal;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::EnterAlternateScreen;
use ratatui::crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::enable_raw_mode;
use ratatui::prelude::CrosstermBackend;
use std::io::BufWriter;
use std::io::Stderr;
use std::io::stderr;

pub(super) type PickerTerminal = Terminal<CrosstermBackend<BufWriter<Stderr>>>;

pub(super) fn enter_terminal() -> eyre::Result<PickerTerminal> {
    enable_raw_mode()?;
    if let Err(error) = execute!(stderr(), EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(error.into());
    }
    let backend = CrosstermBackend::new(BufWriter::new(stderr()));
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => {
            let _ = restore_terminal();
            return Err(error.into());
        }
    };
    if let Err(error) = terminal.clear() {
        let _ = restore_terminal();
        return Err(error.into());
    }
    Ok(terminal)
}

pub(super) fn restore_terminal() -> eyre::Result<()> {
    let raw_mode = disable_raw_mode();
    let alternate_screen = execute!(stderr(), LeaveAlternateScreen);
    match (raw_mode, alternate_screen) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error.into()),
        (Err(raw_mode), Err(alternate_screen)) => Err(eyre::eyre!(
            "failed to restore picker terminal: raw mode: {raw_mode}; alternate screen: {alternate_screen}"
        )),
    }
}
