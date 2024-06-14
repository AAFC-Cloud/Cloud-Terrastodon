use anyhow::bail;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use async_recursion::async_recursion;
use chrono::DateTime;
use chrono::Local;
use pathing_types::Existy;
use pathing_types::IgnoreDir;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::windows::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use std::time::Duration;
use tempfile::Builder;
use tempfile::TempPath;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
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
    async fn apply_args_and_envs(
        &self,
        this: &CommandBuilder,
        cmd: &mut Command,
    ) -> Result<Vec<TempPath>> {
        let mut rtn = Vec::new();
        let mut args = this.args.clone();
        // Write azure args to files
        match (self, this.file_args.is_empty()) {
            (CommandKind::AzureCLI, false) => {
                // todo: add tests
                for (i, arg) in this.file_args.iter() {
                    debug!("Writing arg {}", arg.path.to_string_lossy());
                    let mut patch_arg = async |i: usize, file_path: &PathBuf| -> Result<()> {
                        // Get the arg from the array
                        // We are converting @myfile.txt to @/path/to/myfile.txt
                        let arg_to_update =
                            args.get_mut(i).expect("azure arg must match an argument");

                        // Check assumption - it should already begin with an @
                        let check = arg_to_update.to_string_lossy();
                        let first_char = check.chars().next().unwrap();
                        if first_char != '@' {
                            bail!(
                                "First character in file arg for {:?} must be '@', got {}",
                                this.kind,
                                check
                            )
                        }

                        // Write the file
                        let mut file = OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(&file_path)
                            .await
                            .context(format!("Opening azure arg file {}", file_path.display()))?;
                        file.write_all(arg.content.as_bytes())
                            .await
                            .context(format!("Writing azure arg file {}", file_path.display()))?;

                        // Update the value
                        arg_to_update.clear();
                        arg_to_update.push("@");
                        arg_to_update.push(file_path.canonicalize().context(
                            "azure arg file must be written before absolute path can be determined",
                        )?);
                        Ok(())
                    };
                    let mut file = match &this.cache_behaviour {
                        CacheBehaviour::Some {
                            path: cache_dir, ..
                        } => {
                            // Cache dir has been provided
                            // we won't use temp files
                            cache_dir.ensure_dir_exists().await?;
                            let file_path = cache_dir.join(&arg.path);
                            patch_arg(*i, &file_path).await?;
                            tokio::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(&file_path)
                                .await
                                .context(format!(
                                    "opening azure arg file {}",
                                    arg.path.to_string_lossy()
                                ))?
                        }
                        CacheBehaviour::None => {
                            // No cache dir
                            // We will write azure args to temp files
                            let temp_dir = IgnoreDir::Temp.as_path_buf();
                            temp_dir.ensure_dir_exists().await?;
                            let path = tempfile::Builder::new()
                                .suffix(&arg.path)
                                .tempfile_in(temp_dir)
                                .context(format!(
                                    "creating temp file {}",
                                    arg.path.to_string_lossy()
                                ))?
                                .into_temp_path();
                            patch_arg(*i, &path.to_path_buf()).await?;
                            let file = tokio::fs::OpenOptions::new()
                                .write(true)
                                .open(&path)
                                .await
                                .context(format!(
                                    "opening azure arg file {}",
                                    arg.path.to_string_lossy()
                                ))?;
                            rtn.push(path); // add to rtn list so its not dropped+cleaned immediately
                            file
                        }
                    };
                    file.write_all(arg.content.as_bytes())
                        .await
                        .context(format!(
                            "writing azure arg file {}",
                            arg.path.to_string_lossy()
                        ))?;
                }
            }
            (_, false) => {
                bail!("Only {:?} can use Azure args", CommandKind::AzureCLI);
            }
            (_, true) => {}
        }
        // Apply args and envs to tokio Command
        match self {
            CommandKind::Echo => {
                if !this.env.is_empty() {
                    bail!("envs cannot be specified for {self:?}");
                }
                cmd.env("value", this.args.join(&OsString::from(" ")));
                cmd.args([
                    "-NoProfile",
                    "-Command",
                    "Write-Host -ForegroundColor Green $env:value",
                ]);
            }
            CommandKind::Pause => {
                if !this.env.is_empty() {
                    bail!("envs cannot be specified for {self:?}");
                }
                cmd.args(["-NoProfile", "-Command", "Pause"]);
            }
            _ => {
                cmd.args(args);
                cmd.envs(&this.env);
            }
        }
        Ok(rtn)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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
    Some {
        path: PathBuf,
        valid_for: Duration,
    },
}

#[derive(Debug, Clone)]
struct FileArg {
    path: PathBuf,
    content: String,
}

