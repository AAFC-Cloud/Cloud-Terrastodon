use crate::NoSpaces;
use async_recursion::async_recursion;
pub use bstr;
use bstr::BString;
use bstr::ByteSlice;
use chrono::DateTime;
use chrono::Local;
use cloud_terrastodon_config::CommandsConfig;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Error;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
#[cfg(not(windows))]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tempfile::Builder;
use tempfile::TempPath;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::time::timeout;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub enum CommandKind {
    #[default]
    AzureCLI,
    Terraform,
    VSCode,
    Echo,
    Pwsh,
    Git,
    Other(String),
}

pub const USE_TOFU_FLAG_KEY: &str = "CLOUD_TERRASTODON_USE_TOFU";

static CONFIG: OnceCell<CommandsConfig> = OnceCell::const_new();

async fn get_config(cache: &OnceCell<CommandsConfig>) -> &CommandsConfig {
    let config: &CommandsConfig = cache
        .get_or_init(|| async {
            let config: CommandsConfig = CommandsConfig::load().await.unwrap();
            config
        })
        .await;
    config
}

impl CommandKind {
    async fn program(&self) -> String {
        match self {
            CommandKind::AzureCLI => get_config(&CONFIG).await.azure_cli.to_owned(),
            CommandKind::Terraform => match env::var(USE_TOFU_FLAG_KEY) {
                Err(_) => get_config(&CONFIG).await.terraform.to_owned(),
                Ok(_) => get_config(&CONFIG).await.tofu.to_owned(),
            },
            CommandKind::VSCode => get_config(&CONFIG).await.vscode.to_owned(),
            CommandKind::Echo => "pwsh".to_string(),
            CommandKind::Pwsh => "pwsh".to_string(),
            CommandKind::Git => "git".to_string(),
            CommandKind::Other(x) => x.to_owned(),
        }
    }
    async fn apply_args_and_envs(
        &self,
        this: &CommandBuilder,
        cmd: &mut Command,
    ) -> Result<Vec<TempPath>> {
        let mut rtn = Vec::new();
        let mut args = this.args.clone();
        // Always add --debug for AzureCLI if not present
        if let CommandKind::AzureCLI = self {
            let has_debug = args.iter().any(|a| a == "--debug");
            if !has_debug {
                args.push("--debug".into());
            }
        }
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
                            let temp_dir = AppDir::Temp.as_path_buf();
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
            (CommandKind::Echo, true) => {
                let mut new_args: Vec<OsString> = Vec::with_capacity(3);
                new_args.push("-NoProfile".into());
                new_args.push("-Command".into());
                // new_args.push("echo".into());
                let mut guh = OsString::new();
                guh.push("[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new();'");
                let space: OsString = " ".into();
                guh.push(
                    args.join(&space)
                        .as_encoded_bytes()
                        .replace(b"'", b"''")
                        .to_os_str()?,
                );
                guh.push("'");
                new_args.push(guh);
                args = new_args;
            }
            (_, true) => {}
        }
        // Apply args and envs to tokio Command
        cmd.args(args);
        cmd.envs(&this.env);
        Ok(rtn)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CommandOutput {
    pub stdout: BString,
    pub stderr: BString,
    pub status: i32,
}
impl CommandOutput {
    pub fn success(&self) -> bool {
        #[cfg(windows)]
        return ExitStatus::from_raw(self.status as u32).success();
        #[cfg(not(windows))]
        return ExitStatus::from_raw(self.status).success();
    }
    #[track_caller]
    pub async fn try_interpret<T: DeserializeOwned>(
        &self,
        command: &CommandBuilder,
    ) -> eyre::Result<T> {
        match serde_json::from_slice(self.stdout.to_str_lossy().as_bytes()) {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = command.write_failure(self).await?;
                Err(eyre::Error::new(e)
                    .wrap_err(format!(
                        "Called from {}",
                        RelativeLocation::from(std::panic::Location::caller())
                    ))
                    .wrap_err(format!(
                        "deserializing `{}` failed, dumped to {:?}",
                        command.summarize().await,
                        dir
                    )))
            }
        }
    }
    /// Keeps only the first and last 500 lines of stdout and stderr
    pub fn shorten(&mut self) {
        fn keep_first_and_last_500_lines_with_warning(output: BString) -> BString {
            let lines: Vec<&[u8]> = output.lines().collect();
            let total = lines.len();

            if total <= 1000 {
                output
            } else {
                let mut trimmed = Vec::new();
                trimmed.extend_from_slice(&lines[..500]);

                // Add truncation warning
                let warning = b"...[output truncated: middle lines omitted]...";
                trimmed.push(warning);

                trimmed.extend_from_slice(&lines[total - 500..]);
                BString::from(trimmed.join(&b'\n'))
            }
        }
        let stdout = std::mem::take(&mut self.stdout);
        self.stdout = keep_first_and_last_500_lines_with_warning(stdout);

        let stderr = std::mem::take(&mut self.stderr);
        self.stderr = keep_first_and_last_500_lines_with_warning(stderr);
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
            // stdout: String::from_utf8_lossy_owned(value.stdout),
            // stderr: String::from_utf8_lossy_owned(value.stderr),
            stdout: BString::from(value.stdout),
            stderr: BString::from(value.stderr),
            status: match value.status.code().unwrap_or(1) {
                x if x < 0 => 1,
                x => x,
            },
        })
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub enum RetryBehaviour {
    Fail,
    #[default]
    Retry,
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
    stdin_content: Option<String>, // Added field for stdin content
}

