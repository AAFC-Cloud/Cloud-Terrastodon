use super::picker_terminal::PickerTerminal;
use super::picker_terminal::enter_terminal;
use super::picker_terminal::restore_terminal;
use crate::TerminalBackend;

pub(super) struct PickerTerminalBackend<'a> {
    pub(super) terminal: &'a mut Option<PickerTerminal>,
}

impl TerminalBackend for PickerTerminalBackend<'_> {
    fn is_active(&self) -> bool {
        self.terminal.is_some()
    }

    fn suspend(&mut self) -> eyre::Result<()> {
        if self.terminal.take().is_some() {
            restore_terminal()?;
        }
        Ok(())
    }

    fn resume(&mut self) -> eyre::Result<()> {
        if self.terminal.is_none() {
            *self.terminal = Some(enter_terminal()?);
        }
        Ok(())
    }
}
