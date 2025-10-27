use crate::CommandKind;
use crate::CommandOutput;
use crate::NoSpaces;
use async_recursion::async_recursion;
pub use bstr;
use bstr::BString;
use bstr::ByteSlice;
use chrono::DateTime;
use chrono::Local;
use chrono::TimeDelta;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tempfile::Builder;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::time::timeout;
use tracing::Instrument;
use tracing::debug;
use tracing::debug_span;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::warn;

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
pub struct FileArg {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Default, Clone)]
pub struct CommandBuilder {
    pub(crate) kind: CommandKind,
    pub(crate) args: Vec<OsString>,
    pub(crate) file_args: HashMap<usize, FileArg>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) run_dir: Option<PathBuf>,
    pub(crate) retry_behaviour: RetryBehaviour,
    pub(crate) output_behaviour: OutputBehaviour,
    pub(crate) cache_behaviour: CacheBehaviour,
    pub(crate) should_announce: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) stdin_content: Option<String>, // Added field for stdin content
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
        let start = Instant::now();
        // Short circuit if not using cache or if cache entry not present
        let CacheBehaviour::Some {
            path: cache_dir,
            valid_for,
        } = &self.cache_behaviour
        else {
            debug!("Cache behaviour is None, not using cache");
            return Ok(None);
        };
        if valid_for.is_zero() {
            debug!("Cache validity duration is zero, not using cache");
            return Ok(None);
        }
        if !cache_dir.exists() {
            debug!("Cache directory does not exist, not using cache");
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
        let load_from_path = async |path: &str| -> Result<BString> {
            let span = debug_span!("Reading command cache from disk");
            span.record("path", path);
            load_from_pathbuf(&PathBuf::from(path))
                .instrument(span.or_current())
                .await
        };

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
                    path=%path.display(),
                    found=%file_contents,
                    expected=%expected_contents,
                    "Not using cache due to expected content mismatch. Did Cloud Terrastodon change what command is being called?",
                );
                return Ok(None);
            }
        }

        let timestamp = load_from_path("timestamp.txt").await?;
        let timestamp = DateTime::parse_from_rfc2822(timestamp.to_str()?)?;
        let now = Local::now();
        let time_remaining = timestamp + *valid_for - now.fixed_offset();
        if time_remaining < TimeDelta::zero() {
            debug!(
                %timestamp,
                valid_for_seconds = valid_for.as_secs(),
                expired_for_seconds = time_remaining.abs().num_seconds(),
                "Cache entry has expired (was from {}, was valid for {}, expired {} ago)",
                timestamp,
                humantime::format_duration(*valid_for),
                humantime::format_duration(time_remaining.abs().to_std().unwrap()),
            );
            return Ok(None);
        }

        let status: i32 = load_from_path("status.txt").await?.to_str()?.parse()?;
        let stdout = load_from_path("stdout.json").await?;
        let stderr = load_from_path("stderr.json").await?;

        let elapsed = Instant::now().duration_since(start);
        debug!(
            %timestamp,
            valid_for_seconds = valid_for.as_secs(),
            remaining_seconds = time_remaining.num_seconds(),
            cache_load_ms = elapsed.as_millis(),
            "Loaded command output from cache in {}",
            humantime::format_duration(elapsed),
        );

        // Return!
        Ok(Some(CommandOutput {
            status,
            stdout,
            stderr,
        }))
    }

    pub async fn write_output(&self, output: &CommandOutput, parent_dir: &PathBuf) -> Result<()> {
        debug!(path = %parent_dir.display(), "Writing command results");

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
            Ok(Some(output)) => {
                debug!("Using cached command output");
                return Ok(output);
            }
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
            info!("Running command");
        } else {
            debug!("Running command");
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
                        // "Continuous access evaluation resulted in challenge with result: InteractionRequired" // may require `az logout` first? https://github.com/Azure/azure-cli/issues/26504
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
        let summary = self.summarize().await;
        let span = info_span!("run_command", summary, ?self.run_dir, ?self.cache_behaviour);
        self.run_raw_inner()
            .instrument(span.or_current())
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