#[derive(Debug, Default, Clone)]
pub struct CommandBuilder {
    kind: CommandKind,
    args: Vec<OsString>,
    file_args: HashMap<usize, FileArg>,
    env: HashMap<String, String>,
    run_dir: Option<PathBuf>,
    retry_behaviour: RetryBehaviour,
    output_behaviour: OutputBehaviour,
    cache_behaviour: CacheBehaviour,
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
        self.cache_behaviour = CacheBehaviour::Some {
            path: IgnoreDir::Commands.join(cache),
            valid_for: Duration::from_days(1),
        };
        self
    }
    pub fn use_cache_behaviour(&mut self, mut behaviour: CacheBehaviour) -> &mut Self {
        if let CacheBehaviour::Some { ref mut path, .. } = behaviour {
            *path = IgnoreDir::Commands.join(&path);
        }
        self.cache_behaviour = behaviour;
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
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn file_arg<S: AsRef<Path>>(&mut self, path: S, content: String) -> &mut Self {
        // setup
        let path_buf = path.as_ref().to_path_buf();
        let path = path_buf.clone();
        let mut arg = path_buf.into_os_string();

        // transform based on kind
        if self.kind == CommandKind::AzureCLI {
            let mut new_arg = OsString::new();
            new_arg.push("@");
            new_arg.push(arg);
            arg = new_arg;
        }

        // push
        self.file_args
            .insert(self.args.len(), FileArg { path, content });
        self.args.push(arg);
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
        format!(
            "{} {}",
            self.kind.program(),
            self.args.join(&OsString::from(" ")).to_string_lossy()
        )
    }

    pub async fn get_cached_output(&self) -> Result<Option<CommandOutput>> {
        // Short circuit if not using cache or if cache entry not present
        let CacheBehaviour::Some {
            path: cache_dir,
            valid_for,
        } = &self.cache_behaviour
        else {
            return Ok(None);
        };
        if !cache_dir.exists() {
            return Ok(None);
        }

        let load_path = async |path: &PathBuf| -> Result<String> {
            let path = cache_dir.join(path);
            let mut file = OpenOptions::new()
                .read(true)
                .open(&path)
                .await
                .context(format!("opening cache file {}", path.display()))?;

            // Read the file
            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents)
                .await
                .context(format!("reading cache file {}", path.display()))?;
            Ok(file_contents)
        };
        let load_file =
            async |path: &str| -> Result<String> { load_path(&PathBuf::from(path)).await };

        // Validate cache matches expectations
        let expect_files = [
            // Command summary must match
            (&PathBuf::from("context.txt"), &self.summarize()),
        ];
        let mut expect_files = Vec::from_iter(expect_files);
        for arg in self.file_args.values() {
            // Azure argument files must match
            expect_files.push((&arg.path, &arg.content));
        }
        for (path, expected_contents) in expect_files {
            let file_contents = load_path(path).await?;

            // If an expectation is present, validate it
            if file_contents != *expected_contents {
                bail!(
                    "Cache context mismatch for {}, found: {}, expected: {}",
                    path.display(),
                    file_contents,
                    expected_contents
                );
            }
        }

        let timestamp = load_file("timestamp.txt").await?;
        let timestamp = DateTime::parse_from_rfc2822(&timestamp)?;
        let now = Local::now();
        if now > timestamp + *valid_for {
            bail!(
                "Cache entry has expired (was from {}, was valid for {} hours)",
                timestamp,
                valid_for.as_secs() / 60 / 60
            );
        }

        let status: u32 = load_file("status.txt").await?.parse()?;
        let stdout = load_file("stdout.json").await?;
        let stderr = load_file("stderr.json").await?;

        // Return!
        Ok(Some(CommandOutput {
            status,
            stdout,
            stderr,
        }))
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
            ("timestamp.txt", &Local::now().to_rfc2822()),
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
    pub async fn run_raw(&self) -> Result<CommandOutput> {
        // Check cache
        match self.get_cached_output().await {
            Ok(None) => {}
            Ok(Some(output)) => return Ok(output),
            Err(e) => {
                warn!("Cache load failed: {:?}", e);
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

        // Apply arguments, saving temp files to a variable to be cleaned up when dropped later
        let _temp_files = self
            .kind
            .apply_args_and_envs(self, &mut command)
            .await
            .context("applying args and envs")?;

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
            && let CacheBehaviour::Some {
                path: cache_dir, ..
            } = &self.cache_behaviour
        {
            if let Err(e) = self.write_output(&output, cache_dir).await {
                error!("Encountered problem saving cache: {:?}", e);
            }
        }

        // Return success
        Ok(output)
    }

    // #[async_recursion]
    pub async fn run<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Get stdout
        let output = self.run_raw().await?;

        // Parse
        match serde_json::from_str(&output.stdout) {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = match &self.cache_behaviour {
                    CacheBehaviour::None => IgnoreDir::Commands.join("failed"),
                    CacheBehaviour::Some {
                        path: cache_dir, ..
                    } => cache_dir.join("failed"),
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
    use tokio::time::sleep;

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
    async fn it_works_azure() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["graph", "query", "--graph-query"])
            .file_arg(
                "query.kql",
                r#"
resourcecontainers
| summarize count()
"#
                .to_string(),
            )
            .run_raw()
            .await?;
        println!("{}", result);
        Ok(())
    }

    #[tokio::test]
    async fn it_works_azure_cached() -> Result<()> {
        let result = CommandBuilder::new(CommandKind::AzureCLI)
            .args(["graph", "query", "--graph-query"])
            .file_arg(
                "query.kql",
                r#"
resourcecontainers
| summarize count()
"#
                .to_string(),
            )
            .use_cache_dir(PathBuf::from_iter([
                "az graph query",
                "--graph-query count-resource-containers",
            ]))
            .run_raw()
            .await?;
        println!("{}", result);
        Ok(())
    }

    #[tokio::test]
    async fn it_works_azure_cached_valid_for() -> Result<()> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["graph", "query", "--graph-query"]);
        cmd.file_arg(
            "query.kql",
            r#"
Resources	
| limit 1
| project CurrentTime = now()
"#
            .to_string(),
        );
        cmd.use_cache_behaviour(CacheBehaviour::Some {
            path: PathBuf::from_iter(["az graph query", "--graph-query count-resource-containers"]),
            valid_for: Duration::from_secs(3),
        });

        let result1 = cmd.run_raw().await?;
        let result2 = cmd.run_raw().await?;
        sleep(Duration::from_secs(4)).await;
        let result3 = cmd.run_raw().await?;
        println!("result1: {result1:?}\nresult2: {result2:?}\nresult3: {result3:?}");
        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
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
