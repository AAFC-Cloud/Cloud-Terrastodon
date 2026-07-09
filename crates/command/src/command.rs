use crate::ArtifactMetadata;
use crate::CacheKey;
use crate::CommandArgument;
use crate::CommandKind;
use crate::CommandOutput;
use crate::PathMapper;
use async_recursion::async_recursion;
pub use bstr;
use bstr::BString;
use bstr::ByteSlice;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::future::Future;
use std::panic::Location;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::task::spawn_blocking;
use tokio::time::timeout;
use tracing::Instrument;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::warn;

pub trait FromCommandOutput: Facet<'static> + Send + 'static {}
impl<T> FromCommandOutput for T where T: Facet<'static> + Send + 'static {}

enum CommandOutputDecodeError {
    Deserialize(eyre::Report),
    Map(eyre::Report),
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
pub struct CommandBuilder {
    pub(crate) kind: CommandKind,
    pub(crate) args: Vec<CommandArgument>,
    pub(crate) adjacent_files: HashMap<PathBuf, BString>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) run_dir: Option<PathBuf>,
    pub(crate) retry_behaviour: RetryBehaviour,
    pub(crate) output_behaviour: OutputBehaviour,
    pub(crate) cache_key: Option<CacheKey>,
    pub(crate) should_announce: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) stdin_content: Option<String>,
}

static LOGIN_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::const_new();

#[derive(facet::Facet)]
struct ProcessFingerprint {
    program: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    run_dir: Option<String>,
    stdin_content: Option<String>,
    debug_inputs: BTreeMap<String, BString>,
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
    pub async fn bust_cache(&self) -> Result<()> {
        let Some(cache_key) = &self.cache_key else {
            bail!("no cache entry present");
        };
        let cache_dir = cache_key.path_on_disk();
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
    pub fn cache(&mut self, key: CacheKey) -> &mut Self {
        self.cache_key = Some(key);
        self
    }

    #[track_caller]
    pub fn use_cache(&mut self, key: Option<CacheKey>) -> &mut Self {
        self.cache_key = key;
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
        self.args
            .push(CommandArgument::Literal(arg.as_ref().to_owned()));
        self
    }

    pub fn adjacent_file<P: Into<PathBuf>, C: Into<BString>>(
        &mut self,
        path: P,
        content: C,
    ) -> &mut Self {
        self.adjacent_files.insert(path.into(), content.into());
        self
    }

    /// Write a file to disk and pass it to the command using a mapped canonical path.
    pub fn file_arg<S: AsRef<Path>>(
        &mut self,
        path: S,
        mapper: impl PathMapper,
        content: String,
    ) -> &mut Self {
        let path = path.as_ref().to_path_buf();
        self.args.push(CommandArgument::DeferredAdjacentFilePath {
            key: path.clone(),
            mapper: Arc::new(mapper),
        });
        self.adjacent_files.insert(path, content.into());
        self
    }

    /// Write a file to disk and pass it to the command using the `@path` syntax.
    pub fn azure_file_arg<S: AsRef<Path>>(&mut self, path: S, content: String) -> &mut Self {
        self.file_arg(
            path,
            crate::PrefixPathMapper { prefix: "@".into() },
            content,
        );
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
            let has_debug = args
                .iter()
                .any(|a| matches!(a, CommandArgument::Literal(lit) if lit == "--debug"));
            if !has_debug {
                args.push(CommandArgument::Literal("--debug".into()));
            }
        }
        let args = args.into_iter().map(OsString::from).collect::<Vec<_>>();
        format!(
            "{} {}",
            self.kind.program().await,
            args.join(&OsString::from(" ")).to_string_lossy()
        )
    }

    fn cache_debug_inputs(&self) -> BTreeMap<PathBuf, BString> {
        self.adjacent_files
            .iter()
            .map(|(path, contents)| (path.clone(), contents.clone()))
            .collect()
    }

    async fn cache_metadata(&self, fingerprint: &str) -> ArtifactMetadata {
        ArtifactMetadata::new(
            fingerprint,
            "process",
            std::any::type_name::<CommandOutput>(),
        )
    }

