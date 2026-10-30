use cloud_terrastodon_source::SourceHuntProgress;
use cloud_terrastodon_user_input::TerminalBackend;
use cloud_terrastodon_user_input::TerminalControl;
use cloud_terrastodon_user_input::TerminalCoordinator;
use cloud_terrastodon_user_input::TerminalGuard;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use eyre::Result;
use eyre::WrapErr;
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use std::io::BufWriter;
use std::io::IsTerminal;
use std::io::Stderr;
use std::io::Write;
use std::io::stderr;

/// Owns the stderr progress screen and its coordinated terminal lifetime.
///
/// Raw input mode is deliberately unnecessary: application cancellation keeps
/// handling Ctrl-C while this screen only writes progress to stderr. The guard
/// remains private so restoration always precedes releasing terminal ownership.
pub(crate) struct SourceHuntProgressTerminal {
    terminal: Terminal<CrosstermBackend<BufWriter<Stderr>>>,
    guard: Option<TerminalGuard>,
    // Set before entering the screen so partial setup is restored by Drop too.
    active: bool,
}

impl SourceHuntProgressTerminal {
    pub(crate) fn is_supported() -> bool {
        stderr().is_terminal()
            && std::env::var_os("TERM")
                .is_none_or(|term| !term.to_string_lossy().eq_ignore_ascii_case("dumb"))
    }

    pub(crate) async fn new() -> Result<Self> {
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            eyre::eyre!("source hunt progress needs the application terminal coordinator")
        })?;
        let guard = coordinator.acquire().await?;
        let terminal = match Terminal::new(CrosstermBackend::new(BufWriter::new(stderr()))) {
            Ok(terminal) => terminal,
            Err(error) => {
                guard.release().await?;
                return Err(error).wrap_err("initializing source hunt progress terminal");
            }
        };
        let mut progress = Self {
            terminal,
            guard: Some(guard),
            active: false,
        };
        progress.resume()?;
        Ok(progress)
    }

    /// Draw at regular intervals even when the scan has no new observations so
    /// nested terminal owners can suspend and resume this screen promptly.
    pub(crate) fn draw(&mut self, progress: &SourceHuntProgress) -> Result<()> {
        while let Some(control) = self
            .guard
            .as_mut()
            .ok_or_else(|| eyre::eyre!("progress terminal has already released its guard"))?
            .try_next_control()?
        {
            match &control {
                TerminalControl::Suspend { .. } => self.suspend()?,
                TerminalControl::Resume { .. } => self.resume()?,
                TerminalControl::Poisoned { message } => {
                    eyre::bail!("terminal coordinator poisoned: {message}");
                }
            }
            self.guard
                .as_ref()
                .ok_or_else(|| eyre::eyre!("progress terminal has already released its guard"))?
                .acknowledge(&control)
                .inspect_err(|error| {
                    self.poison(format!("progress terminal handoff failed: {error}"))
                })?;
        }
        if self.active {
            self.terminal
                .draw(|frame| draw_progress(frame, progress))
                .wrap_err("drawing source hunt progress")?;
        }
        Ok(())
    }

    pub(crate) async fn finish(mut self) -> Result<()> {
        let restore = self.suspend();
        let release = self
            .guard
            .take()
            .ok_or_else(|| eyre::eyre!("progress terminal has already released its guard"))?
            .release()
            .await;
        restore?;
        release
    }

    fn poison(&self, message: String) {
        if let Some(guard) = &self.guard {
            guard.poison(message);
        }
    }
}

impl TerminalBackend for SourceHuntProgressTerminal {
    fn is_active(&self) -> bool {
        self.active
    }

    fn suspend(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        // Keep Ratatui's cursor state synchronized so its own Drop does not write
        // after the coordinator has handed the terminal back to another owner.
        let cursor = self.terminal.show_cursor();
        let screen = execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .and_then(|()| self.terminal.backend_mut().flush());
        cursor
            .and(screen)
            .inspect_err(|error| {
                self.poison(format!("restoring source hunt terminal failed: {error}"))
            })
            .wrap_err("restoring source hunt progress terminal")?;
        self.active = false;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }
        self.active = true;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)
            .and_then(|()| self.terminal.hide_cursor())
            .and_then(|()| self.terminal.clear())
            .inspect_err(|error| {
                self.poison(format!("initializing source hunt terminal failed: {error}"))
            })
            .wrap_err("opening source hunt progress screen")?;
        Ok(())
    }
}