static LOGIN_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::const_new();

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
        self.use_cache_behaviour(CacheBehaviour::Some {
            path: cache.as_ref().to_path_buf(),
            valid_for: Duration::from_days(1),
        })
    }
    pub async fn bust_cache(&self) -> Result<()> {
        let CacheBehaviour::Some {
            path: cache_dir, ..
        } = &self.cache_behaviour
        else {
            bail!("no cache entry present");
        };
        let busted_path = cache_dir.join("busted");
        let _file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&busted_path)
            .await
            .context(format!(
                "failed creating busted cache indicator at {}",
                busted_path.display(),
            ))?;
        Ok(())
    }

    #[track_caller]
    pub fn use_cache_behaviour(&mut self, mut behaviour: CacheBehaviour) -> &mut Self {
        if let CacheBehaviour::Some { ref mut path, .. } = behaviour {
            // add app dir prefix and remove spaces
            *path = AppDir::Commands.join(path.no_spaces());
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

    pub async fn summarize(&self) -> String {
        let mut args = self.args.clone();
        if self.kind == CommandKind::AzureCLI {
            let has_debug = args.iter().any(|a| a == "--debug");
            if !has_debug {
                args.push("--debug".into());
            }
        }
        format!(
            "{} {}",
            self.kind.program().await,
            args.join(&OsString::from(" ")).to_string_lossy()
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
        if valid_for.is_zero() {
            return Ok(None);
        }
        if !cache_dir.exists() {
            return Ok(None);
        }

        let load_from_pathbuf = async |path: &PathBuf| -> Result<BString> {
            let path = cache_dir.join(path);
            let mut file = OpenOptions::new()
                .read(true)
                .open(&path)
                .await
                .context(format!("opening cache file {}", path.display()))?;

            // Read the file
            let mut file_contents = Vec::new();
            file.read_to_end(&mut file_contents)
                .await
                .context(format!("reading cache file {}", path.display()))?;
            let file_contents = BString::from(file_contents);
            Ok(file_contents)
        };
        let load_from_path =
            async |path: &str| -> Result<BString> { load_from_pathbuf(&PathBuf::from(path)).await };

        // Check if cache is busted
        if !matches!(
            tokio::fs::try_exists(cache_dir.join("busted")).await,
            Ok(false)
        ) {
            debug!("Cache is busted");
            return Ok(None);
        }

        // Validate cache matches expectations
        let expect_files: [(&PathBuf, &String); 1] = [
            // Command summary must match
            (&PathBuf::from("context.txt"), &self.summarize().await),
        ];
        let mut expect_files = Vec::from_iter(expect_files);
        for arg in self.file_args.values() {
            // Azure argument files must match
            expect_files.push((&arg.path, &arg.content));
        }
        for (path, expected_contents) in expect_files {
            let file_contents = load_from_pathbuf(path).await?;

            // If an expectation is present, validate it
            if file_contents != *expected_contents {
                debug!(
                    "Cache context mismatch for {}, found: {}, expected: {}",
                    path.display(),
                    file_contents,
                    expected_contents
                );
                return Ok(None);
            }
        }

        let timestamp = load_from_path("timestamp.txt").await?;
        let timestamp = DateTime::parse_from_rfc2822(timestamp.to_str()?)?;
        let now = Local::now();
        if now > timestamp + *valid_for {
            debug!(
                "Cache entry has expired (was from {}, was valid for {})",
                timestamp,
                humantime::format_duration(*valid_for),
            );
            return Ok(None);
        }

        let status: i32 = load_from_path("status.txt").await?.to_str()?.parse()?;
        let stdout = load_from_path("stdout.json").await?;
        let stderr = load_from_path("stderr.json").await?;

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
        let summary = self.summarize().await;
        let status = output.status.to_string();
        let timestamp = &Local::now().to_rfc2822();
        let files = [
            ("context.txt", summary.as_bytes()),
            ("stdout.json", &output.stdout),
            ("stderr.json", &output.stderr),
            ("status.txt", status.as_bytes()),
            ("timestamp.txt", timestamp.as_bytes()),
        ];

        // Remove busted marker if present
        let busted_path = parent_dir.join("busted");
        if let Ok(true) = busted_path.try_exists() {
            tokio::fs::remove_file(&busted_path)
                .await
                .context("Removing busted cache marker")?;
        }

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

    /// Sends content to stdin of the command.
    pub fn send_stdin(&mut self, content: impl Into<String>) -> &mut Self {
        self.stdin_content = Some(content.into());
        self
    }

    #[async_recursion]
    #[track_caller]
    pub async fn run_raw_inner(&self) -> Result<CommandOutput> {
        // Check cache
        match self.get_cached_output().await {
            Ok(None) => {}
            Ok(Some(output)) => return Ok(output),
            Err(e) => {
                debug!("Cache load failed: {:?}", e);
            }
        }

        let mut command = Command::new(self.kind.program().await);
        match self.output_behaviour {
            OutputBehaviour::Capture => {
                command.stdin(Stdio::piped()); // Set stdin to piped for capture mode
                command.stdout(Stdio::piped());
                command.stderr(Stdio::piped());
            }
            OutputBehaviour::Display => {
                if self.stdin_content.is_some() {
                    command.stdin(Stdio::piped()); // Still need piped stdin if we want to send content
                }
            }
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
            info!(
                "Running `{}` in \"{}\"",
                self.summarize().await,
                self.run_dir
                    .as_ref()
                    .map(|x| x.display().to_string())
                    .unwrap_or(".".to_string())
            );
        } else {
            debug!(
                "Running `{}` in \"{}\"",
                self.summarize().await,
                self.run_dir
                    .as_ref()
                    .map(|x| x.display().to_string())
                    .unwrap_or(".".to_string())
            );
        }

        // Launch command
        command.kill_on_drop(true);
        let mut child = command.spawn().wrap_err("Failed to spawn command")?;

        // Send stdin content if provided
        if let Some(content) = &self.stdin_content
            && let Some(mut stdin) = child.stdin.take()
        {
            let content = content.to_owned();
            tokio::spawn(async move {
                // Spawn a task to avoid blocking the main thread while writing to stdin
                if let Err(e) = stdin.write_all(content.as_bytes()).await {
                    error!("Failed to write to stdin: {:?}", e);
                }
                // stdin.shutdown().await.ok(); // Not strictly needed, stdin will close when dropped
            });
        }

        // Wait for it to finish
        let timeout_duration = self.timeout.unwrap_or(Duration::MAX);
        let output: CommandOutput = match timeout(timeout_duration, child.wait_with_output()).await
        {
            Ok(result) => result
                .wrap_err("Acquiring result of command execution")?
                .try_into()
                .wrap_err("Converting output of command")?,
            Err(elapsed) => {
                bail!(
                    "Command timeout, {elapsed:?} ({})",
                    humantime::format_duration(timeout_duration)
                );
            }
        };

        // Return if errored
        if !output.success() {
            match self.retry_behaviour {
                 RetryBehaviour::Retry
                    if [
                        "ERROR: Too Many Requests",
                    ]
                    .into_iter()
                    .any(|x| output.stderr.contains_str(x)) =>
                {
                    // Retry the failed command, no further retries
                    warn!("Rate limit detected ⏳ Retrying command after 30 second wait...");
                    tokio::time::sleep(Duration::from_secs(30)).await;

                    info!("It's been 30 seconds, retrying command `{}`", self.summarize().await);
                    let mut retry = self.clone();
                    retry.use_retry_behaviour(RetryBehaviour::Fail);
                    let output = retry.run_raw().await;

                    // Return the result
                    return output;
                },
                RetryBehaviour::Retry
                    if [
                        "AADSTS70043",
                        "No subscription found. Run 'az account set' to select a subscription.",
                        "Please run 'az login' to setup account.",
                        "ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467",
                    ]
                    .into_iter()
                    .any(|x| output.stderr.contains_str(x)) =>
                {
                    let mutex = LOGIN_LOCK
                        .get_or_init(async || Arc::new(Mutex::new(())))
                        .await;
                    match mutex.try_lock() {
                        Ok(x) => {
                            debug!(
                                "Acquired login lock without waiting, there isn't a login in progress"
                            );
                            // Let the user know
                            warn!(
                                "Command failed due to bad auth. Refreshing credential, user action required in a moment..."
                            );

                            // Perform login command
                            // (avoid using azure crate to avoid a dependency)
                            let tenant_id = CommandBuilder::new(CommandKind::AzureCLI)
                                .args([
                                    "account",
                                    "list",
                                    "--query",
                                    "[?isDefault].tenantId",
                                    "--output",
                                    "tsv",
                                ])
                                .run_raw()
                                .await?
                                .stdout;
                            let tenant_id = tenant_id.trim();
                            if tenant_id.is_empty() {
                                warn!(
                                    "Failed to find tenant ID from default account, the login command without tenant ID has been flaky for me .-. trying anyways"
                                );
                                CommandBuilder::new(CommandKind::AzureCLI)
                                    .arg("login")
                                    .run_raw()
                                    .await?;
                            } else {
                                CommandBuilder::new(CommandKind::AzureCLI)
                                    .args([
                                        "login",
                                        "--tenant",
                                        tenant_id
                                            .to_str()
                                            .wrap_err("converting tenant id to str")?,
                                    ])
                                    .run_raw()
                                    .await?;
                            }

                            drop(x);
                        }
                        Err(_) => {
                            debug!("Login lock busy, waiting for the login to complete");
                            warn!(
                                "Command failed due to bad auth. Waiting for login in progress..."
                            );
                            _ = mutex.lock().await;
                        }
                    }

                    // Retry the failed command, no further retries
                    info!("Retrying command with refreshed credential...");
                    let mut retry = self.clone();
                    retry.use_retry_behaviour(RetryBehaviour::Fail);
                    let output = retry.run_raw().await;

                    // Return the result
                    return output;
                }
                _ => {
                    let dir = self.write_failure(&output).await?;
                    let mut error = Err(eyre::Error::from(output).wrap_err(format!(
                        "Command did not execute successfully, using retry behaviour {:?}, dumped to {dir:?}",
                        self.retry_behaviour
                    )));
                    if matches!(self.output_behaviour, OutputBehaviour::Display) {
                        error = error.wrap_err(format!(
                            "The output behaviour was set to {:?} instead of {:?} so the stdout and stderr are not available in the dump, try scrolling up in your terminal.", 
                            OutputBehaviour::Display,
                            OutputBehaviour::Capture,
                        ));
                    }

                    return error;
                }
            }
        }

        // Write happy results to the cache
        if output.success()
            && let CacheBehaviour::Some {
                path: cache_dir, ..
            } = &self.cache_behaviour
            && let Err(e) = self.write_output(&output, cache_dir).await
        {
            error!("Encountered problem saving cache: {:?}", e);
        }

        // Return success
        Ok(output)
    }

    #[track_caller]
    pub async fn run_raw(&self) -> Result<CommandOutput> {
        self.run_raw_inner()
            .await
            .wrap_err(format!(
                "Command::run_raw failed, called from {}",
                RelativeLocation::from(std::panic::Location::caller())
            ))
            .wrap_err(format!(
                "Invoking command failed: {}",
                self.summarize().await
            ))
    }

    #[track_caller]
    pub async fn run<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Get stdout
        let output = self.run_raw().await;
        let output = output.wrap_err(format!(
            "Command::run failed, called from {}",
            RelativeLocation::from(std::panic::Location::caller())
        ))?;

        // Parse
        match serde_json::from_slice(output.stdout.to_str_lossy().as_bytes()) {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = self.write_failure(&output).await?;
                Err(eyre::Error::new(e)
                    // .wrap_err(format!(
                    //     "Called from {}",
                    //     RelativeLocation::from(std::panic::Location::caller())
                    // ))
                    .wrap_err(format!(
                        "deserializing `{}` failed, dumped to {:?}",
                        self.summarize().await,
                        dir
                    )))
            }
        }
    }

    pub async fn run_with_validator<T, F>(&self, validator: F) -> Result<T>
    where
        T: DeserializeOwned,
        F: FnOnce(T) -> Result<T>,
    {
        // Get stdout
        let output = self.run_raw().await?;

        // Parse
        match serde_json::from_slice(&output.stdout) {
            Ok(results) => match validator(results) {
                Ok(results) => Ok(results),
                Err(e) => {
                    let dir = self.write_failure(&output).await?;
                    Err(e).context(format!("Encountered validation error after successful invocation of {}, dumped to {:?}",
                            self.summarize().await,
                            dir
                        ))
                }
            },
            Err(e) => {
                let dir = self.write_failure(&output).await?;
                Err(eyre::Error::new(e).wrap_err(format!(
                    "deserializing {} failed, dumped to {:?}",
                    self.summarize().await,
                    dir
                )))
            }
        }
    }

    pub async fn write_failure(&self, output: &CommandOutput) -> Result<PathBuf> {
        let (dir, write_file_args) = match &self.cache_behaviour {
            CacheBehaviour::None => (AppDir::Commands.join("failed"), true),
            CacheBehaviour::Some {
                path: cache_dir, ..
            } => (cache_dir.join("failed"), true),
        };
        dir.ensure_dir_exists().await?;
        let dir = Builder::new()
            .prefix(Local::now().format("%Y%m%d_%H%M%S_").to_string().as_str())
            .tempdir_in(dir)?
            .keep();
        self.write_output(output, &dir).await?;
        if write_file_args {
            for arg in self.file_args.values() {
                let path = dir.join(&arg.path);
                let mut file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&path)
                    .await
                    .context(format!("Opening arg file {}", arg.path.display()))?;
                file.write_all(arg.content.as_bytes())
                    .await
                    .context(format!("Writing arg file {}", arg.path.display()))?;
            }
        }
        Ok(dir)
    }
}

