use cloud_terrastodon_user_input::TerminalBackend;
use cloud_terrastodon_user_input::TerminalCoordinator;
use eyre::Result;
use ratatui::DefaultTerminal;

pub(crate) fn init_terminal(coordinator: &TerminalCoordinator) -> Result<DefaultTerminal> {
    ratatui::try_init().map_err(|error| {
        coordinator.poison(format!("Ratatui terminal setup failed: {error}"));
        match ratatui::try_restore() {
            Ok(()) => eyre::eyre!("Ratatui terminal setup failed: {error}"),
            Err(cleanup_error) => eyre::eyre!(
                "Ratatui terminal setup failed: {error}; cleanup also failed: {cleanup_error}"
            ),
        }
    })
}

pub(crate) fn restore_terminal(coordinator: &TerminalCoordinator) -> Result<()> {
    ratatui::try_restore().map_err(|error| {
        coordinator.poison(format!("Ratatui terminal restoration failed: {error}"));
        error.into()
    })
}

pub(crate) struct RatatuiTerminalBackend<'a> {
    pub(crate) terminal: &'a mut Option<DefaultTerminal>,
    pub(crate) coordinator: &'a TerminalCoordinator,
}

impl TerminalBackend for RatatuiTerminalBackend<'_> {
    fn is_active(&self) -> bool {
        self.terminal.is_some()
    }

    fn suspend(&mut self) -> Result<()> {
        if self.terminal.take().is_some() {
            restore_terminal(self.coordinator)?;
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        if self.terminal.is_none() {
            *self.terminal = Some(init_terminal(self.coordinator)?);
        }
        Ok(())
    }
}
