use crate::AuthenticationProvider;
use crate::CollectionProgress;
use crate::ExecutableRequest;
use crate::ExecutionEvent;
use crate::ExecutionFuture;
use crate::RequestExecution;
use crate::execution_mode::ExecutionMode;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_cache::ArtifactEntry;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::CacheEntryIdentity;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkAttempt;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use eyre::Result;
use eyre::ensure;
use tracing::warn;
use std::future::Future;
use std::future::IntoFuture;
use std::panic::Location;
use std::sync::Arc;
use std::time::Duration;
use teamy_cancellation::CancellationError;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::mpsc::UnboundedSender;
use tracing::Instrument;

/// Shared execution resources bound to an already-owned invocation lifetime.
///
/// Construction performs no discovery or I/O and creates no cancellation root.
/// Clones share the cache and provider; child callbacks retain dependency lineage
/// while receiving their own linked scope. A clone cannot extend its scope's life.
///
/// Pass this context to operations that need those execution resources. Helpers
/// that only need cancellation receive [`CancellationScope`] directly. The child
/// callbacks below rebind executor state for nested requests; they are not a
/// replacement for using cancellation scopes in local work.
#[derive(Clone)]
pub struct CloudTerrastodonExecutor {
    // Private fields preserve scope ownership, execution restrictions, and cache
    // ancestry when rebinding children.
    cache: WorkCache,
    authentication: Arc<dyn AuthenticationProvider>,
    cancellation: CancellationScope,
    mode: ExecutionMode,
    observer: Option<UnboundedSender<ExecutionEvent>>,
}

impl CloudTerrastodonExecutor {
    /// Bind explicitly supplied resources to an existing bounded scope.
    pub fn new(
        cache: WorkCache,
        authentication: Arc<dyn AuthenticationProvider>,
        cancellation: CancellationScope,
    ) -> Self {
        Self {
            cache,
            authentication,
            cancellation,
            mode: ExecutionMode::Normal(None),
            observer: None,
        }
    }

    /// Explicitly select another provider while retaining this execution lifetime.
    ///
    /// Callers resolving an authentication override own this decision. Request
    /// adapters must not replace an injected provider implicitly. The returned
    /// handle shares the cache, cancellation scope, and semantic dependency
    /// ancestry; the original handle keeps its provider. Credential infrastructure
    /// cannot replace its provider to escape authorization reentry restrictions.
    pub fn with_authentication(
        &self,
        authentication: Arc<dyn AuthenticationProvider>,
    ) -> Result<Self> {
        ensure!(
            matches!(self.mode, ExecutionMode::Normal(_)),
            "credential infrastructure cannot replace its authentication provider"
        );
        Ok(Self {
            authentication,
            ..self.clone()
        })
    }

    /// The bounded scope for checkpoints and lower transport operations.
    pub fn cancellation(&self) -> &CancellationScope {
        &self.cancellation
    }

    /// The explicit owner for invalidation and low-level artifact adapters.
    /// Semantic initialization uses [`Self::cached_work`] or
    /// [`Self::cached_work_with_artifacts`] to retain admission and dependency
    /// ancestry rather than calling the low cache directly.
    /// Credential infrastructure cannot access general execution artifacts.
    pub fn cache(&self) -> Result<&WorkCache> {
        ensure!(
            matches!(self.mode, ExecutionMode::Normal(_)),
            "credential infrastructure cannot access the general work cache"
        );
        Ok(&self.cache)
    }