    async fn cache_fingerprint(&self, debug_inputs: &BTreeMap<PathBuf, BString>) -> Result<String> {
        let mut args = self.args.clone();
        if self.kind == CommandKind::AzureCLI {
            let has_debug = args
                .iter()
                .any(|a| matches!(a, CommandArgument::Literal(lit) if lit == "--debug"));
            if !has_debug {
                args.push(CommandArgument::Literal("--debug".into()));
            }
        }
        let fingerprint = ProcessFingerprint {
            program: self.kind.program().await,
            args: args
                .into_iter()
                .map(OsString::from)
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
            env: self
                .env
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            run_dir: self
                .run_dir
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            stdin_content: self.stdin_content.clone(),
            debug_inputs: debug_inputs
                .iter()
                .map(|(path, contents)| (path.to_string_lossy().into_owned(), contents.clone()))
                .collect(),
        };
        let bytes = crate::json::to_vec(&fingerprint)?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    pub async fn get_cached_output(&self) -> Result<Option<CommandOutput>> {
        let Some(cache_key) = &self.cache_key else {
            debug!("Cache behaviour is None, not using cache");
            return Ok(None);
        };
        let context = self.summarize().await;
        let debug_inputs = self.cache_debug_inputs();
        let fingerprint = self.cache_fingerprint(&debug_inputs).await?;
        crate::artifact_cache::get_cached_output(cache_key, &context, &debug_inputs, &fingerprint)
            .await
    }

    pub async fn write_output(&self, output: &CommandOutput, parent_dir: &Path) -> Result<()> {
        debug!(path = %parent_dir.display(), "Writing command results");
        let context = self.summarize().await;
        let debug_inputs = self.cache_debug_inputs();
        let fingerprint = self.cache_fingerprint(&debug_inputs).await?;
        let metadata = self.cache_metadata(&fingerprint).await;
        crate::artifact_cache::write_output(parent_dir, &context, &debug_inputs, output, &metadata)
            .await
    }

    /// Sends content to stdin of the command.
    pub fn send_stdin(&mut self, content: impl Into<String>) -> &mut Self {
        self.stdin_content = Some(content.into());
        self
    }

    async fn run_raw_inner(&self, caller: &'static Location<'static>) -> Result<CommandOutput> {
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
            info!("Executing command");
        } else {
            debug!("Executing command");
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
                        "Error: Too Many Requests"
                    ]
                    .into_iter()
                    .any(|x| output.stderr.contains_str(x)) =>
                {
                    let mut sleep_duration = Duration::from_secs(30);

                    // Scan output to determine a tighter sleep duration
                    //  'x-ms-user-quota-resets-after': '00:00:04'
                    let needle = "'x-ms-user-quota-resets-after': '";
                    if let Some(pos) = output.stderr.find(needle) {
                        let start = pos + needle.len();
                        if let Some(end) = output.stderr[start..].find("'") {
                            let reset_after_str = String::from_utf8_lossy(&output.stderr[start..start + end]);
                            // parse duration in format "hh:mm:ss"
                            let parts = reset_after_str.split(':').map(|x| x.parse::<u64>()).collect::<Result<Vec<_>, _>>()?;
                            sleep_duration = match parts.as_slice() {
                                [hh, mm, ss] => {
                                    Duration::from_secs(hh * 3600 + mm * 60 + ss) + Duration::from_secs(5)
                                }
                                _ => sleep_duration,
                            };
                        }
                    }


                    // Retry the failed command, no further retries
                    warn!("Rate limit detected ⏳ Retrying command after {sleep_duration:?} wait...");
                    tokio::time::sleep(sleep_duration).await;

                    info!("It's been {sleep_duration:?}, retrying command `{}`", self.summarize().await);
                    let mut retry = self.clone();
                    retry.use_retry_behaviour(RetryBehaviour::Fail);
                    let output = retry.run_raw_from(caller).await;

                    // Return the result
                    return output;
                },
                RetryBehaviour::Retry
                    if [
                        "AADSTS70043",
                        "No subscription found. Run 'az account set' to select a subscription.",
                        "Please run 'az login' to setup account.",
                        "ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467",
                        // r#"ERROR: cli.azure.cli.core.azclierror: Forbidden({"error":{"code":"UnauthorizedAccessException"#, // this one is because the wrong tenant was used - `az rest` ignores `--subscription` and only cares about the active account, we will need to migrate to reqwest+`az account get-access-token` or some `az account set` shenanigans
                        // "Continuous access evaluation resulted in challenge with result: InteractionRequired" // may require `az logout` first? https://github.com/Azure/azure-cli/issues/26504
                    ]
                    .into_iter()
                    .any(|x| output.stderr.contains_str(x)) =>
                {
                    if std::env::var("CLOUD_TERRASTODON_REAUTH").unwrap_or_default().to_uppercase() == "DENY" {
                        bail!("Command failed due to bad auth, and automatic reauthentication is disabled by the CLOUD_TERRASTODON_REAUTH environment variable. Please refresh your credentials and try again.")
                    }
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
                            // This could maybe be moved to cloud_terrastodon_credentials
                            // We could also try and extract `--tenant {}` from args
                            // or just make the tenant id be explicitly set for CommandBuilder
                            let tenant_id = CommandBuilder::new(CommandKind::AzureCLI)
                                .args([
                                    "account",
                                    "list",
                                    "--query",
                                    "[?isDefault].tenantId",
                                    "--output",
                                    "tsv",
                                ])
                                .run_raw_from(caller)
                                .await?
                                .stdout;
                            let tenant_id = tenant_id.trim();
                            if tenant_id.is_empty() {
                                warn!(
                                    "Failed to find tenant ID from default account, the login command without tenant ID has been flaky for me .-. trying anyways"
                                );
                                CommandBuilder::new(CommandKind::AzureCLI)
                                    .arg("login")
                                    .run_raw_from(caller)
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
                                    .run_raw_from(caller)
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
                    let output = retry.run_raw_from(caller).await;

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
            && let Some(cache_key) = &self.cache_key
            && let Err(e) = self.write_output(&output, &cache_key.path_on_disk()).await
        {
            crate::artifact_cache::note_cache_write_failure(&e);
        }

        // Return success
        Ok(output)
    }

    #[track_caller]
    pub fn run_raw(&self) -> impl Future<Output = Result<CommandOutput>> + Send + '_ {
        self.run_raw_from(Location::caller())
    }

    #[track_caller]
    pub fn run_raw_polite(
        &self,
        uncached_delay: Duration,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + '_ {
        self.run_raw_polite_from(uncached_delay, Location::caller())
    }

    async fn run_raw_polite_from(
        &self,
        uncached_delay: Duration,
        caller: &'static Location<'static>,
    ) -> Result<CommandOutput> {
        let cached_output = match self.get_cached_output().await {
            Ok(Some(output)) => return Ok(output),
            cached_output => cached_output,
        };

        if !uncached_delay.is_zero() {
            debug!(
                delay_ms = uncached_delay.as_millis(),
                "Sleeping before uncached command execution"
            );
            tokio::time::sleep(uncached_delay).await;
        }
        self.run_raw_from_with_cached_output(caller, Some(cached_output))
            .await
    }

    #[async_recursion]
    async fn run_raw_from(&self, caller: &'static Location<'static>) -> Result<CommandOutput> {
        self.run_raw_from_with_cached_output(caller, None).await
    }

    async fn run_raw_from_with_cached_output(
        &self,
        caller: &'static Location<'static>,
        cached_output: Option<Result<Option<CommandOutput>>>,
    ) -> Result<CommandOutput> {
        let summary = self.summarize().await;
        let span =
            info_span!("command_run_raw", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current();

        async {
            // Check cache
            let cached_output = match cached_output {
                Some(cached_output) => cached_output,
                None => self.get_cached_output().instrument(span.clone()).await,
            };
            match cached_output {
                Ok(None) => {}
                Ok(Some(output)) => {
                    return Ok(output);
                }
                Err(error) => {
                    debug!(?self.cache_key, %error, "Cache load failed");
                }
            }

            let start = Instant::now();
            let rtn = self.run_raw_inner(caller).instrument(span.clone()).await;
            let elapsed = Instant::now().duration_since(start);
            debug!(
                elapsed_ms = elapsed.as_millis(),
                "Command executed in {}",
                humantime::format_duration(elapsed),
            );
            rtn
        }
        .instrument(span.clone())
        .await
        .wrap_err(format!(
            "Command::run_raw failed, called from {}",
            RelativeLocation::from(caller)
        ))
        .wrap_err(format!("Invoking command failed: {summary}",))
    }

    #[track_caller]
    pub fn run<T: FromCommandOutput>(&self) -> impl Future<Output = Result<T>> + Send + '_ {
        self.run_from(Location::caller())
    }

    /// Politely wait between uncached command executions, but still return cached results if available.
    #[track_caller]
    pub fn run_polite<T: FromCommandOutput>(
        &self,
        uncached_delay: Duration,
    ) -> impl Future<Output = Result<T>> + Send + '_ {
        self.run_polite_from(uncached_delay, Location::caller())
    }

