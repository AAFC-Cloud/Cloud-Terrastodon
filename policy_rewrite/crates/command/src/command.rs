use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::ffi::OsStr;
use std::process::Stdio;
use tokio::process::Child;
use tokio::process::Command;

use crate::errors::dump_to_ignore_file;

pub enum CommandKind {
    AzureCLI,
}
impl CommandKind {
    fn program(&self) -> &'static str {
        match self {
            CommandKind::AzureCLI => "az.cmd",
        }
    }
}
pub struct CommandBuilder {
    command: Command,
    context: Vec<String>,
}
impl CommandBuilder {
    pub fn new(kind: CommandKind) -> CommandBuilder {
        let mut command = Command::new(kind.program());
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        let context = vec![kind.program().to_string()];
        CommandBuilder { command, context }
    }
    pub fn args<I, S>(&mut self, args: I) -> &mut CommandBuilder
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut CommandBuilder {
        self.context
            .push(arg.as_ref().to_string_lossy().to_string());
        self.command.arg(arg);
        self
    }

    pub async fn run<T: DeserializeOwned>(mut self) -> Result<T> {
        // Launch command
        let child = self.command.spawn()?;

        // Wait for it to finish
        let output = child.wait_with_output().await?;

        // Return if errored
        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Error::msg(error_message).context(self.context.join(" ")));
        }

        // Get stdout
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse
        match serde_json::from_str(&stdout) {
            Ok(results) => Ok(results),
            Err(e) => {
                let context = dump_to_ignore_file(&stdout)?;
                Err(e)
                    .context("deserializing")
                    .context(format!("dumped to {:?}", context))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI).args(["--version"]);
    }
}
