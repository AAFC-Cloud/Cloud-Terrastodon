use crate::ArtifactMetadata;
use crate::CacheKey;
use crate::CancellationScope;
use crate::CommandArgument;
use crate::CommandKind;
use crate::CommandOutput;
use crate::FromCommandOutput;
use crate::OutputBehaviour;
use crate::PathMapper;
use crate::RetryBehaviour;
use crate::command_output_decode_error::CommandOutputDecodeError;
use crate::command_process::CommandProcess;
use crate::process_fingerprint::ProcessFingerprint;
pub use bstr;
use bstr::BString;
use bstr::ByteSlice;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use eyre::bail;
use std::any::type_name;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::future::Future;
use std::future::IntoFuture;
use std::io::IsTerminal;
use std::panic::Location;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellableScopeFuture;
use teamy_cancellation::CancellationReportExt;
use tokio::fs::OpenOptions;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use tokio::time::sleep;
use tracing::Instrument;
use tracing::Span;
use tracing::debug;
use tracing::info;
use tracing::info_span;
use tracing::warn;

#[derive(Debug, Default, Clone)]
pub struct CommandBuilder {
    pub(crate) kind: CommandKind,
    pub(crate) args: Vec<CommandArgument>,
    pub(crate) adjacent_files: HashMap<PathBuf, BString>,
    pub(crate) env: HashMap<String, String>,
    pub(crate) removed_env: BTreeSet<String>,
    pub(crate) removed_env_prefixes: BTreeSet<String>,
    pub(crate) run_dir: Option<PathBuf>,
    pub(crate) retry_behaviour: RetryBehaviour,
    pub(crate) output_behaviour: OutputBehaviour,
    pub(crate) cache_key: Option<CacheKey>,
    pub(crate) should_announce: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) stdin_content: Option<String>,
    /// False by default: retain command execution, decoding, and artifacts.
    pub(crate) cancellation_safe: bool,
}

static LOGIN_LOCK: Mutex<()> = Mutex::const_new(());

impl CommandBuilder {
    pub fn new(kind: CommandKind) -> CommandBuilder {
        Self {
            kind,
            ..Self::default()
        }
    }

    /// Allows this command to be abandoned in response to cooperative cancellation.
    ///
    /// Commands otherwise defer cancellation through execution, output decoding,
    /// and artifact persistence. Opt in only when interrupting the command is
    /// acceptable. An enclosing protected scope still takes precedence; explicit
    /// future drops, task aborts, ordinary errors, and timeouts still apply.
    pub fn cancellation_safe(&mut self) -> &mut Self {
        self.cancellation_safe = true;
        self
    }

    #[track_caller]
    fn with_cancellation_policy<C, F, T>(
        &self,
        cancellation: &CancellationScope,
        execute: C,
    ) -> CancellableScopeFuture<C, F>
    where
        C: FnOnce(CancellationScope) -> F,
        F: IntoFuture<Output = Result<T>>,
    {
        if self.cancellation_safe {
            cancellation.cancellable_scope(execute)
        } else {
            cancellation.deferred(
                "finishing command execution, decoding its outcome, and retaining execution artifacts",
                execute,
            )
        }
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
        let key = key.as_ref();
        if cfg!(windows) {
            self.env
                .retain(|existing, _| !existing.eq_ignore_ascii_case(key));
        }
        self.env.insert(key.to_owned(), value.as_ref().to_owned());
        self
    }

    /// Remove inherited variables and earlier explicit values with this prefix.
    /// Matching follows platform environment-name casing, and a later `env`
    /// call can deliberately replace a removed value.
    pub fn env_remove_prefix(&mut self, prefix: impl AsRef<str>) -> &mut Self {
        let prefix = prefix.as_ref();
        self.env
            .retain(|key, _| !Self::environment_name_has_prefix(key, prefix));
        self.removed_env_prefixes.insert(prefix.to_owned());
        self
    }

    pub(crate) fn environment_name_has_prefix(name: &str, prefix: &str) -> bool {
        if cfg!(windows) {
            name.get(..prefix.len())
                .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
        } else {
            name.starts_with(prefix)
        }
    }

