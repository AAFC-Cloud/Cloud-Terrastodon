use anyhow::bail;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use async_recursion::async_recursion;
use pathing_types::Existy;
use pathing_types::IgnoreDir;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::windows::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use std::time::Duration;
use tempfile::Builder;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;

#[derive(Clone, Default, Debug)]
pub enum CommandKind {
    #[default]
    Echo,
    Pause,
    AzureCLI,
    Tofu,
    VSCode,
}
impl CommandKind {
    fn program(&self) -> &'static str {
        match self {
            CommandKind::Echo => "pwsh",
            CommandKind::Pause => "pwsh",
            CommandKind::AzureCLI => "az.cmd",
            CommandKind::Tofu => "tofu.exe",
            CommandKind::VSCode => "code.cmd",
        }
    }
    fn apply_args_and_envs(&self, this: &CommandBuilder, cmd: &mut Command) {
        match self {
            CommandKind::Echo => {
                cmd.args([
                    "-NoProfile",
                    "-Command",
                    "Write-Host -ForegroundColor Green $env:value",
                ]);
                cmd.env("value", this.args.join(" "));
            }
            CommandKind::Pause => {
                cmd.args(["-NoProfile", "-Command", "Pause"]);
            }
            _ => {
                cmd.args(&this.args);
                cmd.envs(&this.env);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: u32,
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

#[derive(Clone, Copy, Default, Debug)]
pub enum RetryBehaviour {
    Fail,
    #[default]
    Reauth,
}
#[derive(Clone, Copy, Default, Debug)]
pub enum OutputBehaviour {
    Display,
    #[default]
    Capture,
}

#[derive(Debug, Default, Clone)]
pub enum CacheBehaviour {
    #[default]
    None,
    Recommended,
    FileBased(PathBuf),
}

#[derive(Debug, Default, Clone)]
pub struct CommandBuilder {
    kind: CommandKind,
    args: Vec<String>,
    env: HashMap<String, String>,
    run_dir: Option<PathBuf>,
    retry_behaviour: RetryBehaviour,
    output_behaviour: OutputBehaviour,
    cache_dir: Option<PathBuf>,
    should_announce: bool,
    timeout: Option<Duration>,
}
impl CommandBuilder {
    pub fn new(kind: CommandKind) -> CommandBuilder {
        let mut cmd = CommandBuilder::default();
        cmd.use_command(kind);
        cmd
    }
    pub fn use_command(&mut self, kind: CommandKind) -> &mut Self {
        self.kind = kind;
        self
    }
    pub fn use_cache_dir(&mut self, cache: impl AsRef<Path>) -> &mut Self {
        self.cache_dir = Some(IgnoreDir::Commands.join(cache));
        self
    }
    pub fn use_run_dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.run_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    pub fn use_retry_behaviour(&mut self, behaviour: RetryBehaviour) -> &mut Self {
        self.retry_behaviour = behaviour;
        self
    }
    pub fn use_output_behaviour(&mut self, behaviour: OutputBehaviour) -> &mut Self {
        self.output_behaviour = behaviour;
        self
    }
    pub fn use_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.arg(arg);
        }
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.as_ref().to_string_lossy().to_string());
        self
    }

    pub fn env(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> &mut Self {
        self.env
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    pub fn should_announce(&mut self, value: bool) -> &mut Self {
        self.should_announce = value;
        self
    }

    pub fn summarize(&self) -> String {
        format!("{} {}", self.kind.program(), self.args.join(" "))
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
            ("context.txt", &mut String::new(), Some(self.summarize())),
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
                .context(format!("opening file {}", path.display()))?;

            // Read the file
            file.read_to_string(file_contents)
                .await
                .context(format!("reading file {}", path.display()))?;

            // If an expectation is present, validate it
            if let Some(ref expected_contents) = expected_contents
                && file_contents != &expected_contents
            {
                bail!(
                    "Cache context mismatch, found: {}, expected: {}",
                    file_contents,
                    expected_contents
                );
            }
        }

        // Finish moving info into destination
        output.status = status.parse().context(format!("parsing status: {status}"))?;

        // Return!
        Ok(Some(output))
    }

    pub async fn write_output(&self, output: &CommandOutput, parent_dir: &PathBuf) -> Result<()> {
        debug!("Writing command results to {}", parent_dir.display());

        // Validate directory presence
        parent_dir.ensure_dir_exists().await?;

        // Prepare write contents
        let files = [
            ("context.txt", &self.summarize()),
            ("stdout.json", &output.stdout),
            ("stderr.json", &output.stderr),
            ("status.txt", &output.status.to_string()),
        ];

        // Write to files
        for (file_name, file_contents) in files {
            // Open file
            let path = parent_dir.join(file_name);
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)
                .await
                .context(format!(
                    "opening file {}",
                    path.to_string_lossy().into_owned()
                ))?;

            // Write content
            file.write_all(file_contents.as_bytes())
                .await
                .context(format!(
                    "writing file {}",
                    path.to_string_lossy().into_owned()
                ))?;
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
                error!("Failed to load from cache: {:?}", e);
            }
        }

        let mut command = Command::new(self.kind.program());
        match self.output_behaviour {
            OutputBehaviour::Capture => {
                command.stdin(Stdio::piped());
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            OutputBehaviour::Display => {}
        }

        self.kind.clone().apply_args_and_envs(self, &mut command);

        if let Some(ref dir) = self.run_dir {
            command.current_dir(dir);
        }

        // Announce launch
        if self.should_announce {
            info!("Running {}", self.summarize());
        }

        // Launch command
        command.kill_on_drop(true);
        let child = command.spawn()?;

        // Wait for it to finish
        let output: CommandOutput = match timeout(
            self.timeout.unwrap_or(Duration::MAX),
            child.wait_with_output(),
        )
        .await
        {
            Ok(result) => result?.try_into()?,
            Err(elapsed) => {
                bail!("Command timeout, {elapsed:?}: {}", self.summarize());
            }
        };

        // // Convert to our output type
        // let output: CommandOutput = output.try_into()?;

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
                    warn!("Command failed due to bad auth. Refreshing credential, user action required in a moment...");

                    // Perform login command
                    CommandBuilder::new(CommandKind::AzureCLI)
                        .arg("login")
                        .run_raw()
                        .await?;

                    // Retry the failed command, no further retries
                    info!("Retrying command with refreshed credential...");
                    let mut retry = self.clone();
                    retry.use_retry_behaviour(RetryBehaviour::Fail);
                    let output = retry.run_raw().await;

                    // Return the result
                    return output;
                }
                (_, o) => {
                    return Err(Error::from(o).context(format!(
                        "Command did not execute successfully: {}",
                        self.summarize()
                    )));
                }
            }
        }

        // Write happy results to the cache
        if output.success()
            && let Some(ref cache_dir) = self.cache_dir
        {
            if let Err(e) = self.write_output(&output, cache_dir).await {
                error!("Encountered problem saving cache: {:?}", e);
            }
        }

        // Return success
        Ok(output)
    }

    // #[async_recursion]
    pub async fn run<T>(&mut self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Get stdout
        let output = self.run_raw().await?;

        // Parse
        match serde_json::from_str(&output.stdout) {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = match self.cache_dir {
                    None => IgnoreDir::Commands.join("failed"),
                    Some(ref x) => x.join("failed"),
                };
                dir.ensure_dir_exists().await?;
                let dir = Builder::new().prefix("temp_").tempdir_in(dir)?.into_path();
                self.write_output(&output, &dir).await?;
                Err(e).context(format!(
                    "deserializing {} failed, dumped to {:?}",
                    self.summarize(),
                    dir
                ))
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
    async fn echo() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::Echo)
            .use_output_behaviour(OutputBehaviour::Display)
            .args(["Ahoy,", "world!"])
            .run_raw()
            .await?;
        println!("{}", result);
        Ok(())
    }
    #[tokio::test]
    async fn pwsh() -> Result<()> {
        let result = Command::new("pwsh.exe")
            .args(["-NoProfile", "-Command", "echo \"got $($env:a)\""])
            .env("a", "b$(2+2)")
            .spawn()?
            .wait_with_output()
            .await?;
        println!("{:?}", result);
        Ok(())
    }
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
            .use_cache_dir("version")
            .run_raw()
            .await?;
        println!("{}", result);
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
    #[ignore]
    async fn login() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["login"])
            .run_raw()
            .await?;
        println!("{}", result);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn logout() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["logout"])
            .run_raw()
            .await;
        println!("{:?}", result);
        Ok(())
    }
    #[tokio::test]
    #[ignore]
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