    /// Bind a request without running it, capturing its caller's tracing context.
    #[track_caller]
    pub fn execute<'a, R>(&'a self, request: R) -> RequestExecution<'a, R>
    where
        R: ExecutableRequest + 'a,
        R::Output: 'a,
        R::Error: 'a,
    {
        RequestExecution {
            executor: self,
            request,
            invalidation: None,
            span: tracing::trace_span!(
                "execute_request",
                request = std::any::type_name::<R>(),
                file = Location::caller().file(),
                line = Location::caller().line(),
                column = Location::caller().column(),
            )
            .or_current(),
        }
    }

    pub(crate) fn with_observer(&self, observer: UnboundedSender<ExecutionEvent>) -> Self {
        Self {
            observer: Some(observer),
            ..self.clone()
        }
    }

    pub(crate) fn report_event(&self, event: ExecutionEvent) {
        if let Some(observer) = &self.observer {
            // A dropped observer is not an execution failure. The channel carries
            // notifications only and never controls the lifetime of the work.
            if let Err(e) = observer.send(event) {
                warn!("failed to report execution event: {e:#}");
            }
        }
    }

    /// Report progress belonging to this initializer, without retaining a callback
    /// in request data or a shared cache entry. Waiting callers do not inherit it.
    pub fn report_progress(&self, progress: CollectionProgress) {
        self.report_event(ExecutionEvent::Progress(progress));
    }

    /// Check owned authentication policy without acquiring credentials.
    pub fn admit(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        self.authentication.admit(selection)
    }

    /// Acquire credentials in a linked infrastructure scope, outside semantic work.
    #[track_caller]
    pub fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(
            self.cancellable_scope(move |mut child| async move {
                ensure!(
                    matches!(child.mode, ExecutionMode::Normal(_)),
                    "credential acquisition cannot recursively request authorization"
                );
                child.admit(selection)?;
                child.mode = ExecutionMode::NoWorkCacheOrAuthorizationReentry;
                child
                    .authentication
                    .authorization(selection, resource, &child)
                    .await
            })
            .in_current_span(),
        )
    }

    /// Resolve a registry challenge in isolated credential infrastructure.
    /// The provider cannot cache token exchanges as semantic work or recursively
    /// request authorization through the restricted child executor.
    #[track_caller]
    pub fn container_registry_authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        challenge: &'a ContainerRegistryBearerChallenge,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(
            self.cancellable_scope(move |mut child| async move {
                ensure!(
                    matches!(child.mode, ExecutionMode::Normal(_)),
                    "credential acquisition cannot recursively request authorization"
                );
                ensure!(
                    matches!(selection, AuthenticationSelection::AzureBearer { .. }),
                    "Container Registry requires an explicitly tenant-bound bearer selection"
                );
                child.admit(selection)?;
                child.mode = ExecutionMode::NoWorkCacheOrAuthorizationReentry;
                child
                    .authentication
                    .container_registry_authorization(selection, challenge, &child)
                    .await
            })
            .in_current_span(),
        )
    }

    /// Bind canonical work after admission, retaining one decoded result per key.
    ///
    /// Admission is checked at binding, before cache access, around initialization,
    /// and before returning a result. A denial or identity change while waiting
    /// cannot reuse a previously bound hit or start a retry under the old identity.
    /// The initializer owns its attempt and receives the attempt's child scope;
    /// nested cached requests automatically record their semantic dependencies.
    /// Caller-specific mapping/validation belongs after the returned future.
    #[track_caller]
    #[allow(
        clippy::too_many_arguments,
        reason = "operation identity, selection, reuse, and initialization are separate contracts"
    )]
    pub fn cached_work<'a, T, F, Fut>(
        &'a self,
        kind: ExecutorKind,
        key: CacheKey,
        owner: CacheOperationOwner,
        selection: &'a AuthenticationSelection,
        fingerprint: NonEmptyString,
        reuse: WorkReuse,
        initialize: F,
    ) -> Result<ExecutionFuture<'a, Arc<T>>>
    where
        T: Send + Sync + 'static,
        F: FnOnce(Self) -> Fut + Send + 'a,
        Fut: Future<Output = Result<WorkOutput<T>>> + Send + 'a,
    {
        self.bind_cached_work(
            kind,
            key,
            owner,
            selection,
            fingerprint,
            reuse,
            move |child, _identity| initialize(child),
        )
    }

    /// Bind canonical work and its raw artifacts to the same admitted identity.
    ///
    /// The initializer receives an entry bound to its active work attempt. Load
    /// disk results through that entry and return [`cloud_terrastodon_cache::LoadedArtifact::decoded`]
    /// so original freshness and invalidation generations survive decoding.
    /// Finish observed artifacts even on decoding failure; publish them only
    /// after canonical acceptance. Publication remains limited to leaf results.
    ///
    /// `ExecuteEveryTime` and zero-validity requests cannot load or publish raw
    /// results for reuse, but may retain inspectable execution artifacts. Disabled
    /// retention suppresses raw disk access independently of decoded memory reuse.
    /// Credential infrastructure cannot enter either general work-cache API.
    /// Cancellation policy belongs to the caller: use [`Self::deferred`] for work
    /// whose effects, decoding, and artifacts must finish together. Caller-specific
    /// validators and projections run after this future, including on memory hits.
    #[track_caller]
    #[allow(
        clippy::too_many_arguments,
        reason = "identity, auth admission, reuse, raw retention, and initialization are separate contracts"
    )]
    pub fn cached_work_with_artifacts<'a, T, F, Fut>(
        &'a self,
        kind: ExecutorKind,
        key: CacheKey,
        owner: CacheOperationOwner,
        selection: &'a AuthenticationSelection,
        fingerprint: NonEmptyString,
        reuse: WorkReuse,
        retention: ArtifactRetention,
        initialize: F,
    ) -> Result<ExecutionFuture<'a, Arc<T>>>
    where
        T: Send + Sync + 'static,
        F: FnOnce(Self, ArtifactEntry) -> Fut + Send + 'a,
        Fut: Future<Output = Result<WorkOutput<T>>> + Send + 'a,
    {
        let artifact_validity = match reuse {
            WorkReuse::ReusableRead => key.valid_for,
            WorkReuse::ExecuteEveryTime => Duration::ZERO,
        };
        self.bind_cached_work(
            kind,
            key,
            owner,
            selection,
            fingerprint,
            reuse,
            move |child, identity| async move {
                let entry = child.cache.artifacts::<T>(
                    identity.kind,
                    CacheKey {
                        scope: identity.scope,
                        valid_for: artifact_validity,
                    },
                    owner,
                    identity.auth_partition,
                    identity.fingerprint,
                    retention,
                )?;
                let entry = match &child.mode {
                    ExecutionMode::Normal(Some(attempt)) => entry.for_work(attempt)?,
                    // Nonreusable work has no initializer token of its own. Its
                    // zero artifact validity prevents disk substitution too.
                    ExecutionMode::Normal(None) => entry,
                    ExecutionMode::NoWorkCacheOrAuthorizationReentry => {
                        eyre::bail!(
                            "credential infrastructure cannot retain general work artifacts"
                        );
                    }
                };
                initialize(child, entry).await
            },
        )
    }

    /// Execute independent work and return its caller-owned result.
    ///
    /// There is no cache lookup, join, or result retention, so output may borrow
    /// invocation data and need not be `Sync`. A reusable initializer cannot admit
    /// this operation: catching that error still prevents reuse of its ancestors.
    /// Credential infrastructure cannot enter this semantic work boundary.
    ///
    /// The callback receives a linked cancellable scope. Use [`Self::deferred`]
    /// around effectful work and its required completion; caller-specific borrowed
    /// or non-`Send` callbacks can run after awaiting the owned result.
    #[track_caller]
    pub fn uncached_work<'a, T, F, Fut>(
        &'a self,
        selection: &'a AuthenticationSelection,
        execute: F,
    ) -> Result<ExecutionFuture<'a, T>>
    where
        T: Send + 'a,
        F: FnOnce(Self) -> Fut + Send + 'a,
        Fut: Future<Output = Result<T>> + Send + 'a,
    {
        let partition = self.admit_uncached_work(selection)?;
        Ok(self.await_admitted_work(selection, partition, self.cancellable_scope(execute)))
    }

    /// Execute independent work with inspectable artifacts and an owned result.
    ///
    /// This retains the canonical operation/output ownership claim without
    /// requiring `Sync`, allocating an `Arc<T>`, or acquiring an initializer lock.
    /// The artifact key always has zero validity, regardless of requested lifetime:
    /// old disk results cannot replace execution and new outcomes cannot become
    /// reusable selections. Disabled retention suppresses all raw artifact I/O.
    ///
    /// The caller owns canonical decoding and artifact completion within the
    /// callback. Use [`Self::deferred`] when effects and their completion must stay
    /// together. The `'static` output bound identifies the artifact's canonical
    /// type; borrowed outputs without artifacts use [`Self::uncached_work`].
    #[track_caller]
    #[allow(
        clippy::too_many_arguments,
        reason = "artifact identity, authentication, retention, and execution are separate contracts"
    )]
    pub fn uncached_work_with_artifacts<'a, T, F, Fut>(
        &'a self,
        kind: ExecutorKind,
        key: CacheKey,
        owner: CacheOperationOwner,
        selection: &'a AuthenticationSelection,
        fingerprint: NonEmptyString,
        retention: ArtifactRetention,
        execute: F,
    ) -> Result<ExecutionFuture<'a, T>>
    where
        T: Send + 'static,
        F: FnOnce(Self, ArtifactEntry) -> Fut + Send + 'a,
        Fut: Future<Output = Result<T>> + Send + 'a,
    {
        let partition = self.admit_uncached_work(selection)?;
        let entry = self.cache.artifacts::<T>(
            kind,
            CacheKey {
                valid_for: Duration::ZERO,
                ..key
            },
            owner,
            partition.clone(),
            fingerprint,
            retention,
        )?;
        Ok(self.await_admitted_work(
            selection,
            partition,
            self.cancellable_scope(move |child| execute(child, entry)),
        ))
    }

    // Cached entry points share coordination and ancestry. The
    // exact admitted identity is carried into the initializer rather than rebuilt
    // by a transport adapter or resolved again after authentication changes.
    #[track_caller]
    #[allow(
        clippy::too_many_arguments,
        reason = "operation identity, selection, reuse, and initialization are separate contracts"
    )]
    fn bind_cached_work<'a, T, F, Fut>(
        &'a self,
        kind: ExecutorKind,
        key: CacheKey,
        owner: CacheOperationOwner,
        selection: &'a AuthenticationSelection,
        fingerprint: NonEmptyString,
        reuse: WorkReuse,
        initialize: F,
    ) -> Result<ExecutionFuture<'a, Arc<T>>>
    where
        T: Send + Sync + 'static,
        F: FnOnce(Self, CacheEntryIdentity) -> Fut + Send + 'a,
        Fut: Future<Output = Result<WorkOutput<T>>> + Send + 'a,
    {
        let (partition, parent) = self.admit_work(selection)?;
        let initialization_partition = partition.clone();
        let identity = CacheEntryIdentity {
            kind,
            scope: key.scope,
            auth_partition: partition.clone(),
            fingerprint,
        };
        let initialization_identity = identity.clone();
        let observation = self
            .observer
            .clone()
            .map(|observer| (observer, identity.clone()));
        let spec = self.cache.cached_work(
            identity,
            owner,
            reuse,
            key.valid_for,
            parent,
            &self.cancellation,
            move |attempt, scope| async move {
                ensure!(
                    self.admit(selection)? == initialization_partition,
                    "authentication identity changed before cached work initialization"
                );
                let mut child = self.with_scope(scope);
                child.mode = ExecutionMode::Normal(attempt);
                let output = initialize(child, initialization_identity).await?;
                ensure!(
                    self.admit(selection)? == initialization_partition,
                    "authentication identity changed during cached work initialization"
                );
                Ok(output)
            },
        )?;
        let spec = if let Some((observer, observed_identity)) = observation {
            spec.observe(move |status| {
                if let Err(e) = observer.send(ExecutionEvent::Cache {
                    identity: observed_identity.clone(),
                    status,
                }) {
                    warn!("failed to report execution event: {e:#}");
                }
            })
        } else {
            spec
        };
        Ok(self.await_admitted_work(selection, partition, spec))
    }

    // Keep admission and the canonical claim synchronous so track_caller can
    // still identify the operation's binding site, before a future is polled.
    fn admit_work(
        &self,
        selection: &AuthenticationSelection,
    ) -> Result<(AuthPartition, Option<&WorkAttempt>)> {
        let ExecutionMode::Normal(parent) = &self.mode else {
            eyre::bail!("credential infrastructure cannot enter semantic work");
        };
        let partition = self.record_semantic_result(self.admit(selection))?;
        Ok((partition, parent.as_ref()))
    }

    fn admit_uncached_work(&self, selection: &AuthenticationSelection) -> Result<AuthPartition> {
        let (partition, parent) = self.admit_work(selection)?;
        if parent.is_some() {
            // All active cache attempts are reusable initializers. Reject their
            // nonreusable child before callbacks, claims, or artifact I/O, and
            // poison the ancestor even when its caller catches this rejection.
            return self.record_semantic_result(Err(eyre::eyre!(
                "a reusable initializer cannot admit nonreusable semantic child work"
            )));
        }
        Ok(partition)
    }

    fn await_admitted_work<'a, T, F>(
        &'a self,
        selection: &'a AuthenticationSelection,
        partition: AuthPartition,
        work: F,
    ) -> ExecutionFuture<'a, T>
    where
        T: Send + 'a,
        F: IntoFuture<Output = Result<T>>,
        F::IntoFuture: Send + 'a,
    {
        let work = work.into_future();
        Box::pin(
            async move {
                let result = async {
                    ensure!(
                        self.admit(selection)? == partition,
                        "authentication identity changed after binding work"
                    );
                    let value = work.await?;
                    ensure!(
                        self.admit(selection)? == partition,
                        "authentication identity changed while awaiting work"
                    );
                    Ok(value)
                }
                .await;
                self.record_semantic_result(result)
            }
            .in_current_span(),
        )
    }

    /// Automatically cancellable child work sharing the parent's cancellation intent.
    #[track_caller]
    pub fn cancellable_scope<C, F, T, E>(
        &self,
        factory: C,
    ) -> impl Future<Output = Result<T, E>> + use<C, F, T, E>
    where
        C: FnOnce(Self) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = self.clone();
        self.cancellation
            .cancellable_scope(move |scope| factory(parent.with_scope(scope)))
    }

    /// Linked child work with its own cancellation signal.
    ///
    /// Cancelling the returned token affects this child and its descendants,
    /// while parent cancellation still propagates. The future owns its executor
    /// clone and preserves cache ancestry and authentication restrictions.
    #[track_caller]
    pub fn cancellable_child_scope<C, F, T, E>(
        &self,
        factory: C,
    ) -> (
        CancellationToken,
        impl Future<Output = Result<T, E>> + use<C, F, T, E>,
    )
    where
        C: FnOnce(Self) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = self.clone();
        self.cancellation
            .cancellable_child_scope(move |scope| factory(parent.with_scope(scope)))
    }

    /// Cooperative child work whose owner must run checkpoints and shutdown.
    #[track_caller]
    pub fn with_async_scope<C, F, T, E>(
        &self,
        factory: C,
    ) -> impl Future<Output = Result<T, E>> + use<C, F, T, E>
    where
        C: FnOnce(Self) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = self.clone();
        self.cancellation
            .with_async_scope(move |scope| factory(parent.with_scope(scope)))
    }

    /// Cooperative child work with an independent cancellation signal.
    ///
    /// Once admitted, the callback owns shutdown and draining required work.
    /// Its cancellation signal leaves the parent and sibling jobs unaffected.
    #[track_caller]
    pub fn with_async_child_scope<C, F, T, E>(
        &self,
        factory: C,
    ) -> (
        CancellationToken,
        impl Future<Output = Result<T, E>> + use<C, F, T, E>,
    )
    where
        C: FnOnce(Self) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = self.clone();
        self.cancellation
            .with_async_child_scope(move |scope| factory(parent.with_scope(scope)))
    }

    /// Protected new work; cancellation admission is owned by the supplied scope.
    #[track_caller]
    pub fn deferred<C, F, T, E>(
        &self,
        explanation: impl Into<String>,
        factory: C,
    ) -> impl Future<Output = Result<T, E>>
    where
        C: FnOnce(Self) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = self.clone();
        self.cancellation
            .deferred(explanation, move |scope| factory(parent.with_scope(scope)))
    }

    /// Cleanup already owed, admitted even after cancellation was requested.
    #[track_caller]
    pub fn finish_required<C, F, T, E>(
        &self,
        explanation: impl Into<String>,
        factory: C,
    ) -> impl Future<Output = Result<T, E>>
    where
        C: FnOnce(Self) -> F,
        F: IntoFuture<Output = Result<T, E>>,
        E: From<CancellationError>,
    {
        let parent = self.clone();
        self.cancellation
            .finish_required(explanation, move |scope| factory(parent.with_scope(scope)))
    }

    fn with_scope(&self, cancellation: CancellationScope) -> Self {
        Self {
            cancellation,
            ..self.clone()
        }
    }

    // Validation may fail before the low cache admits a child or after it has
    // accepted a result. Catching that error must not make a fallback reusable.
    fn record_semantic_result<T>(&self, result: Result<T>) -> Result<T> {
        if let Err(error) = result {
            if let ExecutionMode::Normal(Some(attempt)) = &self.mode
                && let Err(reuse_error) = attempt.prevent_reuse()
            {
                return Err(error.wrap_err(format!(
                    "could not prevent reuse after semantic work failed: {reuse_error:#}"
                )));
            }
            return Err(error);
        }
        result
    }
}