    /// Remove an inherited environment variable and any earlier explicit value.
    /// A subsequent `env` call can supply a replacement value.
    pub fn env_remove(&mut self, key: impl AsRef<str>) -> &mut Self {
        let key = key.as_ref();
        self.env.retain(|existing, _| {
            if cfg!(windows) {
                !existing.eq_ignore_ascii_case(key)
            } else {
                existing != key
            }
        });
        self.removed_env.insert(key.to_owned());
        self
    }

    pub fn should_announce(&mut self, value: bool) -> &mut Self {
        self.should_announce = value;
        self
    }

    pub async fn summarize(&self) -> Result<String> {
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
        Ok(format!(
            "{} {}",
            self.kind.program().await?,
            args.join(&OsString::from(" ")).to_string_lossy()
        ))
    }

    fn cache_debug_inputs(&self) -> BTreeMap<PathBuf, BString> {
        self.adjacent_files
            .iter()
            .map(|(path, contents)| (path.clone(), contents.clone()))
            .collect()
    }

    fn cache_metadata(fingerprint: &str) -> ArtifactMetadata {
        ArtifactMetadata::new(fingerprint, "process", type_name::<CommandOutput>())
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
            program: self.kind.program().await?,
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
            removed_env: self.removed_env.clone(),
            removed_env_prefixes: self.removed_env_prefixes.clone(),
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
        let context = self.summarize().await?;
        let debug_inputs = self.cache_debug_inputs();
        let fingerprint = self.cache_fingerprint(&debug_inputs).await?;
        crate::artifact_cache::get_cached_output(cache_key, &context, &debug_inputs, &fingerprint)
            .await
    }

    pub async fn write_output(&self, output: &CommandOutput, parent_dir: &Path) -> Result<()> {
        debug!(path = %parent_dir.display(), "Writing command results");
        let context = self.summarize().await?;
        let debug_inputs = self.cache_debug_inputs();
        let fingerprint = self.cache_fingerprint(&debug_inputs).await?;
        let metadata = Self::cache_metadata(&fingerprint);
        crate::artifact_cache::write_output(parent_dir, &context, &debug_inputs, output, &metadata)
            .await
    }

    /// Sends content to stdin of the command.
    pub fn send_stdin(&mut self, content: impl Into<String>) -> &mut Self {
        self.stdin_content = Some(content.into());
        self
    }