impl Drop for SourceHuntProgressTerminal {
    fn drop(&mut self) {
        // Also runs when the invoking future is cancelled or a render panics.
        // Never panic during cleanup; suspend poisons unsafe terminal ownership.
        let _ = self.suspend();
    }
}

fn draw_progress(frame: &mut Frame<'_>, progress: &SourceHuntProgress) {
    let active_rows = u16::try_from(progress.active_paths.len().clamp(1, 6)).unwrap_or(6);
    let sections = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(active_rows + 2),
        Constraint::Min(4),
        Constraint::Length(1),
    ])
    .split(frame.area());
    let columns = Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(sections[2]);
    let heading = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    frame.render_widget(
        Paragraph::new(format!(
            "Directories: {}   Files: {}   Repositories: {}   Errors: {}",
            progress.directories_scanned,
            progress.files_scanned,
            progress.repositories_found,
            progress.errors,
        ))
        .block(Block::bordered().title("Source hunt")),
        sections[0],
    );

    let active_paths = if progress.active_paths.is_empty() {
        vec![Line::from("Waiting for directory workers…")]
    } else {
        progress
            .active_paths
            .iter()
            .take(usize::from(active_rows))
            .map(|path| {
                Line::from(terminal_text(
                    &path.display().to_string(),
                    usize::from(sections[1].width.saturating_sub(2)),
                ))
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(active_paths).block(Block::bordered().title("Currently analyzing")),
        sections[1],
    );

    let mut repositories: Vec<_> = progress.top_repositories.iter().collect();
    repositories.sort_unstable_by(|left, right| {
        right
            .matched_files
            .cmp(&left.matched_files)
            .then_with(|| left.path.cmp(&right.path))
    });
    let repository_rows = repositories
        .into_iter()
        .take(usize::from(columns[0].height.saturating_sub(3)))
        .map(|repository| {
            Row::new([
                terminal_text(
                    &repository.path.display().to_string(),
                    usize::from(columns[0].width.saturating_sub(23)),
                ),
                repository.matched_files.to_string(),
                repository.total_files.to_string(),
            ])
        });
    frame.render_widget(
        Table::new(
            repository_rows,
            [
                Constraint::Min(1),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(Row::new(["Repository", "Matched", "Files"]).style(heading))
        .block(Block::bordered().title("Top repositories by matching files")),
        columns[0],
    );

    let mut extensions: Vec<_> = progress.extension_counts.iter().collect();
    extensions
        .sort_unstable_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    let extension_rows = extensions
        .into_iter()
        .take(usize::from(columns[1].height.saturating_sub(3)))
        .map(|(extension, count)| {
            Row::new([
                if extension == "." {
                    "(no extension)".to_owned()
                } else {
                    terminal_text(extension, usize::from(columns[1].width.saturating_sub(14)))
                },
                count.to_string(),
            ])
        });
    frame.render_widget(
        Table::new(extension_rows, [Constraint::Min(1), Constraint::Length(12)])
            .header(Row::new(["Extension", "Files"]).style(heading))
            .block(Block::bordered().title("Extension totals")),
        columns[1],
    );
    frame.render_widget(Paragraph::new("Ctrl-C cancels the scan"), sections[3]);
}

/// Paths and extensions are local, untrusted text. Strip terminal controls and
/// retain the useful end of deep paths when a panel cannot fit their full text.
fn terminal_text(value: &str, width: usize) -> String {
    let safe: String = value
        .chars()
        .map(|character| {
            if character.is_control() {
                '�'
            } else {
                character
            }
        })
        .collect();
    let length = safe.chars().count();
    if length <= width {
        safe
    } else if width == 0 {
        String::new()
    } else {
        std::iter::once('…')
            .chain(safe.chars().skip(length - width + 1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::terminal_text;

    #[test]
    fn untrusted_paths_cannot_write_terminal_controls() {
        assert_eq!(
            terminal_text("repo\u{1b}[2J\nfile.ts", 80),
            "repo�[2J�file.ts"
        );
    }

    #[test]
    fn deep_paths_keep_their_last_components() {
        assert_eq!(terminal_text("parent/child/file.ts", 14), "…child/file.ts");
        assert_eq!(terminal_text("path", 0), "");
    }
}