    /// Politely wait between uncached command executions, but still return cached results if available.
    async fn run_polite_from<T: FromCommandOutput>(
        &self,
        uncached_delay: Duration,
        caller: &'static Location<'static>,
    ) -> Result<T> {
        let summary = self.summarize().await;
        let span = info_span!("command_run_polite", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current();

        let output = self
            .run_raw_polite_from(uncached_delay, caller)
            .instrument(span.clone())
            .await
            .wrap_err(format!(
                "Command::run_polite failed, called from {}",
                RelativeLocation::from(caller)
            ))?;
        let output = Arc::new(output);

        let parse_result = {
            let output = Arc::clone(&output);
            let span = span.clone();
            spawn_blocking(move || {
                let _guard = span.enter();
                let span2 = info_span!("command_parse_output").or_current();
                let _guard2 = span2.enter();
                let start = Instant::now();

                let stdout = output.stdout.to_str_lossy();
                let slice = stdout.as_bytes();
                let parse_result = crate::json::from_slice(slice);

                let elapsed = Instant::now().duration_since(start);
                debug!(
                    parse_ms = elapsed.as_millis(),
                    "Parsed command output in {}",
                    humantime::format_duration(elapsed),
                );
                parse_result
            })
            .await?
        };

        match parse_result {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = self
                    .write_failure(&output)
                    .instrument(span.or_current())
                    .await?;
                Err(e.wrap_err(format!(
                    "Deserialization failed!\n - Command: `{summary}`\n - Called by: \"{}\"\n - Dumped to: {dir:?}\n - Type: {}",
                    RelativeLocation::from(caller),
                    std::any::type_name::<T>()
                )))
            }
        }
    }