    /// Execute attempts without entering cache lookup or publishing an outcome.
    /// A retry remains inside the one owner admitted by the outer boundary.
    async fn run_uncached(
        &self,
        cancellation: &CancellationScope,
    ) -> Result<(CommandOutput, RetryBehaviour)> {
        let mut retry_behaviour = self.retry_behaviour;
        loop {
            let output = CommandProcess::spawn(self, cancellation)
                .await?
                .wait(self.stdin_content.as_deref(), self.timeout)
                .await?;
            if output.success() {
                return Ok((output, retry_behaviour));
            }

            match retry_behaviour {
                RetryBehaviour::Retry
                    if ["ERROR: Too Many Requests", "Error: Too Many Requests"]
                        .into_iter()
                        .any(|message| output.stderr.contains_str(message)) =>
                {
                    let mut sleep_duration = Duration::from_secs(30);
                    let needle = "'x-ms-user-quota-resets-after': '";
                    if let Some(pos) = output.stderr.find(needle) {
                        let start = pos + needle.len();
                        if let Some(end) = output.stderr[start..].find("'") {
                            let reset_after = String::from_utf8_lossy(&output.stderr[start..start + end]);
                            let parts = reset_after
                                .split(':')
                                .map(|part| part.parse::<u64>())
                                .collect::<Result<Vec<_>, _>>()?;
                            if let [hours, minutes, seconds] = parts.as_slice() {
                                let seconds = hours
                                    .checked_mul(3600)
                                    .and_then(|total| minutes.checked_mul(60).and_then(|minutes| total.checked_add(minutes)))
                                    .and_then(|total| total.checked_add(*seconds))
                                    .and_then(|total| total.checked_add(5))
                                    .ok_or_else(|| eyre::eyre!("command retry delay exceeds the supported duration"))?;
                                sleep_duration = Duration::from_secs(seconds);
                            }
                        }
                    }
                    warn!("Rate limit detected; retrying command after {sleep_duration:?}");
                    sleep(sleep_duration).await;
                    cancellation.bail_if_cancelled()?;
                    info!("Retrying command after rate-limit wait");
                }
                // Legacy CLI callers retain their current login policy until their
                // explicit provider-bound request migration. Credential-provider
                // subprocesses already bypass this general command boundary.
                RetryBehaviour::Retry
                    if [
                        "AADSTS70043",
                        "No subscription found. Run 'az account set' to select a subscription.",
                        "Please run 'az login' to setup account.",
                        "ERROR: (pii). Status: Response_Status.Status_InteractionRequired, Error code: 3399614467",
                    ]
                    .into_iter()
                    .any(|message| output.stderr.contains_str(message)) =>
                {
                    if !allow_interactive_reauthentication() {
                        bail!(
                            "Command failed due to an authentication challenge; interactive Azure CLI reauthentication is disabled for the selected/headless authentication source"
                        );
                    }
                    if std::env::var("CLOUD_TERRASTODON_REAUTH").unwrap_or_default().to_uppercase() == "DENY" {
                        bail!("Command failed due to bad auth, and automatic reauthentication is disabled by the CLOUD_TERRASTODON_REAUTH environment variable. Please refresh your credentials and try again.");
                    }
                    match LOGIN_LOCK.try_lock() {
                        Ok(login_guard) => {
                            debug!(
                                "Acquired login lock without waiting, there isn't a login in progress"
                            );
                            // Let the user know
                            warn!(
                                "Command failed due to bad auth. Refreshing credential, user action required in a moment..."
                            );

                            // Perform login command
                            // (avoid using azure crate to avoid a dependency)
                            // This could maybe be moved to cloud_terrastodon_auth
                            // We could also try and extract `--tenant {}` from args
                            // or just make the tenant id be explicitly set for CommandBuilder
                            // Reauthentication commands must not retry and reacquire this lock.
                            let tenant_id = CommandBuilder::new(CommandKind::AzureCLI)
                                .cancellation_safe()
                                .use_retry_behaviour(RetryBehaviour::Fail)
                                .args([
                                    "account",
                                    "list",
                                    "--query",
                                    "[?isDefault].tenantId",
                                    "--output",
                                    "tsv",
                                ])
                                .run_reauthentication_command(cancellation)
                                .await?
                                .stdout;
                            let tenant_id = tenant_id.trim();
                            if tenant_id.is_empty() {
                                warn!(
                                    "Failed to find tenant ID from default account, the login command without tenant ID has been flaky for me .-. trying anyways"
                                );
                                CommandBuilder::new(CommandKind::AzureCLI)
                                    .use_retry_behaviour(RetryBehaviour::Fail)
                                    .arg("login")
                                    .use_output_behaviour(OutputBehaviour::Display)
                                    .run_reauthentication_command(cancellation)
                                    .await?;
                            } else {
                                CommandBuilder::new(CommandKind::AzureCLI)
                                    .use_retry_behaviour(RetryBehaviour::Fail)
                                    .args([
                                        "login",
                                        "--tenant",
                                        tenant_id
                                            .to_str()
                                            .wrap_err("converting tenant id to str")?,
                                    ])
                                    .run_reauthentication_command(cancellation)
                                    .await?;
                            }

                            drop(login_guard);
                        }
                        Err(_) => {
                            debug!("Login lock busy, waiting for the login to complete");
                            warn!(
                                "Command failed due to bad auth. Waiting for login in progress..."
                            );
                            let _login_guard = LOGIN_LOCK.lock().await;
                        }
                    }


                    cancellation.bail_if_cancelled()?;
                    info!("Retrying command with refreshed credential");
                }
                RetryBehaviour::Fail | RetryBehaviour::Retry => {
                    return Ok((output, retry_behaviour));
                }
            }

            // Both retry routes execute once more without revisiting the cache,
            // repeating a polite delay, or reacquiring a future per-key lock.
            retry_behaviour = RetryBehaviour::Fail;
        }
    }

