use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_command::CancellationScope;
use crossterm::event::Event;
use crossterm::event::EventStream;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use crossterm::terminal;
use eyre::Context;
use eyre::OptionExt;
use eyre::Result;
use futures::StreamExt;
use std::io::IsTerminal;
use std::io::Write;
use std::io::stderr;
use std::io::stdin;

/// Keeps terminal echo disabled until the password prompt is complete.
pub(super) struct BmcHelixPasswordPrompt {
    restore_raw_mode: bool,
}

impl BmcHelixPasswordPrompt {
    pub(super) async fn read(
        server: &BmcHelixServer,
        cancellation: &CancellationScope,
    ) -> Result<String> {
        eyre::ensure!(
            stdin().is_terminal() && stderr().is_terminal(),
            "BMC Helix sign-in requires an interactive terminal"
        );
        let mut prompt = Self::begin(server)?;
        let password = prompt.read_keys(cancellation).await;
        prompt.restore()?;
        password
    }

    fn begin(server: &BmcHelixServer) -> Result<Self> {
        let restore_raw_mode = !terminal::is_raw_mode_enabled()?;
        if restore_raw_mode {
            terminal::enable_raw_mode().wrap_err("disabling echo for the password prompt")?;
        }
        let prompt = Self { restore_raw_mode };
        write!(
            stderr(),
            "BMC Helix password for {} ({}): ",
            server.name,
            server.url
        )?;
        stderr().flush()?;
        Ok(prompt)
    }

    async fn read_keys(&self, cancellation: &CancellationScope) -> Result<String> {
        let mut events = EventStream::new();
        let mut password = String::new();
        loop {
            let event = tokio::select! {
                _ = cancellation.cancelled() => {
                    eyre::bail!("BMC Helix sign-in cancelled");
                }
                event = events.next() => event.ok_or_eyre("terminal input ended")??,
            };
            let Event::Key(key) = event else {
                continue;
            };
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Enter => return Ok(password),
                KeyCode::Esc => {
                    eyre::bail!("BMC Helix sign-in cancelled");
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    eyre::bail!("BMC Helix sign-in cancelled");
                }
                KeyCode::Backspace => {
                    password.pop();
                }
                KeyCode::Char(character) if !character.is_control() => password.push(character),
                _ => {}
            }
        }
    }

    fn restore(&mut self) -> Result<()> {
        if self.restore_raw_mode {
            terminal::disable_raw_mode().wrap_err("restoring terminal echo")?;
            self.restore_raw_mode = false;
        }
        writeln!(stderr())?;
        Ok(())
    }
}

impl Drop for BmcHelixPasswordPrompt {
    fn drop(&mut self) {
        if self.restore_raw_mode {
            // This also runs if automatic cancellation drops the prompt future.
            let _ = terminal::disable_raw_mode();
        }
    }
}
