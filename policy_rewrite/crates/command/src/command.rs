use anyhow::anyhow;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use async_recursion::async_recursion;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsStr;
use std::os::windows::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use tokio::fs::create_dir_all;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    stdout: String,
    stderr: String,
    status: u32,
}
impl CommandOutput {
    fn success(&self) -> bool {
        ExitStatus::from_raw(self.status).success()
    }
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
            status: match value.status.code().unwrap_or(1) {
                x if x < 0 => 1,
                x => x as u32,
            },
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
    cache_dir: Option<PathBuf>,
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
            cache_dir: self.cache_dir.clone(),
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
            cache_dir: None,
        }
    }
    pub fn context(&self) -> String {
        format!("{} {}", self.kind.program(), self.args.join(" "))
    }
    pub fn use_cache_dir(&mut self, cache: Option<PathBuf>) -> &mut CommandBuilder {
        self.cache_dir = cache;
        self
    }
    pub fn use_retry_behaviour(&mut self, behaviour: RetryBehaviour) -> &mut CommandBuilder {
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

    pub async fn get_cached_output(&self) -> Result<Option<CommandOutput>> {
        // Short circuit if no cache
        let Some(ref cache_dir) = self.cache_dir else {
            return Ok(None);
        };
        if !cache_dir.exists() {
            return Ok(None);
        }

        // Prepare destination object
        let mut output = CommandOutput {
            status: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        let mut status = String::new();

        // Prepare for file reading, with an expectation that context content matches
        let mut files = [
            ("context.txt", &mut String::new(), Some(self.context())),
            ("stdout.json", &mut output.stdout, None),
            ("stderr.json", &mut output.stderr, None),
            ("status.txt", &mut status, None),
        ];

        // For each file
        for (ref file_name, ref mut file_contents, ref expected_contents) in files.iter_mut() {
            // Open the file
            let path = cache_dir.join(file_name);
            let mut file = OpenOptions::new()
                .read(true)
                .open(&path)
                .await
                .context("opening file")
                .context(path.to_string_lossy().into_owned())?;

            // Read the file
            file.read_to_string(file_contents)
                .await
                .context("reading file")
                .context(path.to_string_lossy().into_owned())?;

            // If an expectation is present, validate it
            if let Some(ref expected_contents) = expected_contents
                && file_contents != &expected_contents
            {
                return Err(anyhow!("Cache context mismatch")
                    .context(format!("found: {}", file_contents))
                    .context(format!("expected: {}", expected_contents)));
            }
        }

        // Finish moving info into destination
        output.status = status.parse().context("parsing status").context(status)?;

        // Return!
        Ok(Some(output))
    }

    pub async fn put_cached_output(&self, output: &CommandOutput) -> Result<()> {
        // Ensure cache dir is known
        let Some(ref cache_dir) = self.cache_dir else {
            return Err(anyhow!("Cache destination unknown"));
        };

        // Validate directory presence
        if !cache_dir.exists() {
            create_dir_all(cache_dir)
                .await
                .context("creating directories")?;
        } else if !cache_dir.is_dir() {
            return Err(anyhow!("Cache destination isn't a directory")
                .context(cache_dir.clone().to_string_lossy().into_owned()));
        }

        // Prepare write contents
        let files = [
            ("context.txt", &self.context()),
            ("stdout.json", &output.stdout),
            ("stderr.json", &output.stderr),
            ("status.txt", &output.status.to_string()),
        ];

        // Write to files
        for (file_name, file_contents) in files {
            // Open file
            let path = cache_dir.join(file_name);
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await
                .context("opening file")
                .context(path.to_string_lossy().into_owned())?;

            // Write content
            file.write_all(file_contents.as_bytes())
                .await
                .context("writing file")
                .context(path.to_string_lossy().into_owned())?;
        }

        Ok(())
    }

    #[async_recursion]
    pub async fn run_raw(&mut self) -> Result<CommandOutput> {
        // Check cache
        match self.get_cached_output().await {
            Ok(None) => {}
            Ok(Some(output)) => return Ok(output),
            Err(e) => {
                eprintln!("Failed to load from cache: {:?}", e);
            }
        }

        // Launch command
        let child = self.command.spawn()?;

        // Wait for it to finish
        let output = child.wait_with_output().await?;

        // Convert to our output type
        let output: CommandOutput = output.try_into()?;

        // Return if errored
        if !output.success() {
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
                    retry.use_retry_behaviour(RetryBehaviour::Fail);
                    let output = retry.run_raw().await;

                    // Return the result
                    return output;
                }
                (_, o) => {
                    return Err(Error::from(o).context(self.context()));
                }
            }
        }

        // Write happy results to the cache
        if output.success() && self.cache_dir.is_some() {
            println!("Writing command results to cache file...");
            if let Err(e) = self.put_cached_output(&output).await {
                eprintln!("Encountered problem saving cache: {:?}", e);
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
    use std::str::FromStr;

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
    async fn it_works_cached() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["--version"])
            .use_cache_dir(Some(PathBuf::from_str(r"ignore\version")?))
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