    /// Login infrastructure has no cache key and cannot retry or recurse into
    /// cache admission. Keep each nested command's own cancellation policy.
    fn run_reauthentication_command<'a>(
        &'a self,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        // Keep the nested command's previous boxing boundary: login branches
        // must not enlarge every ordinary command's debug poll frame.
        Box::pin(
            self.with_cancellation_policy(cancellation, move |scope| async move {
                let output = CommandProcess::spawn(self, &scope)
                    .await?
                    .wait(self.stdin_content.as_deref(), self.timeout)
                    .await?;
                self.require_success(output, RetryBehaviour::Fail).await
            }),
        )
    }

    async fn require_success(
        &self,
        output: CommandOutput,
        retry_behaviour: RetryBehaviour,
    ) -> Result<CommandOutput> {
        if output.success() {
            return Ok(output);
        }
        let dir = self.write_failure(&output).await?;
        let mut error = Err(Report::from(output).wrap_err(format!(
            "Command did not execute successfully, using retry behaviour {retry_behaviour:?}, dumped to {dir:?}",
        )));
        if matches!(self.output_behaviour, OutputBehaviour::Display) {
            error = error.wrap_err(format!(
                "The output behaviour was set to {:?} instead of {:?} so the stdout and stderr are not available in the dump, try scrolling up in your terminal.",
                OutputBehaviour::Display,
                OutputBehaviour::Capture,
            ));
        }
        error
    }

    /// Executes a command using its configured cancellation policy.
    #[track_caller]
    pub fn run_raw<'a>(
        &'a self,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.run_raw_scoped_from(Duration::ZERO, Location::caller(), cancellation)
    }

    /// Returns cached output immediately, or waits before an uncached execution.
    #[track_caller]
    pub fn run_raw_polite<'a>(
        &'a self,
        uncached_delay: Duration,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.run_raw_scoped_from(uncached_delay, Location::caller(), cancellation)
    }

    #[track_caller]
    fn run_raw_scoped_from<'a>(
        &'a self,
        uncached_delay: Duration,
        caller: &'static Location<'static>,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.with_cancellation_policy(cancellation, move |scope| async move {
            self.run_raw_from(uncached_delay, caller, &scope).await
        })
    }

    /// Runs within an already-admitted policy; retries reuse that same owner.
    fn run_raw_from<'a>(
        &'a self,
        uncached_delay: Duration,
        caller: &'static Location<'static>,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        // Preserve the old async-recursion boundary's bounded caller-frame size
        // while keeping retries in the uncached loop rather than recursive calls.
        Box::pin(async move {
            let summary = self.summarize().await?;
            let span = info_span!("command_run_raw", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current();

            async {
                match self.get_cached_output().await {
                    Ok(Some(output)) => return Ok(output),
                    Ok(None) => {}
                    Err(error) if error.is_cancellation() => {
                        return Err(error);
                    }
                    Err(error) => debug!(?self.cache_key, %error, "Cache load failed"),
                }

                if !uncached_delay.is_zero() {
                    debug!(
                        delay_ms = uncached_delay.as_millis(),
                        "Sleeping before uncached command execution"
                    );
                    sleep(uncached_delay).await;
                }

                let start = Instant::now();
                let output = async {
                    let (output, retry_behaviour) = self.run_uncached(cancellation).await?;
                    let output = self.require_success(output, retry_behaviour).await?;
                    if let Some(cache_key) = &self.cache_key {
                        cancellation
                            .finish_required(
                                "persisting command output and execution artifacts",
                                |_| async {
                                    if let Err(error) =
                                        self.write_output(&output, &cache_key.path_on_disk()).await
                                    {
                                        crate::artifact_cache::note_cache_write_failure(&error);
                                    }
                                    Ok::<_, Report>(())
                                },
                            )
                            .await?;
                    }
                    Ok(output)
                }
                .await;
                let elapsed = start.elapsed();
                debug!(
                    elapsed_ms = elapsed.as_millis(),
                    "Command executed in {}",
                    humantime::format_duration(elapsed),
                );
                output
            }
            .instrument(span)
            .await
            .wrap_err_with(|| {
                format!(
                    "Command::run_raw failed, called from {}",
                    RelativeLocation::from(caller)
                )
            })
            .wrap_err_with(|| format!("Invoking command failed: {summary}"))
        })
    }

    #[track_caller]
    pub fn run<'a, T: FromCommandOutput>(
        &'a self,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.run_from(None, Location::caller(), cancellation)
    }

    /// Returns decoded cached output immediately, or delays an uncached execution.
    #[track_caller]
    pub fn run_polite<'a, T: FromCommandOutput>(
        &'a self,
        uncached_delay: Duration,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.run_from(Some(uncached_delay), Location::caller(), cancellation)
    }

    #[track_caller]
    fn run_from<'a, T: FromCommandOutput>(
        &'a self,
        uncached_delay: Option<Duration>,
        caller: &'static Location<'static>,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.with_cancellation_policy(cancellation, move |scope| async move {
            let summary = self.summarize().await?;
            let (operation, span) = if uncached_delay.is_some() {
                ("Command::run_polite", info_span!("command_run_polite", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current())
            } else {
                ("Command::run", info_span!("command_run", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current())
            };
            async {
                let output = self
                    .run_raw_from(uncached_delay.unwrap_or_default(), caller, &scope)
                    .await
                    .wrap_err_with(|| format!("{operation} failed, called from {}", RelativeLocation::from(caller)))?;
                self.decode_output::<T, T, _>(output, Ok, caller, &summary).await
            }
            .instrument(span)
            .await
        })
    }

    #[track_caller]
    pub fn run_with_validator<'a, T, F>(
        &'a self,
        validator: F,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + Send + 'a
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T> + Send + 'static,
    {
        self.run_with_validator_from(validator, Location::caller(), cancellation)
    }

    /// Parse command output as `Raw`, then fallibly map it to a different output shape.
    /// Both parse and map failures are dumped through the command failure artifact path.
    #[track_caller]
    pub fn run_with_mapper<'a, Raw, Output, F>(
        &'a self,
        mapper: F,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<Output>> + Send + 'a
    where
        Raw: FromCommandOutput,
        Output: Send + 'static,
        F: FnOnce(Raw) -> Result<Output> + Send + 'static,
    {
        self.run_with_mapper_from(mapper, Location::caller(), cancellation)
    }

    #[track_caller]
    fn run_with_mapper_from<'a, Raw, Output, F>(
        &'a self,
        mapper: F,
        caller: &'static Location<'static>,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<Output>> + Send + 'a
    where
        Raw: FromCommandOutput,
        Output: Send + 'static,
        F: FnOnce(Raw) -> Result<Output> + Send + 'static,
    {
        self.with_cancellation_policy(cancellation, move |scope| async move {
            let summary = self.summarize().await?;
            let span = info_span!("command_run_with_mapper", summary, ?self.run_dir, ?self.cache_key, location=%RelativeLocation::from(caller)).or_current();
            async {
                let output = self.run_raw_from(Duration::ZERO, caller, &scope).await
                    .wrap_err_with(|| format!("Command::run_with_mapper failed, called from {}", RelativeLocation::from(caller)))?;
                self.decode_output::<Raw, Output, F>(output, mapper, caller, &summary).await
            }
            .instrument(span)
            .await
        })
    }

    async fn decode_output<Raw, Output, F>(
        &self,
        output: CommandOutput,
        mapper: F,
        caller: &'static Location<'static>,
        summary: &str,
    ) -> Result<Output>
    where
        Raw: FromCommandOutput,
        Output: Send + 'static,
        F: FnOnce(Raw) -> Result<Output> + Send + 'static,
    {
        let span = Span::current();
        let output = Arc::new(output);

        let decode_result = {
            let output = Arc::clone(&output);
            let span = span.clone();
            let dispatch = tracing::dispatcher::get_default(Clone::clone);
            spawn_blocking(move || {
                tracing::dispatcher::with_default(&dispatch, || {
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
            })
            .await?
        };

        match decode_result {
            Ok(results) => Ok(results),
            Err(CommandOutputDecodeError::Deserialize(e) | CommandOutputDecodeError::Map(e))
                if e.is_cancellation() =>
            {
                Err(e)
            }
            Err(CommandOutputDecodeError::Deserialize(e)) => {
                let dir = self
                    .write_failure(&output)
                    .instrument(span.or_current())
                    .await?;
                Err(e.wrap_err(format!(
                    "Deserialization failed!\n - Command: `{summary}`\n - Called by: \"{}\"\n - Dumped to: {dir:?}\n - Type: {}{}",
                    RelativeLocation::from(caller),
                    type_name::<Raw>(),
                    crate::artifact_cache::cache_clean_recommendation(self.cache_key.as_ref())
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
                    type_name::<Raw>(),
                    type_name::<Output>()
                )))
            }
        }
    }

    pub async fn run_with_validator_from<T, F>(
        &self,
        validator: F,
        caller: &'static Location<'static>,
        cancellation: &CancellationScope,
    ) -> Result<T>
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T>,
    {
        self.with_cancellation_policy(cancellation, |cancellation| async move {
            // Get stdout
            let output = self.run_raw_from(Duration::ZERO, caller, &cancellation).await?;

            // Parse
            match crate::json::from_slice(&output.stdout) {
                Ok(results) => match validator(results) {
                    Ok(results) => Ok(results),
                    Err(e) if e.is_cancellation() => Err(e),
                    Err(e) => {
                        let dir = self.write_failure(&output).await?;
                        Err(e).context(format!("Encountered validation error after successful invocation of `{}`\ncalled by \"{}\"\ndumped to {:?}",
                                    self.summarize().await?,
                                    RelativeLocation::from(caller),
                                    dir
                                ))
                    }
                },
                Err(e) if e.is_cancellation() => Err(e),
                Err(e) => {
                    let dir = self.write_failure(&output).await?;
                    Err(e.wrap_err(format!(
                        "deserializing `{}` failed\ncalled by \"{}\"\ndumped to {:?}{}",
                        self.summarize().await?,
                        RelativeLocation::from(caller),
                        dir,
                        crate::artifact_cache::cache_clean_recommendation(self.cache_key.as_ref())
                    )))
                }
            }
        })
        .await
    }

    pub async fn write_failure(&self, output: &CommandOutput) -> Result<PathBuf> {
        let context = self.summarize().await?;
        let debug_inputs = self.cache_debug_inputs();
        let fingerprint = self.cache_fingerprint(&debug_inputs).await?;
        let metadata = Self::cache_metadata(&fingerprint);
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

fn allow_interactive_reauthentication() -> bool {
    // Presence of any WIF input means Auto must not turn a failed token
    // request into an unrelated interactive Azure CLI login. The credentials
    // crate provides the detailed completeness error at the actual boundary.
    let has_workload_identity_input = [
        "AZURE_CLIENT_ID",
        "servicePrincipalId",
        "AZURE_TENANT_ID",
        "tenantId",
        "AZURE_FEDERATED_TOKEN",
        "idToken",
    ]
    .into_iter()
    .any(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()));
    let headless = std::env::var("CI")
        .or_else(|_| std::env::var("TF_BUILD"))
        .or_else(|_| std::env::var("BUILD_BUILDID"))
        .is_ok_and(|value| !value.is_empty())
        || !std::io::stdin().is_terminal();
    allow_interactive_reauthentication_for("auto", has_workload_identity_input, headless)
}

fn allow_interactive_reauthentication_for(
    selected_source: &str,
    has_workload_identity_input: bool,
    headless: bool,
) -> bool {
    if matches!(
        selected_source,
        "workload-identity"
            | "workload_identity"
            | "wif"
            | "browser"
            | "delegated"
            | "pat"
            | "personal-access-token"
            | "personal_access_token"
    ) || has_workload_identity_input
        || headless
    {
        return false;
    }
    true
}

#[cfg(test)]
mod execution_policy_tests {
    use super::CommandBuilder;
    use crate::CommandKind;
    use std::collections::BTreeMap;
    use std::ffi::OsStr;
    use tokio::process::Command;

    #[tokio::test]
    async fn removed_environment_changes_cache_identity_and_clears_prior_overrides()
    -> eyre::Result<()> {
        let mut builder = CommandBuilder::new(CommandKind::Other("unused-test-program".to_owned()));
        let fingerprint = builder.cache_fingerprint(&BTreeMap::new()).await?;
        builder.env_remove("CT_SYNTHETIC_SESSION");
        assert_ne!(
            fingerprint,
            builder.cache_fingerprint(&BTreeMap::new()).await?
        );

        builder.env("CT_SYNTHETIC_SESSION", "synthetic");
        // Windows treats differently cased environment names as the same key.
        let removed_name = if cfg!(windows) {
            "ct_synthetic_session"
        } else {
            "CT_SYNTHETIC_SESSION"
        };
        builder.env_remove(removed_name);
        let mut process = Command::new("unused-test-program");
        builder
            .kind
            .apply_args_and_envs(&builder, &mut process)
            .await?;
        assert!(
            process
                .as_std()
                .get_envs()
                .all(|(_, value)| value.is_none())
        );

        builder.env("CT_SYNTHETIC_SESSION", "replacement");
        let mut process = Command::new("unused-test-program");
        builder
            .kind
            .apply_args_and_envs(&builder, &mut process)
            .await?;
        assert!(
            process
                .as_std()
                .get_envs()
                .any(|(_, value)| { value == Some(OsStr::new("replacement")) })
        );
        Ok(())
    }

    #[tokio::test]
    async fn environment_prefix_removal_clears_overrides_and_changes_cache_identity()
    -> eyre::Result<()> {
        let mut builder = CommandBuilder::new(CommandKind::Other("unused-test-program".to_owned()));
        builder.env("CT_UNRELATED_TEST_VALUE", "keep");
        let baseline = builder.cache_fingerprint(&BTreeMap::new()).await?;
        let credential_name = if cfg!(windows) {
            "ct_synthetic_session_secret"
        } else {
            "CT_SYNTHETIC_SESSION_SECRET"
        };
        builder.env(credential_name, "synthetic-credential");
        builder.env_remove_prefix("CT_SYNTHETIC_SESSION_");
        assert_ne!(baseline, builder.cache_fingerprint(&BTreeMap::new()).await?);
        let mut process = Command::new("unused-test-program");
        builder
            .kind
            .apply_args_and_envs(&builder, &mut process)
            .await?;
        assert!(
            !process
                .as_std()
                .get_envs()
                .any(|(_, value)| { value == Some(OsStr::new("synthetic-credential")) })
        );
        assert!(
            process
                .as_std()
                .get_envs()
                .any(|(_, value)| value == Some(OsStr::new("keep")))
        );

        builder.env("CT_SYNTHETIC_SESSION_SECRET", "selected-credential");
        let mut process = Command::new("unused-test-program");
        builder
            .kind
            .apply_args_and_envs(&builder, &mut process)
            .await?;
        assert!(
            process
                .as_std()
                .get_envs()
                .any(|(_, value)| { value == Some(OsStr::new("selected-credential")) })
        );
        Ok(())
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn environment_replacement_has_one_case_insensitive_identity() -> eyre::Result<()> {
        let mut builder = CommandBuilder::new(CommandKind::Other("unused-test-program".to_owned()));
        builder.env("cloud_terrastodon_auth_partition", "unselected");
        builder.env("CLOUD_TERRASTODON_AUTH_PARTITION", "selected");
        let mut expected =
            CommandBuilder::new(CommandKind::Other("unused-test-program".to_owned()));
        expected.env("CLOUD_TERRASTODON_AUTH_PARTITION", "selected");
        assert_eq!(
            builder.cache_fingerprint(&BTreeMap::new()).await?,
            expected.cache_fingerprint(&BTreeMap::new()).await?
        );
        Ok(())
    }
}

#[cfg(test)]
mod auth_retry_tests {
    use super::allow_interactive_reauthentication_for;

    #[test]
    fn headless_and_wif_contexts_never_allow_cli_login() {
        assert!(!allow_interactive_reauthentication_for("auto", true, false));
        assert!(!allow_interactive_reauthentication_for("auto", false, true));
        assert!(!allow_interactive_reauthentication_for(
            "workload-identity",
            false,
            false
        ));
        assert!(!allow_interactive_reauthentication_for(
            "browser", false, false
        ));
        assert!(!allow_interactive_reauthentication_for("pat", false, false));
    }

    #[test]
    fn explicit_local_context_can_reauthenticate() {
        assert!(allow_interactive_reauthentication_for(
            "azure-cli",
            false,
            false
        ));
    }
}
