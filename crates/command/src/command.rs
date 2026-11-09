use crate::CancellationScope;
use crate::CommandArgument;
use crate::CommandKind;
use crate::CommandOutput;
use crate::FromCommandOutput;
use crate::OutputBehaviour;
use crate::PathMapper;
use crate::RetryBehaviour;
use crate::command_process::CommandProcess;
use crate::process_fingerprint::ProcessFingerprint;
pub use bstr;
use bstr::BString;
use bstr::ByteSlice;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_cache::ArtifactEntry;
use cloud_terrastodon_cache::ArtifactFilePath;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_relative_location::RelativeLocation;
use eyre::Context;
use eyre::Report;
use eyre::Result;
use std::any::type_name;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::future::Future;
use std::future::IntoFuture;
use std::panic::Location;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use teamy_cancellation::CancellationReportExt;
use tokio::task::spawn_blocking;
use tokio::time::sleep;
use tracing::Instrument;
use tracing::Span;
use tracing::info;
use tracing::info_span;
use tracing::warn;

#[derive(Debug, Clone)]
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
    pub(crate) artifact_retention: ArtifactRetention,
}

impl Default for CommandBuilder {
    fn default() -> Self {
        Self {
            kind: CommandKind::default(),
            args: Vec::new(),
            adjacent_files: HashMap::new(),
            env: HashMap::new(),
            removed_env: BTreeSet::new(),
            removed_env_prefixes: BTreeSet::new(),
            run_dir: None,
            retry_behaviour: RetryBehaviour::default(),
            output_behaviour: OutputBehaviour::default(),
            cache_key: None,
            should_announce: false,
            timeout: None,
            stdin_content: None,
            cancellation_safe: false,
            artifact_retention: ArtifactRetention::Full,
        }
    }
}

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

    async fn with_cancellation_policy<'a, C, F, T>(
        &'a self,
        cancellation: &'a CancellationScope,
        execute: C,
    ) -> Result<T>
    where
        C: FnOnce(CancellationScope) -> F + 'a,
        F: IntoFuture<Output = Result<T>> + 'a,
        F::IntoFuture: 'a,
        T: 'a,
    {
        if self.cancellation_safe {
            cancellation.cancellable_scope(execute).await
        } else {
            cancellation
                .deferred(
                    "finishing command execution, decoding, and caller validation",
                    execute,
                )
                .await
        }
    }

    async fn with_executor_cancellation_policy<'a, C, F, T>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
        execute: C,
    ) -> Result<T>
    where
        C: FnOnce(CloudTerrastodonExecutor) -> F + 'a,
        F: IntoFuture<Output = Result<T>> + 'a,
        F::IntoFuture: 'a,
        T: 'a,
    {
        if self.cancellation_safe {
            executor.cancellable_scope(execute).await
        } else {
            executor
                .deferred(
                    "finishing command execution, decoding, caller validation, and artifacts",
                    execute,
                )
                .await
        }
    }

    /// Suppress every raw input, output, and error artifact for sensitive commands.
    /// Sensitive failures do not include captured stdout, stderr, or decoder text.
    pub fn artifact_retention(&mut self, retention: ArtifactRetention) -> &mut Self {
        self.artifact_retention = retention;
        self
    }

    pub fn use_command(&mut self, kind: CommandKind) -> &mut Self {
        self.kind = kind;
        self
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
            kind: self.kind.clone(),
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
        let bytes = facet_json::to_vec(&fingerprint).map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
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
                            let reset_after =
                                String::from_utf8_lossy(&output.stderr[start..start + end]);
                            let parts = reset_after
                                .split(':')
                                .map(|part| part.parse::<u64>())
                                .collect::<Result<Vec<_>, _>>()?;
                            if let [hours, minutes, seconds] = parts.as_slice() {
                                let seconds = hours
                                    .checked_mul(3600)
                                    .and_then(|total| {
                                        minutes
                                            .checked_mul(60)
                                            .and_then(|minutes| total.checked_add(minutes))
                                    })
                                    .and_then(|total| total.checked_add(*seconds))
                                    .and_then(|total| total.checked_add(5))
                                    .ok_or_else(|| {
                                        eyre::eyre!(
                                            "command retry delay exceeds the supported duration"
                                        )
                                    })?;
                                sleep_duration = Duration::from_secs(seconds);
                            }
                        }
                    }
                    warn!("Rate limit detected; retrying command after {sleep_duration:?}");
                    sleep(sleep_duration).await;
                    cancellation.bail_if_cancelled()?;
                    info!("Retrying command after rate-limit wait");
                }
                RetryBehaviour::Fail | RetryBehaviour::Retry => {
                    return Ok((output, retry_behaviour));
                }
            }

            // Retry once below the canonical work boundary.
            retry_behaviour = RetryBehaviour::Fail;
        }
    }

    fn artifact_key(&self, caller: &'static Location<'static>) -> Result<CacheKey> {
        Ok(self.cache_key.clone().unwrap_or(CacheKey {
            scope: CacheScope::try_from_components([
                "uncached".to_owned(),
                caller.file().to_owned(),
                caller.line().to_string(),
                caller.column().to_string(),
            ])?,
            valid_for: Duration::ZERO,
        }))
    }

    fn artifact_inputs(&self) -> Result<BTreeMap<ArtifactFilePath, Vec<u8>>> {
        self.adjacent_files
            .iter()
            .map(|(path, bytes)| {
                let path = path
                    .to_str()
                    .ok_or_else(|| eyre::eyre!("command input path is not Unicode"))?;
                #[cfg(windows)]
                let path = path.replace('\\', "/");
                Ok((ArtifactFilePath::new(path)?, bytes.to_vec()))
            })
            .collect()
    }

    fn diagnostics(
        &self,
        summary: &str,
        output: Option<&CommandOutput>,
        error: Option<&Report>,
    ) -> Result<BTreeMap<ArtifactFilePath, Vec<u8>>> {
        let mut files = BTreeMap::from([(
            ArtifactFilePath::new("context.txt")?,
            summary.as_bytes().to_vec(),
        )]);
        if let Some(output) = output {
            files.insert(
                ArtifactFilePath::new("stdout.json")?,
                output.stdout.to_vec(),
            );
            files.insert(
                ArtifactFilePath::new("stderr.json")?,
                output.stderr.to_vec(),
            );
            files.insert(
                ArtifactFilePath::new("status.txt")?,
                output.status.to_string().into_bytes(),
            );
        }
        if let Some(error) = error {
            files.insert(
                ArtifactFilePath::new("error.txt")?,
                format!("{error:?}").into_bytes(),
            );
        }
        Ok(files)
    }

    async fn diagnostic_summary(&self) -> Result<String> {
        if self.artifact_retention == ArtifactRetention::Disabled {
            Ok("sensitive command (arguments and output suppressed)".to_owned())
        } else {
            self.summarize().await
        }
    }

    async fn decode<T: Send + 'static>(
        &self,
        output: CommandOutput,
        decoder: fn(&CommandOutput) -> Result<T>,
    ) -> Result<(CommandOutput, Result<T>)> {
        let span = Span::current();
        let dispatch = tracing::dispatcher::get_default(Clone::clone);
        let sensitive = self.artifact_retention == ArtifactRetention::Disabled;
        spawn_blocking(move || {
            tracing::dispatcher::with_default(&dispatch, || {
                let _entered = span.enter();
                let decoded = if !output.success() {
                    if sensitive {
                        Err(eyre::eyre!(
                            "sensitive command failed with status {}",
                            output.status
                        ))
                    } else {
                        Err(Report::from(output.clone()))
                    }
                } else {
                    decoder(&output).map_err(|error| {
                        if sensitive {
                            eyre::eyre!(
                                "sensitive command output could not be decoded as {}",
                                type_name::<T>()
                            )
                        } else {
                            error
                        }
                    })
                };
                (output, decoded)
            })
        })
        .await
        .wrap_err("command decoder worker failed")
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "canonical identity, owned transport outcome, decoding, and diagnostics are distinct contracts"
    )]
    async fn canonical<T: Send + 'static>(
        &self,
        cancellation: &CancellationScope,
        entry: ArtifactEntry,
        reusable: bool,
        delay: Duration,
        caller: &'static Location<'static>,
        summary: &str,
        decoder: fn(&CommandOutput) -> Result<T>,
    ) -> Result<WorkOutput<T>> {
        if reusable && let Some(loaded) = entry.load(cancellation).await? {
            // Move the complete loaded proof through decoding. Its original age
            // and generation must survive the blocking worker unchanged.
            let span = Span::current();
            let dispatch = tracing::dispatcher::get_default(Clone::clone);
            let (loaded, decoded) = spawn_blocking(move || {
                tracing::dispatcher::with_default(&dispatch, || {
                    let _entered = span.enter();
                    let decoded =
                        facet_json::from_slice::<CommandOutput>(&loaded.artifacts.payload)
                            .map_err(|error| eyre::eyre!("{error:?}"))
                            .and_then(|output| {
                                if output.success() {
                                    decoder(&output)
                                } else {
                                    Err(Report::from(output))
                                }
                            });
                    (loaded, decoded)
                })
            })
            .await
            .wrap_err("cached command decoder worker failed")?;
            return match decoded {
                Ok(value) => Ok(loaded.decoded(value)),
                Err(error) => {
                    cancellation.finish_required(
                        "invalidate rejected command output and retain its diagnosis",
                        |child| async move {
                            entry.reject(&child).await?;
                            let mut artifacts = loaded.artifacts;
                            artifacts.diagnostics.extend(self.diagnostics(summary, None, Some(&error))?);
                            let directory = if let Some(attempt) = entry.begin_attempt(self.artifact_inputs()?, &child).await? {
                                Some(attempt.finish(artifacts, ArtifactObservation::now(), &child).await?.directory().to_path_buf())
                            } else { None };
                            Err(error).wrap_err(format!(
                                "Canonical cached command decoding failed; called from {}; artifacts: {directory:?}",
                                RelativeLocation::from(caller)
                            ))
                        },
                    ).await
                }
            };
        }

        let attempt = entry
            .begin_attempt(self.artifact_inputs()?, cancellation)
            .await?;
        if !delay.is_zero() {
            sleep(delay).await;
            cancellation.bail_if_cancelled()?;
        }
        let outcome = self.run_uncached(cancellation).await;
        let observation = ArtifactObservation::now();
        cancellation.finish_required(
            "decode the command outcome and retain its artifacts",
            |child| async move {
                let (output, decoded) = match outcome {
                    Ok((output, _retry)) => {
                        let (output, decoded) = self.decode(output, decoder).await?;
                        (Some(output), decoded)
                    }
                    Err(error) => (None, Err(error)),
                };
                let stored = if let Some(attempt) = attempt {
                    let payload = output
                        .as_ref()
                        .map(facet_json::to_vec)
                        .transpose()
                        .map_err(|error| eyre::eyre!("{error:?}"))?
                        .unwrap_or_default();
                    let artifacts = ArtifactSet {
                        payload,
                        diagnostics: self.diagnostics(summary, output.as_ref(), decoded.as_ref().err())?,
                    };
                    Some(attempt.finish(artifacts, observation, &child).await?)
                } else { None };
                match decoded {
                    Ok(value) => {
                        if reusable && let Some(stored) = &stored {
                            entry.publish(stored, &child).await?;
                        }
                        Ok(WorkOutput::new(value, observation))
                    }
                    Err(error) => {
                        let directory = stored.as_ref().map(|stored| stored.directory());
                        Err(error).wrap_err(format!(
                            "Command execution or canonical decoding failed; called from {}; artifacts: {directory:?}",
                            RelativeLocation::from(caller),
                        ))
                    }
                }
            },
        ).await
    }

    fn decode_json<T: FromCommandOutput>(output: &CommandOutput) -> Result<T> {
        facet_json::from_slice(output.stdout.to_str_lossy().as_bytes())
            .map_err(|error| eyre::eyre!("{error:?}"))
    }

    // Local execution has no work identity, provider, cache root, or persistent
    // artifacts. Managed callers admit semantic work before using this same
    // subprocess/decode machinery; a scope alone cannot track cache ancestry.
    fn run_local_from<'a, T: Send + 'static>(
        &'a self,
        cancellation: &'a CancellationScope,
        delay: Duration,
        caller: &'static Location<'static>,
        decoder: fn(&CommandOutput) -> Result<T>,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.with_cancellation_policy(cancellation, move |scope| async move {
            eyre::ensure!(
                self.cache_key.is_none(),
                "plain command execution cannot use a cache key; use an explicit cached or artifact execution method"
            );
            let summary = self.diagnostic_summary().await?;
            let span = info_span!("command_run", summary, location=%RelativeLocation::from(caller))
                .or_current();
            async {
                if !delay.is_zero() {
                    sleep(delay).await;
                    scope.bail_if_cancelled()?;
                }
                let outcome = self.run_uncached(&scope).await;
                scope.finish_required("decode the observed command outcome", |_completion| async move {
                    let (output, _) = outcome?;
                    let (_, decoded) = self.decode(output, decoder).await?;
                    decoded
                }).await.wrap_err_with(|| format!(
                    "Command execution or decoding failed; called from {}",
                    RelativeLocation::from(caller),
                ))
            }.instrument(span).await
        })
    }

    /// Run a local process and decode its output without a cache or artifact store.
    ///
    /// Semantic requests that need admission, dependency tracking, or retained
    /// artifacts use the explicit executor-managed methods instead. A configured
    /// cache key is rejected here rather than silently ignored.
    #[track_caller]
    pub fn run<'a, T: FromCommandOutput>(
        &'a self,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.run_local_from(
            cancellation,
            Duration::ZERO,
            Location::caller(),
            Self::decode_json::<T>,
        )
    }

    /// Run a local process without cache lookup or persistent execution artifacts.
    #[track_caller]
    pub fn run_raw<'a>(
        &'a self,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.run_local_from(cancellation, Duration::ZERO, Location::caller(), |output| {
            Ok(output.clone())
        })
    }

    /// Delay this independent local invocation before starting its process.
    #[track_caller]
    pub fn run_polite<'a, T: FromCommandOutput>(
        &'a self,
        delay: Duration,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.run_local_from(
            cancellation,
            delay,
            Location::caller(),
            Self::decode_json::<T>,
        )
    }

    /// Delay a local process without entering the executor or artifact lifecycle.
    #[track_caller]
    pub fn run_raw_polite<'a>(
        &'a self,
        delay: Duration,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.run_local_from(cancellation, delay, Location::caller(), |output| {
            Ok(output.clone())
        })
    }

    fn local_policy_failure(&self, error: Report, caller: &'static Location<'static>) -> Report {
        if error.is_cancellation() {
            error
        } else if self.artifact_retention == ArtifactRetention::Disabled {
            eyre::eyre!(
                "sensitive command output policy rejected its result; called from {}",
                RelativeLocation::from(caller)
            )
        } else {
            error.wrap_err(format!(
                "Command output mapping or validation failed; called from {}",
                RelativeLocation::from(caller)
            ))
        }
    }

    /// Decode locally and apply caller policy without requiring shared execution.
    /// The callback and output may borrow non-Send caller state.
    #[track_caller]
    pub fn run_with_mapper<'a, Raw, Output, F>(
        &'a self,
        mapper: F,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<Output>> + 'a
    where
        Raw: FromCommandOutput,
        Output: 'a,
        F: FnOnce(Raw) -> Result<Output> + 'a,
    {
        let caller = Location::caller();
        self.with_cancellation_policy(cancellation, move |scope| async move {
            let value = self
                .run_local_from(&scope, Duration::ZERO, caller, Self::decode_json::<Raw>)
                .await?;
            mapper(value).map_err(|error| self.local_policy_failure(error, caller))
        })
        .instrument(info_span!("command_output_policy", location=%RelativeLocation::from(caller)))
    }

    /// Validate this invocation's owned value without persistent artifacts.
    #[track_caller]
    pub fn run_with_validator<'a, T, F>(
        &'a self,
        validator: F,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + 'a
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T> + 'a,
    {
        self.run_with_validator_from(validator, Location::caller(), cancellation)
    }

    /// Validate locally with the caller captured by a forwarding operation.
    pub fn run_with_validator_from<'a, T, F>(
        &'a self,
        validator: F,
        caller: &'static Location<'static>,
        cancellation: &'a CancellationScope,
    ) -> impl Future<Output = Result<T>> + 'a
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T> + 'a,
    {
        self.with_cancellation_policy(cancellation, move |scope| async move {
            let value = self
                .run_local_from(&scope, Duration::ZERO, caller, Self::decode_json::<T>)
                .await?;
            validator(value).map_err(|error| self.local_policy_failure(error, caller))
        })
        .instrument(info_span!("command_output_policy", location=%RelativeLocation::from(caller)))
    }

    #[track_caller]
    fn run_with_artifacts_from<'a, T: Send + 'static>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
        delay: Duration,
        caller: &'static Location<'static>,
        decoder: fn(&CommandOutput) -> Result<T>,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        Box::pin(
            self.with_executor_cancellation_policy(executor, move |child| async move {
                let summary = self.diagnostic_summary().await?;
                let key = self.artifact_key(caller)?;
                let fingerprint =
                    NonEmptyString::new(self.cache_fingerprint(&self.cache_debug_inputs()).await?)?;
                let span =
                    info_span!("command_run", summary, location=%RelativeLocation::from(caller))
                        .or_current();
                child
                    .uncached_work_with_artifacts(
                        ExecutorKind::Command,
                        key,
                        CacheOperationOwner::of::<Self>(),
                        &AuthenticationSelection::Anonymous,
                        fingerprint,
                        self.artifact_retention,
                        |attempt, entry| async move {
                            Ok(self
                                .canonical(
                                    attempt.cancellation(),
                                    entry,
                                    false,
                                    delay,
                                    caller,
                                    &summary,
                                    decoder,
                                )
                                .await?
                                .value)
                        },
                    )?
                    .instrument(span)
                    .await
            }),
        )
    }

    #[track_caller]
    fn run_cached_from<'a, T: Send + Sync + 'static>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
        delay: Duration,
        caller: &'static Location<'static>,
        decoder: fn(&CommandOutput) -> Result<T>,
    ) -> impl Future<Output = Result<Arc<T>>> + Send + 'a {
        Box::pin(self.with_executor_cancellation_policy(executor, move |child| async move {
            eyre::ensure!(matches!(self.output_behaviour, OutputBehaviour::Capture), "shared command execution requires captured output; use owned execution to display output on every invocation");
            let key = self.cache_key.clone().ok_or_else(|| eyre::eyre!("cached commands require an explicit cache key"))?;
            let summary = self.diagnostic_summary().await?;
            let fingerprint = NonEmptyString::new(self.cache_fingerprint(&self.cache_debug_inputs()).await?)?;
            let span = info_span!("command_run_cached", summary, location=%RelativeLocation::from(caller)).or_current();
            child.cached_work_with_artifacts(
                ExecutorKind::Command,
                key,
                owner,
                &AuthenticationSelection::Anonymous,
                fingerprint,
                WorkReuse::ReusableRead,
                self.artifact_retention,
                |attempt, entry| async move {
                    self.canonical(attempt.cancellation(), entry, true, delay, caller, &summary, decoder).await
                },
            )?.instrument(span).await
        }))
    }

    /// Admit independent executor-managed work and retain execution artifacts.
    /// Return owned JSON-decoded output; configured keys identify artifacts only.
    #[track_caller]
    pub fn run_with_artifacts<'a, T: FromCommandOutput>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.run_with_artifacts_from(
            executor,
            Duration::ZERO,
            Location::caller(),
            Self::decode_json::<T>,
        )
    }

    /// Admit independent executor-managed work and retain its raw outcome.
    #[track_caller]
    pub fn run_raw_with_artifacts<'a>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.run_with_artifacts_from(executor, Duration::ZERO, Location::caller(), |output| {
            Ok(output.clone())
        })
    }

    /// Share one canonical JSON value for an explicitly owned cache identity.
    #[track_caller]
    pub fn run_cached<'a, T: FromCommandOutput + Sync>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<T>>> + Send + 'a {
        self.run_cached_from(
            executor,
            owner,
            Duration::ZERO,
            Location::caller(),
            Self::decode_json::<T>,
        )
    }

    /// Share a canonical raw outcome without repeating process execution.
    #[track_caller]
    pub fn run_raw_cached<'a>(
        &'a self,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<CommandOutput>>> + Send + 'a {
        self.run_cached_from(
            executor,
            owner,
            Duration::ZERO,
            Location::caller(),
            |output| Ok(output.clone()),
        )
    }

    /// Delay independent executor-managed work while retaining its artifacts.
    #[track_caller]
    pub fn run_polite_with_artifacts<'a, T: FromCommandOutput>(
        &'a self,
        delay: Duration,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + Send + 'a {
        self.run_with_artifacts_from(executor, delay, Location::caller(), Self::decode_json::<T>)
    }

    /// Delay independent executor-managed work and retain its raw outcome.
    #[track_caller]
    pub fn run_raw_polite_with_artifacts<'a>(
        &'a self,
        delay: Duration,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<CommandOutput>> + Send + 'a {
        self.run_with_artifacts_from(executor, delay, Location::caller(), |output| {
            Ok(output.clone())
        })
    }

    /// Delay only a cache miss, including its bounded rate-limit retry.
    #[track_caller]
    pub fn run_cached_polite<'a, T: FromCommandOutput + Sync>(
        &'a self,
        delay: Duration,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<T>>> + Send + 'a {
        self.run_cached_from(
            executor,
            owner,
            delay,
            Location::caller(),
            Self::decode_json::<T>,
        )
    }

    #[track_caller]
    pub fn run_raw_cached_polite<'a>(
        &'a self,
        delay: Duration,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<CommandOutput>>> + Send + 'a {
        self.run_cached_from(executor, owner, delay, Location::caller(), |output| {
            Ok(output.clone())
        })
    }

    async fn retain_callback_failure<T: 'static>(
        &self,
        executor: &CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
        caller: &'static Location<'static>,
        error: Report,
    ) -> Report {
        if self.artifact_retention == ArtifactRetention::Disabled {
            return eyre::eyre!("sensitive command output policy rejected its result");
        }
        let retained = async {
            let summary = self.diagnostic_summary().await?;
            let mut key = self.artifact_key(caller)?;
            key.valid_for = Duration::ZERO;
            let entry = executor.cache()?.artifacts::<T>(
                ExecutorKind::Command,
                key,
                owner,
                executor.admit(&AuthenticationSelection::Anonymous)?,
                NonEmptyString::new(self.cache_fingerprint(&self.cache_debug_inputs()).await?)?,
                self.artifact_retention,
            )?;
            if let Some(attempt) = entry
                .begin_attempt(self.artifact_inputs()?, executor.cancellation())
                .await?
            {
                let stored = attempt
                    .finish(
                        ArtifactSet {
                            payload: Vec::new(),
                            diagnostics: self.diagnostics(&summary, None, Some(&error))?,
                        },
                        ArtifactObservation::now(),
                        executor.cancellation(),
                    )
                    .await?;
                eyre::Ok(Some(stored.directory().to_path_buf()))
            } else {
                Ok(None)
            }
        }
        .await;
        match retained {
            Ok(directory) => error.wrap_err(format!(
                "Command output mapping or validation failed; called from {}; artifacts: {directory:?}",
                RelativeLocation::from(caller),
            )),
            Err(retention_error) => error.wrap_err(format!(
                "Command output mapping or validation failed; could not retain diagnostics: {retention_error:#}"
            )),
        }
    }

    /// Retain an owned execution, then run caller policy without a blocking worker.
    /// The callback and its output may borrow caller-owned, non-Send state.
    #[track_caller]
    pub fn run_with_artifacts_and_mapper<'a, Raw, Output, F>(
        &'a self,
        mapper: F,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<Output>> + 'a
    where
        Raw: FromCommandOutput,
        Output: 'a,
        F: FnOnce(Raw) -> Result<Output> + 'a,
    {
        let caller = Location::caller();
        self.with_executor_cancellation_policy(executor, move |child| async move {
            let value = self
                .run_with_artifacts_from(&child, Duration::ZERO, caller, Self::decode_json::<Raw>)
                .await?;
            match mapper(value) {
                Ok(value) => Ok(value),
                Err(error) if error.is_cancellation() => Err(error),
                Err(error) => Err(self
                    .retain_callback_failure::<Raw>(
                        &child,
                        CacheOperationOwner::of::<Self>(),
                        caller,
                        error,
                    )
                    .await),
            }
        })
        .instrument(info_span!("command_output_policy", location=%RelativeLocation::from(caller)))
    }

    /// Validate an owned managed result, retaining rejection diagnostics.
    #[track_caller]
    pub fn run_with_artifacts_and_validator<'a, T, F>(
        &'a self,
        validator: F,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + 'a
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T> + 'a,
    {
        self.run_with_artifacts_and_validator_from(validator, Location::caller(), executor)
    }

    /// Retain managed execution and validation using a forwarding caller's site.
    pub fn run_with_artifacts_and_validator_from<'a, T, F>(
        &'a self,
        validator: F,
        caller: &'static Location<'static>,
        executor: &'a CloudTerrastodonExecutor,
    ) -> impl Future<Output = Result<T>> + 'a
    where
        T: FromCommandOutput,
        F: FnOnce(T) -> Result<T> + 'a,
    {
        self.with_executor_cancellation_policy(executor, move |child| async move {
            let value = self
                .run_with_artifacts_from(&child, Duration::ZERO, caller, Self::decode_json::<T>)
                .await?;
            match validator(value) {
                Ok(value) => Ok(value),
                Err(error) if error.is_cancellation() => Err(error),
                Err(error) => Err(self
                    .retain_callback_failure::<T>(
                        &child,
                        CacheOperationOwner::of::<Self>(),
                        caller,
                        error,
                    )
                    .await),
            }
        })
        .instrument(info_span!("command_output_policy", location=%RelativeLocation::from(caller)))
    }

    /// Apply this caller's projection on every result, including hits and joiners.
    /// Rejected caller policy does not evict the operation's canonical output.
    #[track_caller]
    pub fn run_cached_with_mapper<'a, Raw, Output, F>(
        &'a self,
        mapper: F,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Output>> + 'a
    where
        Raw: FromCommandOutput + Sync,
        Output: 'a,
        F: FnOnce(Arc<Raw>) -> Result<Output> + 'a,
    {
        let caller = Location::caller();
        self.with_executor_cancellation_policy(executor, move |child| async move {
            let value = self
                .run_cached_from(
                    &child,
                    owner,
                    Duration::ZERO,
                    caller,
                    Self::decode_json::<Raw>,
                )
                .await?;
            match mapper(value) {
                Ok(value) => Ok(value),
                Err(error) if error.is_cancellation() => Err(error),
                Err(error) => Err(self
                    .retain_callback_failure::<Raw>(&child, owner, caller, error)
                    .await),
            }
        })
        .instrument(info_span!("command_output_policy", location=%RelativeLocation::from(caller)))
    }

    /// Check caller policy without copying or replacing a canonical shared value.
    #[track_caller]
    pub fn run_cached_with_validator<'a, T, F>(
        &'a self,
        validator: F,
        executor: &'a CloudTerrastodonExecutor,
        owner: CacheOperationOwner,
    ) -> impl Future<Output = Result<Arc<T>>> + 'a
    where
        T: FromCommandOutput + Sync,
        F: FnOnce(&T) -> Result<()> + 'a,
    {
        self.run_cached_with_mapper(
            move |value: Arc<T>| {
                validator(&value)?;
                Ok(value)
            },
            executor,
            owner,
        )
    }
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