    async fn run_from<T: FromCommandOutput>(
        &self,
        caller: &'static Location<'static>,
    ) -> Result<T> {
        let summary = self.summarize().await;
        let span = info_span!("command_run", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current();

        let output = self
            .run_raw_from(caller)
            .instrument(span.clone())
            .await
            .wrap_err(format!(
                "Command::run failed, called from {}",
                RelativeLocation::from(caller)
            ))?;
        let output = Arc::new(output);

        let parse_result = {
            let output = Arc::clone(&output);
            let span = span.clone();
            spawn_blocking(move || {
                let _guard = span.enter();
                let span2 = info_span!("command_parse_output").or_current();
                let _guard2 = span2.enter();
                let start = Instant::now();

                let stdout = output.stdout.to_str_lossy();
                let slice = stdout.as_bytes();
                let parse_result = crate::json::from_slice(slice);

                let elapsed = Instant::now().duration_since(start);
                debug!(
                    parse_ms = elapsed.as_millis(),
                    "Parsed command output in {}",
                    humantime::format_duration(elapsed),
                );
                parse_result
            })
            .await?
        };

        match parse_result {
            Ok(results) => Ok(results),
            Err(e) => {
                let dir = self
                    .write_failure(&output)
                    .instrument(span.or_current())
                    .await?;
                Err(e.wrap_err(format!(
                        "Deserialization failed!\n - Command: `{summary}`\n - Called by: \"{}\"\n - Dumped to: {dir:?}\n - Type: {}",
                        RelativeLocation::from(caller),
                        std::any::type_name::<T>()
                    )))
            }
        }
    }

    #[track_caller]
    pub fn run_with_validator<T, F>(
        &self,
        validator: F,
    ) -> impl Future<Output = Result<T>> + Send + '_
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T> + Send + 'static,
    {
        self.run_with_validator_from(validator, Location::caller())
    }

    /// Parse command output as `Raw`, then fallibly map it to a different output shape.
    /// Both parse and map failures are dumped through the command failure artifact path.
    #[track_caller]
    pub fn run_with_mapper<Raw, Output, F>(
        &self,
        mapper: F,
    ) -> impl Future<Output = Result<Output>> + Send + '_
    where
        Raw: FromCommandOutput,
        Output: Send + 'static,
        F: FnOnce(Raw) -> Result<Output> + Send + 'static,
    {
        self.run_with_mapper_from(mapper, Location::caller())
    }

    async fn run_with_mapper_from<Raw, Output, F>(
        &self,
        mapper: F,
        caller: &'static Location<'static>,
    ) -> Result<Output>
    where
        Raw: FromCommandOutput,
        Output: Send + 'static,
        F: FnOnce(Raw) -> Result<Output> + Send + 'static,
    {
        let summary = self.summarize().await;
        let span = info_span!("command_run_with_mapper", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current();

        let output = self
            .run_raw_from(caller)
            .instrument(span.clone())
            .await
            .wrap_err(format!(
                "Command::run_with_mapper failed, called from {}",
                RelativeLocation::from(caller)
            ))?;
        let output = Arc::new(output);

        let decode_result = {
            let output = Arc::clone(&output);
            let span = span.clone();
            spawn_blocking(move || {
                let _guard = span.enter();
                let span2 = info_span!("command_parse_and_map_output").or_current();
                let _guard2 = span2.enter();
                let start = Instant::now();

                let stdout = output.stdout.to_str_lossy();
                let slice = stdout.as_bytes();
                let decode_result = crate::json::from_slice::<Raw>(slice)
                    .map_err(CommandOutputDecodeError::Deserialize)
                    .and_then(|raw| mapper(raw).map_err(CommandOutputDecodeError::Map));

                let elapsed = Instant::now().duration_since(start);
                debug!(
                    parse_ms = elapsed.as_millis(),
                    "Parsed and mapped command output in {}",
                    humantime::format_duration(elapsed),
                );
                decode_result
            })
            .await?
        };

        match decode_result {
            Ok(results) => Ok(results),
            Err(CommandOutputDecodeError::Deserialize(e)) => {
                let dir = self
                    .write_failure(&output)
                    .instrument(span.or_current())
                    .await?;
                Err(e.wrap_err(format!(
                    "Deserialization failed!\n - Command: `{summary}`\n - Called by: \"{}\"\n - Dumped to: {dir:?}\n - Type: {}",
                    RelativeLocation::from(caller),
                    std::any::type_name::<Raw>()
                )))
            }
            Err(CommandOutputDecodeError::Map(e)) => {
                let dir = self
                    .write_failure(&output)
                    .instrument(span.or_current())
                    .await?;
                Err(e.wrap_err(format!(
                    "Output mapping failed!\n - Command: `{summary}`\n - Called by: \"{}\"\n - Dumped to: {dir:?}\n - Parsed Type: {}\n - Output Type: {}",
                    RelativeLocation::from(caller),
                    std::any::type_name::<Raw>(),
                    std::any::type_name::<Output>()
                )))
            }
        }
    }

    pub async fn run_with_validator_from<T, F>(
        &self,
        validator: F,
        caller: &'static Location<'static>,
    ) -> Result<T>
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T>,
    {
        // Get stdout
        let output = self.run_raw().await?;

        // Parse
        match crate::json::from_slice(&output.stdout) {
            Ok(results) => match validator(results) {
                Ok(results) => Ok(results),
                Err(e) => {
                    let dir = self.write_failure(&output).await?;
                    Err(e).context(format!("Encountered validation error after successful invocation of `{}`\ncalled by \"{}\"\ndumped to {:?}",
                                self.summarize().await,
                                RelativeLocation::from(caller),
                                dir
                            ))
                }
            },
            Err(e) => {
                let dir = self.write_failure(&output).await?;
                Err(e.wrap_err(format!(
                    "deserializing `{}` failed\ncalled by \"{}\"\ndumped to {:?}",
                    self.summarize().await,
                    RelativeLocation::from(caller),
                    dir
                )))
            }
        }
    }

    pub async fn write_failure(&self, output: &CommandOutput) -> Result<PathBuf> {
        let context = self.summarize().await;
        let debug_inputs = self.cache_debug_inputs();
        let fingerprint = self.cache_fingerprint(&debug_inputs).await?;
        let metadata = self.cache_metadata(&fingerprint).await;
        crate::artifact_cache::write_failure(
            self.cache_key.as_ref(),
            &context,
            &debug_inputs,
            output,
            &metadata,
            None,
        )
        .await
    }
}