/// These tests require user interaction and change the state of the system!
///
/// You have been warned.
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tokio::time::Instant;
    use tokio::time::sleep_until;

    #[test]
    fn encoding() {
        let x = "é";
        let bytes = x.as_bytes().to_vec();
        let _y = String::from_utf8(bytes).unwrap();

        let x = "�";
        let bytes = x.as_bytes().to_vec();
        let _y = String::from_utf8(bytes).unwrap();

        let x = "\u{FFFD}";
        let bytes = x.as_bytes().to_vec();
        let _y = String::from_utf8(bytes).unwrap();
    }

    #[tokio::test]
    async fn encoding_2() {
        let mut cmd = CommandBuilder::new(CommandKind::Echo);
        // cmd.args(["ad","user","show","--id",""]);
        cmd.args(["aéa"]);
        let x = cmd.run_raw().await.unwrap();
        println!("Got {x:?}");
        println!("Expected: {:?}", "aéa".as_bytes());
        println!("Given:    {:?}", x.stdout.trim());
        // assert_eq!(x.stdout.trim(), "aéa".as_bytes());
        assert_eq!(x.stdout.trim().to_str().unwrap(), "aéa");
    }

    #[tokio::test]
    #[ignore]
    /// The Azure CLI uses system locale by default, which is latin-1 instead of UTF-8
    /// https://github.com/Azure/azure-cli/issues/22616
    async fn encoding_3() -> eyre::Result<()> {
        let user_id = cloud_terrastodon_user_input::prompt_line(
            "Enter the ID for the user who is experiencing encoding issues:",
        )
        .await?;
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "ad",
            "user",
            "show",
            "--id",
            user_id.as_ref(),
            "--query",
            "displayName",
        ]);
        let x = cmd.run_raw().await.unwrap().stdout;
        let bytes = x.as_ref() as &[u8];
        println!("Got {:?}", x);
        println!("Got {:?}", bytes);
        println!(
            "Got {:?}",
            bytes
                .iter()
                .map(|x| char::from_u32(*x as u32))
                .collect::<Vec<_>>()
        );
        let z = String::from_utf8(bytes.to_vec())?;
        println!("Decoded {z:?}");
        let y = x.to_str()?;
        println!("Decoded {y:?}");
        Ok(())
    }

    #[test]
    fn encoding_4() -> eyre::Result<()> {
        let byte = 233 as u8;
        println!("{byte} => {:?}", char::from_u32(byte as u32));
        let bytes = vec![byte, 101];
        println!("bytes: {bytes:?}");
        let str = String::from_utf8(bytes);
        println!("str: {str:?}");

        // 233 is a latin-1 not utf-8 valid.
        // Therefore, it should fail.
        assert!(str.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn echo_works() {
        let mut cmd = CommandBuilder::new(CommandKind::Echo);
        cmd.args(["ahoy", "world"]);
        let x = cmd.run_raw().await.unwrap();
        println!("Got {:?}", x.stdout);
        assert_eq!(x.stdout.trim(), "ahoy world".as_bytes());
    }

    #[tokio::test]
    async fn echo_works2() {
        let mut cmd = CommandBuilder::new(CommandKind::Echo);
        cmd.args(["a\"ho\"y", "w'or\nl'd"]);
        let x = cmd.run_raw().await.unwrap();
        println!("Got {:?}", x.stdout);
        assert_eq!(x.stdout.trim(), "a\"ho\"y w'or\nl'd".as_bytes());
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
                "az",
                "graph",
                "query",
                "count-resource-containers",
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
        let period = Duration::from_secs(5);
        cmd.use_cache_behaviour(CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "current-time"]),
            valid_for: period,
        });

        // we don't want anything between our `await` calls that could mess with the timing
        thread::Builder::new().spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    // fetch and cache
                    let t1 = Instant::now();
                    let result1 = cmd.run_raw().await?;

                    // ensure there is at least 1 second remaining before cache expiry
                    let t2 = Instant::now();
                    assert!(t2 + Duration::from_secs(1) < t1 + period);

                    // fetch using cache
                    let result2 = cmd.run_raw().await?;

                    // sleep until cache expired
                    sleep_until(t2 + period + Duration::from_secs(1)).await;
                    let t3 = Instant::now();
                    assert!(t3 > t2 + period);

                    // fetch new results without using cache
                    let result3 = cmd.run_raw().await?;

                    // ensure first two match and don't match third
                    println!("result1: {result1:?}\nresult2: {result2:?}\nresult3: {result3:?}");
                    assert_eq!(result1, result2);
                    assert_ne!(result1, result3);
                    Ok::<(), eyre::Error>(())
                })
                .unwrap();
        })?;
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
                    if stderr.contains_str("ERROR: There are no active accounts.") =>
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

    #[tokio::test]
    async fn send_stdin_echo() -> Result<()> {
        let mut cmd = CommandBuilder::new(CommandKind::Pwsh);
        cmd.args(["-NoProfile", "-Command", "-" /* Read from stdin */]); // For pwsh, "-" means read from stdin for command
        cmd.send_stdin("echo 'hello stdin'");
        let output = cmd.run_raw().await?;
        println!("Stdout: {:?}", output.stdout);
        assert_eq!(output.stdout.trim(), "hello stdin".as_bytes());
        Ok(())
    }

    #[tokio::test]
    async fn send_stdin_terraform_fmt() -> Result<()> {
        let content = r#"resource "time_static" "wait_1_second" {
depends_on = []
triggers_complete = null
}
"#;
        let mut cmd = CommandBuilder::new(CommandKind::Terraform);
        cmd.args(["fmt", "-"]);
        cmd.send_stdin(content);
        let output = cmd.run_raw().await?;
        println!("Stdout: {:?}", output.stdout);
        let expected = r#"resource "time_static" "wait_1_second" {
  depends_on        = []
  triggers_complete = null
}
"#;
        assert_eq!(output.stdout.trim(), expected.trim().as_bytes());
        Ok(())
    }
}
