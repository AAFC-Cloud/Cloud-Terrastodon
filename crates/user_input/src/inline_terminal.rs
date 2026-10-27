use crate::TerminalBackend;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use eyre::Result;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::TerminalOptions;
use ratatui::Viewport;
use ratatui::backend::CrosstermBackend;
use tracing::error;
use std::io::BufWriter;
use std::io::IsTerminal;
use std::io::Stderr;
use std::io::Write;

/// An inline stderr viewport with log output inserted into scrollback above it.
///
/// The caller must acquire a `TerminalCoordinator` guard before construction,
/// service its controls using `apply_terminal_control`, and close this backend
/// before releasing the guard. Only poll terminal input while `is_active()`.
/// This owns raw mode and rendering, but never enters the alternate screen.
///
/// See <https://ratatui.rs/examples/apps/inline/> for Ratatui's viewport contract.
pub struct InlineTerminal {
    terminal: Option<Terminal<CrosstermBackend<BufWriter<Stderr>>>>,
    height: u16,
}

impl InlineTerminal {
    pub fn supported() -> bool {
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
    }

    pub fn new(height: u16) -> Result<Self> {
        eyre::ensure!(
            Self::supported(),
            "an inline terminal requires interactive stdin and stderr"
        );
        eyre::ensure!(height > 0, "an inline viewport must have a positive height");
        let mut result = Self {
            terminal: None,
            height,
        };
        result.resume()?;
        Ok(result)
    }

    pub fn is_active(&self) -> bool {
        self.terminal.is_some()
    }

    pub fn draw(&mut self, draw: impl FnOnce(&mut Frame<'_>)) -> Result<()> {
        if let Some(terminal) = &mut self.terminal {
            terminal.draw(draw)?;
        }
        Ok(())
    }

    /// Print the tracing subscriber's formatted output above the viewport.
    ///
    /// Only call while active. Acknowledge the corresponding log-buffer cursor
    /// after this succeeds so these records are not printed again on shutdown.
    /// The caller supplies trusted formatter output, with untrusted controls
    /// escaped before styling. ANSI styles and native terminal wrapping are
    /// preserved; the log text is never reconstructed as Ratatui cells.
    pub fn print_logs(&mut self, records: &[String]) -> Result<()> {
        let terminal = self
            .terminal
            .as_mut()
            .ok_or_else(|| eyre::eyre!("inline terminal is suspended"))?;
        if records.is_empty() {
            return Ok(());
        }
        terminal.autoresize()?;
        let origin = terminal.get_frame().area().as_position();
        terminal.clear()?;
        terminal.set_cursor_position(origin)?;
        write_formatted_logs(terminal.backend_mut(), records)?;
        // Native output can wrap or scroll by any number of rows. Re-anchor the
        // existing inline viewport at the actual cursor, with a zero offset in
        // the previous viewport established above. No guessed text widths,
        // terminal mode toggles or terminal-owner handoffs are needed.
        terminal.resize(terminal.size()?.into())?;
        terminal.backend_mut().flush()?;
        Ok(())
    }

    /// Restore the main screen and input mode while retaining log scrollback.
    pub fn close(&mut self) -> Result<()> {
        let Some(mut terminal) = self.terminal.take() else {
            return Ok(());
        };
        // Attempt all restoration operations even if an earlier write fails.
        let origin = terminal.get_frame().area().as_position();
        let clear = terminal.clear();
        let cursor = terminal.show_cursor();
        let column = execute!(terminal.backend_mut(), MoveTo(origin.x, origin.y));
        let flush = terminal.backend_mut().flush();
        let raw = disable_raw_mode();
        clear?;
        cursor?;
        column?;
        flush?;
        raw?;
        Ok(())
    }
}

impl TerminalBackend for InlineTerminal {
    fn is_active(&self) -> bool {
        self.is_active()
    }

    fn suspend(&mut self) -> Result<()> {
        self.close()
    }

    fn resume(&mut self) -> Result<()> {
        if self.is_active() {
            return Ok(());
        }
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(BufWriter::new(std::io::stderr()));
        match Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Inline(self.height),
            },
        ) {
            Ok(terminal) => {
                self.terminal = Some(terminal);
                Ok(())
            }
            Err(error) => {
                let _ = disable_raw_mode();
                Err(error.into())
            }
        }
    }
}

impl Drop for InlineTerminal {
    fn drop(&mut self) {
        if let Err(e) = self.close() {
            error!("failed to close inline terminal: {e:?}");
        }
    }
}

fn write_formatted_logs(mut writer: impl Write, records: &[String]) -> std::io::Result<()> {
    for record in records {
        for line in record.lines() {
            writer.write_all(line.as_bytes())?;
            // Raw mode leaves LF at the current column. Explicit CRLF makes the
            // next formatter line start at column zero on every backend.
            writer.write_all(b"\r\n")?;
        }
    }
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formatter_output_retains_ansi_unicode_and_line_boundaries_in_raw_mode() {
        let records = vec![
            "\x1b[32m   INFO ab界e\u{301}cd\x1b[0m\n    at request.rs:12\n    in request"
                .to_owned(),
            "\x1b[33m   WARN second event\x1b[0m\n".to_owned(),
        ];
        let mut output = Vec::new();
        write_formatted_logs(&mut output, &records).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            concat!(
                "\x1b[32m   INFO ab界e\u{301}cd\x1b[0m\r\n",
                "    at request.rs:12\r\n    in request\r\n",
                "\x1b[33m   WARN second event\x1b[0m\r\n",
            )
        );
    }
}
