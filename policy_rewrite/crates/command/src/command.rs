use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use async_recursion::async_recursion;
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::process::Output;
use std::process::Stdio;
use tokio::process::Command;

use crate::errors::dump_to_ignore_file;

#[derive(Clone)]
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

#[derive(Debug, Clone)]
pub struct CommandOutput {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus,
}
impl std::error::Error for CommandOutput {}
impl std::fmt::Display for CommandOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "status={}\nstdout={}\nstderr={}",
            self.status, self.stdout, self.stderr
        ))
    }
}
impl TryFrom<Output> for CommandOutput {
    type Error = Error;
    fn try_from(value: Output) -> Result<Self> {
        Ok(CommandOutput {
            stdout: String::from_utf8(value.stdout)?,
            stderr: String::from_utf8(value.stderr)?,
            status: value.status,
        })
    }
}

#[derive(Clone, Copy)]
pub enum RetryBehaviour {
    Fail,
    Reauth,
}
pub struct CommandBuilder {
    kind: CommandKind,
    command: Command,
    args: Vec<String>,
    retry_behaviour: RetryBehaviour,
}
impl Clone for CommandBuilder {
    fn clone(&self) -> Self {
        let mut command = Command::new(self.kind.program());
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.args(&self.args);
        Self {
            kind: self.kind.clone(),
            command,
            args: self.args.clone(),
            retry_behaviour: self.retry_behaviour,
        }
    }
}
impl CommandBuilder {
    pub fn new(kind: CommandKind) -> CommandBuilder {
        let mut command = Command::new(kind.program());
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let args = vec![];
        CommandBuilder {
            kind,
            command,
            args,
            retry_behaviour: RetryBehaviour::Reauth,
        }
    }
    pub fn with_retry_behaviour(&mut self, behaviour: RetryBehaviour) -> &mut CommandBuilder {
        self.retry_behaviour = behaviour;
        self
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
        self.args.push(arg.as_ref().to_string_lossy().to_string());
        self.command.arg(arg);
        self
    }

    #[async_recursion]
    pub async fn run_raw(&mut self) -> Result<CommandOutput> {
        // Launch command
        let child = self.command.spawn()?;

        // Wait for it to finish
        let output = child.wait_with_output().await?;

        // Convert to our output type
        let output: CommandOutput = output.try_into()?;

        // Return if errored
        if !output.status.success() {
            match (self.retry_behaviour, output) {
                (RetryBehaviour::Reauth, output)
                    if [
                        "AADSTS70043",
                        "No subscription found. Run 'az account set' to select a subscription.",
                    ]
                    .into_iter()
                    .any(|x| output.stderr.contains(x)) =>
                {
                    // Let the user know
                    println!("Refreshing credential, user action required in a moment...");

                    // Perform login command
                    CommandBuilder::new(CommandKind::AzureCLI)
                        .arg("login")
                        .run_raw()
                        .await?;

                    // Retry the failed command, no further retries
                    println!("Retrying command with refreshed credential...");
                    let mut retry = self.clone();
                    retry.with_retry_behaviour(RetryBehaviour::Fail);
                    return retry.run_raw().await;
                }
                (_, o) => {
                    return Err(Error::from(o).context(format!(
                        "{} {}",
                        self.kind.program(),
                        self.args.join(" ")
                    )));
                }
            }
        }

        // Return success
        Ok(output)
    }

    #[async_recursion]
    pub async fn run<T: DeserializeOwned>(&mut self) -> Result<T> {
        // Get stdout
        let output = self.run_raw().await?;

        // Parse
        match serde_json::from_str(&output.stdout) {
            Ok(results) => Ok(results),
            Err(e) => {
                let context = dump_to_ignore_file(&output.to_string())?;
                Err(e)
                    .context("deserializing")
                    .context(format!("dumped to {:?}", context))
            }
        }
    }
}

/// These tests require user interaction and change the state of the system!
/// 
/// You have been warned.
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["--version"])
            .run_raw()
            .await?;
        println!("{}", result);
        Ok(())
    }

    #[tokio::test]
    async fn login() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["login"])
            .run_raw()
            .await?;
        println!("{}", result);
        Ok(())
    }

    #[tokio::test]
    async fn logout() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["logout"])
            .run_raw()
            .await;
        println!("{:?}", result);
        Ok(())
    }

    #[tokio::test]
    async fn user() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["ad", "signed-in-user", "show"])
            .run_raw()
            .await;
        println!("{:?}", result);
        Ok(())
    }
    #[tokio::test]
    async fn reauth() -> Result<()> {
        println!("Logging out...");
        let logout_result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["logout"])
            .run_raw()
            .await;
        match logout_result {
            Ok(msg) => println!("{}", msg),
            Err(e) => match e.downcast_ref::<CommandOutput>() {
                Some(CommandOutput { stderr, .. })
                    if stderr.contains("ERROR: There are no active accounts.") =>
                {
                    println!("Already logged out!")
                }
                _ => {
                    return Err(e).context("unknown logout failure");
                }
            },
        }
        println!("Performing command, it should prompt for login...");
        println!(
            "{}",
            CommandBuilder::new(CommandKind::AzureCLI)
                .args(["ad", "signed-in-user", "show"])
                .run_raw()
                .await?
        );
        Ok(())
    }
}
