#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::AuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::TestAuthenticationProvider;
use eyre::Result;
use eyre::ensure;
use std::cell::Cell;
use std::future::Future;
use std::future::IntoFuture;
use std::future::poll_fn;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;
use tokio::time::timeout;

const AUTHENTICATION: AuthenticationSelection =
    AuthenticationSelection::AzureDevOpsPersonalAccessToken;
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

#[derive(Default)]
struct CredentialLineageProbe {
    calls: AtomicUsize,
}

impl AuthenticationProvider for CredentialLineageProbe {
    fn admit(&self, _selection: &AuthenticationSelection) -> Result<AuthPartition> {
        AuthPartition::new("synthetic-credential-probe")
    }

    fn authorization<'a>(
        &'a self,
        selection: &'a AuthenticationSelection,
        resource: AzureCredentialResource,
        executor: &'a CloudTerrastodonExecutor,
    ) -> ExecutionFuture<'a, Authorization> {
        Box::pin(async move {
            // Bound recursion even if the executor accidentally loses lineage.
            ensure!(
                self.calls.fetch_add(1, Ordering::SeqCst) == 0,
                "synthetic provider was recursively entered"
            );
            executor
                .with_async_scope(|child| async move {
                    assert!(child.cache().is_err());
                    assert!(
                        child
                            .with_authentication(Arc::new(TestAuthenticationProvider::new(
                                "synthetic-test-session",
                                Authorization::personal_access_token(
                                    "synthetic-offline-credential"
                                )?
                            )?))
                            .is_err(),
                        "credential infrastructure cannot escape its restrictions by rebinding"
                    );
                    let recursive = child.authorization(selection, resource).await;
                    assert!(recursive.unwrap_err().to_string().contains("recursively"));
                    let cache_attempt = child.cached_work(
                        ExecutorKind::InProcess,
                        key("credential-infrastructure"),
                        CacheOperationOwner::of::<CredentialLineageProbe>(),
                        selection,
                        NonEmptyString::new("synthetic-credential-input")?,
                        WorkReuse::ReusableRead,
                        |_| async { Ok(output(1_u64)) },
                    );
                    assert!(
                        cache_attempt
                            .err()
                            .expect("credential work must be rejected before cache lookup")
                            .to_string()
                            .contains("credential infrastructure")
                    );
                    Authorization::personal_access_token("synthetic-offline-credential")
                })
                .await
        })
    }
}

struct BorrowedLabel<'a> {
    label: &'a str,
}

impl<'input> ExecutableRequest for BorrowedLabel<'input> {
    type Error = eyre::Report;
    type Output = (&'input str, CancellationScope);

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move { Ok((self.label, executor.cancellation().clone())) })
    }
}

struct OwnedCell;

impl ExecutableRequest for OwnedCell {
    type Error = eyre::Report;
    type Output = Cell<usize>;

    fn run<'a>(self, _executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async { Ok(Cell::new(7)) })
    }
}

struct NumberOperation;
struct CompositionOperation;

fn key(name: &str) -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", name]).unwrap(),
        valid_for: Duration::from_secs(3600),
    }
}

fn output<T>(value: T) -> WorkOutput<T> {
    WorkOutput::new(value, ArtifactObservation::now())
}

struct CachedNumber<'a> {
    text: &'a str,
    decodes: &'a AtomicUsize,
    release: Option<oneshot::Receiver<()>>,
}

impl<'a> CachedNumber<'a> {
    fn new(text: &'a str, decodes: &'a AtomicUsize) -> Self {
        Self {
            text,
            decodes,
            release: None,
        }
    }
}

impl ExecutableRequest for CachedNumber<'_> {
    type Error = eyre::Report;
    type Output = Arc<u64>;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key("numbers"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    NonEmptyString::new("synthetic-number-input")?,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        let _authorization = child
                            .authorization(&AUTHENTICATION, AzureCredentialResource::AzureDevOps)
                            .await?;
                        if let Some(release) = self.release {
                            release.await?;
                        }
                        self.decodes.fetch_add(1, Ordering::SeqCst);
                        Ok(output(self.text.parse::<u64>()?))
                    },
                )?
                .await
        })
    }
}

struct ComposedNumber<'a> {
    text: &'a str,
    decodes: &'a AtomicUsize,
    compositions: &'a AtomicUsize,
}

impl ExecutableRequest for ComposedNumber<'_> {
    type Error = eyre::Report;
    type Output = Arc<u64>;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key("compositions"),
                    CacheOperationOwner::of::<CompositionOperation>(),
                    &AUTHENTICATION,
                    NonEmptyString::new("synthetic-composition-input")?,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        self.compositions.fetch_add(1, Ordering::SeqCst);
                        let number = child
                            .with_async_scope(|scoped| async move {
                                CachedNumber::new(self.text, self.decodes)
                                    .execute(&scoped.clone())
                                    .await
                            })
                            .await?;
                        Ok(output(*number * 2))
                    },
                )?
                .await
        })
    }
}

// Poll once to establish that the future reached its wait before changing
// admission, cancelling its owner, or releasing another attempt.
async fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
    poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn both_execution_spellings_accept_borrowed_outputs_and_owned_non_sync_outputs() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::personal_access_token("synthetic-offline-credential")?,
                )?),
                scope,
            );
            let text = String::from("borrowed request data");
            let (first, first_scope) = executor.execute(BorrowedLabel { label: &text }).await?;
            let (second, second_scope) = BorrowedLabel { label: &text }.execute(&executor).await?;
            assert!(std::ptr::eq(first.as_ptr(), text.as_ptr()));
            assert!(std::ptr::eq(second.as_ptr(), text.as_ptr()));
            assert!(first_scope.bail_if_cancelled().is_err());
            assert!(second_scope.bail_if_cancelled().is_err());
            executor.cancellation().bail_if_cancelled()?;

            // This caller-owned observer deliberately borrows non-Send state.
            let observations = Rc::new(Cell::new(0));
            let observe = |value: Cell<usize>| {
                observations.set(observations.get() + 1);
                value.get() + observations.get()
            };
            assert_eq!(observe(executor.execute(OwnedCell).await?), 8);
            assert_eq!(observations.get(), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn scope_helpers_accept_borrowed_non_send_callbacks_and_close_their_child() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::personal_access_token("synthetic-offline-credential")?,
                )?),
                scope,
            );
            let observations = Rc::new(Cell::new(0));
            let label = String::from("scoped borrowed data");
            let callback = |child: CloudTerrastodonExecutor| {
                let observations = &observations;
                let label = &label;
                async move {
                    observations.set(observations.get() + 1);
                    let (borrowed, request_scope) = BorrowedLabel { label }.execute(&child).await?;
                    assert_eq!(borrowed, label);
                    assert!(request_scope.bail_if_cancelled().is_err());
                    child.cancellation().bail_if_cancelled()?;
                    Ok::<_, eyre::Report>(child.cancellation().clone())
                }
            };
            let cancellable = executor.cancellable_scope(callback).await?;
            let (_, child) = executor.cancellable_child_scope(callback);
            let independent = child.await?;
            let cooperative = executor.with_async_scope(callback).await?;
            let (_, child) = executor.with_async_child_scope(callback);
            let independent_cooperative = child.await?;
            let protected = executor
                .deferred("synthetic protected work", callback)
                .await?;
            let required = executor
                .finish_required("synthetic owed cleanup", callback)
                .await?;
            for closed in [
                cancellable,
                independent,
                cooperative,
                independent_cooperative,
                protected,
                required,
            ] {
                assert!(closed.bail_if_cancelled().is_err());
            }
            executor.cancellation().bail_if_cancelled()?;
            assert_eq!(observations.get(), 6);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn independent_jobs_share_resources_but_preserve_separate_cancellation() -> Result<()> {
    let root = CancellationToken::new();
    root.with_async_scope(|scope| async {
        let temporary = tempfile::tempdir()?;
        let authentication = Arc::new(TestAuthenticationProvider::new(
            "synthetic-test-session",
            Authorization::personal_access_token("synthetic-offline-credential")?,
        )?);
        let executor = CloudTerrastodonExecutor::new(
            WorkCache::new(temporary.path())?,
            authentication.clone(),
            scope,
        );
        let decodes = AtomicUsize::new(0);
        let cached = executor.execute(CachedNumber::new("42", &decodes)).await?;
        let (cancel_first, first) = executor.cancellable_child_scope(|child| async move {
            child.cancellation().cancelled().await;
            Ok::<_, eyre::Report>(())
        });
        tokio::pin!(first);
        assert_pending(first.as_mut()).await;
        cancel_first.request_cancel("cancel one synthetic job");
        assert!(first.await.is_err());
        executor.cancellation().bail_if_cancelled()?;
        let (_, second) = executor.with_async_child_scope(|child| {
            let decodes = &decodes;
            async move {
                let result = child.execute(CachedNumber::new("99", decodes)).await?;
                child
                    .cancellation()
                    .request_cancel("cooperative child shutdown");
                Ok::<_, eyre::Report>(result)
            }
        });
        assert!(Arc::ptr_eq(&cached, &second.await?));
        executor.cancellation().bail_if_cancelled()?;
        assert_eq!(decodes.load(Ordering::SeqCst), 1);
        assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 1);

        // Creating the future through a temporary clone proves it owns its
        // resources rather than borrowing the executor handle used to bind it.
        let (_, last) = executor
            .clone()
            .cancellable_child_scope(|child| async move {
                child.cancellation().cancelled().await;
                Ok::<_, eyre::Report>(())
            });
        tokio::pin!(last);
        assert_pending(last.as_mut()).await;
        root.request_cancel("cancel the synthetic parent");
        assert!(last.await.is_err());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn cloned_executors_share_one_decoded_result_without_reacquiring_credentials() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let authentication = Arc::new(TestAuthenticationProvider::new(
                "synthetic-test-session",
                Authorization::personal_access_token("synthetic-offline-credential")?,
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                authentication.clone(),
                scope,
            );
            let cloned = executor.clone();
            let decodes = AtomicUsize::new(0);
            let text = String::from("42");
            let (release, released) = oneshot::channel();
            let leader = executor
                .execute(CachedNumber {
                    text: &text,
                    decodes: &decodes,
                    release: Some(released),
                })
                .into_future();
            tokio::pin!(leader);
            assert_pending(leader.as_mut()).await;
            let waiter = CachedNumber::new("99", &decodes)
                .execute(&cloned)
                .into_future();
            tokio::pin!(waiter);
            assert_pending(waiter.as_mut()).await;
            release.send(()).unwrap();
            let first = timeout(DEADLOCK_GUARD, leader).await??;
            let second = timeout(DEADLOCK_GUARD, waiter).await??;
            let warm = cloned.execute(CachedNumber::new("100", &decodes)).await?;
            assert_eq!(*first, 42);
            assert!(Arc::ptr_eq(&first, &second));
            assert!(Arc::ptr_eq(&first, &warm));
            assert_eq!(decodes.load(Ordering::SeqCst), 1);
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn warm_results_and_previously_bound_work_still_require_current_admission() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let authentication = Arc::new(TestAuthenticationProvider::new(
                "synthetic-test-session",
                Authorization::personal_access_token("synthetic-offline-credential")?,
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                authentication.clone(),
                scope,
            );
            let decodes = AtomicUsize::new(0);
            let original = executor.execute(CachedNumber::new("42", &decodes)).await?;
            authentication
                .allow_authenticated
                .store(false, Ordering::SeqCst);
            let denied = executor.execute(CachedNumber::new("99", &decodes)).await;
            assert!(denied.unwrap_err().to_string().contains("synthetic policy"));

            authentication
                .allow_authenticated
                .store(true, Ordering::SeqCst);
            let bound = executor.cached_work(
                ExecutorKind::InProcess,
                key("numbers"),
                CacheOperationOwner::of::<NumberOperation>(),
                &AUTHENTICATION,
                NonEmptyString::new("synthetic-number-input")?,
                WorkReuse::ReusableRead,
                |_| async { Ok(output(999_u64)) },
            )?;
            authentication
                .allow_authenticated
                .store(false, Ordering::SeqCst);
            assert!(
                bound
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("synthetic policy")
            );
            assert_eq!(*original, 42);
            assert_eq!(decodes.load(Ordering::SeqCst), 1);
            assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn nested_execution_and_scope_helpers_preserve_dependencies_on_warm_child_results()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::personal_access_token("synthetic-offline-credential")?,
                )?),
                scope,
            );
            let decodes = AtomicUsize::new(0);
            let compositions = AtomicUsize::new(0);
            executor.execute(CachedNumber::new("21", &decodes)).await?;
            let first = executor
                .execute(ComposedNumber {
                    text: "99",
                    decodes: &decodes,
                    compositions: &compositions,
                })
                .await?;
            assert_eq!(*first, 42);
            assert_eq!(decodes.load(Ordering::SeqCst), 1);

            // The composed key is in a separate branch: this requires the
            // executor's inherited work lineage, not prefix invalidation alone.
            executor
                .cache()?
                .invalidate(&key("numbers").scope, executor.cancellation())
                .await?;
            let refreshed = executor
                .execute(ComposedNumber {
                    text: "30",
                    decodes: &decodes,
                    compositions: &compositions,
                })
                .await?;
            assert_eq!(*first, 42);
            assert_eq!(*refreshed, 60);
            assert!(!Arc::ptr_eq(&first, &refreshed));
            assert_eq!(decodes.load(Ordering::SeqCst), 2);
            assert_eq!(compositions.load(Ordering::SeqCst), 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn authentication_rebinding_preserves_partitioned_reuse_dependencies_and_cancellation()
-> Result<()> {
    let token = CancellationToken::new();
    token
        .with_async_scope(|scope| async {
            let original = Arc::new(TestAuthenticationProvider::new(
                "synthetic-test-session",
                Authorization::personal_access_token("synthetic-offline-credential")?,
            )?);
            let selected = Arc::new(TestAuthenticationProvider {
                partition_generation: AtomicUsize::new(1),
                ..TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::personal_access_token("synthetic-offline-credential")?,
                )?
            });
            let executor =
                CloudTerrastodonExecutor::new(WorkCache::temporary()?, original.clone(), scope);
            let rebound = executor.with_authentication(selected.clone())?;
            let decodes = AtomicUsize::new(0);
            let original_value = executor.execute(CachedNumber::new("21", &decodes)).await?;
            let selected_value = rebound.execute(CachedNumber::new("30", &decodes)).await?;
            assert_eq!(*original_value, 21);
            assert_eq!(*selected_value, 30);
            assert!(!Arc::ptr_eq(&original_value, &selected_value));
            assert!(Arc::ptr_eq(
                &original_value,
                &executor.execute(CachedNumber::new("99", &decodes)).await?
            ));
            let same_selection = executor.with_authentication(selected.clone())?;
            assert!(Arc::ptr_eq(
                &selected_value,
                &same_selection
                    .execute(CachedNumber::new("99", &decodes))
                    .await?
            ));

            // A provider change inside a reusable parent must preserve the child's
            // dependency edge, even when that child is already cached in another
            // authentication partition and a different logical key branch.
            let compositions = AtomicUsize::new(0);
            for (text, expected) in [("99", 30), ("40", 40)] {
                let selected = selected.clone();
                let decodes = &decodes;
                let compositions = &compositions;
                let value = executor
                    .cached_work(
                        ExecutorKind::InProcess,
                        key("rebound-composition"),
                        CacheOperationOwner::of::<CompositionOperation>(),
                        &AUTHENTICATION,
                        NonEmptyString::new("synthetic-rebound-composition")?,
                        WorkReuse::ReusableRead,
                        move |child| async move {
                            compositions.fetch_add(1, Ordering::SeqCst);
                            let child = child.with_authentication(selected)?;
                            let value = child.execute(CachedNumber::new(text, decodes)).await?;
                            Ok(output(*value))
                        },
                    )?
                    .await?;
                assert_eq!(*value, expected);
                if expected == 30 {
                    executor
                        .cache()?
                        .invalidate(&key("numbers").scope, executor.cancellation())
                        .await?;
                }
            }
            assert_eq!(compositions.load(Ordering::SeqCst), 2);
            assert_eq!(decodes.load(Ordering::SeqCst), 3);
            assert_eq!(original.authorizations.load(Ordering::SeqCst), 1);
            assert_eq!(selected.authorizations.load(Ordering::SeqCst), 2);

            token.request_cancel("end the lifetime shared by both provider bindings");
            assert!(executor.cancellation().bail_if_cancelled().is_err());
            assert!(rebound.cancellation().bail_if_cancelled().is_err());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancelling_the_owner_allows_an_independently_owned_waiter_to_retry() -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let cache = WorkCache::new(temporary.path())?;
    let authentication = Arc::new(TestAuthenticationProvider::new(
        "synthetic-test-session",
        Authorization::personal_access_token("synthetic-offline-credential")?,
    )?);
    let decodes = AtomicUsize::new(0);
    let owner_token = CancellationToken::new();
    let waiter_token = CancellationToken::new();
    let (_release, released) = oneshot::channel();
    let owner = owner_token.with_async_scope(|scope| async {
        let executor = CloudTerrastodonExecutor::new(cache.clone(), authentication.clone(), scope);
        executor
            .execute(CachedNumber {
                text: "42",
                decodes: &decodes,
                release: Some(released),
            })
            .await
    });
    tokio::pin!(owner);
    assert_pending(owner.as_mut()).await;
    let waiter = waiter_token.with_async_scope(|scope| async {
        let executor = CloudTerrastodonExecutor::new(cache.clone(), authentication.clone(), scope);
        executor.execute(CachedNumber::new("77", &decodes)).await
    });
    tokio::pin!(waiter);
    assert_pending(waiter.as_mut()).await;

    owner_token.request_cancel("cancel only the synthetic initiating caller");
    assert!(timeout(DEADLOCK_GUARD, owner).await?.is_err());
    let result = timeout(DEADLOCK_GUARD, waiter).await??;
    assert_eq!(*result, 77);
    assert!(!waiter_token.is_cancelled());
    assert_eq!(decodes.load(Ordering::SeqCst), 1);
    assert_eq!(authentication.authorizations.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn protected_descendants_finish_and_owed_cleanup_can_start_after_cancellation() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::personal_access_token("synthetic-offline-credential")?,
                )?),
                scope,
            );
            let (release, released) = oneshot::channel();
            let operation = executor.cancellable_scope(|child| async move {
                child
                    .deferred(
                        "finish synthetic protected operation",
                        |protected| async move {
                            released.await?;
                            let (value, _) = BorrowedLabel { label: "completed" }
                                .execute(&protected)
                                .await?;
                            Ok::<_, eyre::Report>((value, protected.cancellation().clone()))
                        },
                    )
                    .await
            });
            tokio::pin!(operation);
            assert_pending(operation.as_mut()).await;
            executor.cancellation().request_cancel("synthetic shutdown");
            assert_pending(operation.as_mut()).await;
            release.send(()).unwrap();
            let (value, closed) = timeout(DEADLOCK_GUARD, operation).await??;
            assert_eq!(value, "completed");
            assert!(closed.bail_if_cancelled().is_err());

            let optional_started = Cell::new(false);
            let optional = executor
                .deferred("new optional work", |_| async {
                    optional_started.set(true);
                    Ok::<_, eyre::Report>(())
                })
                .await;
            assert!(optional.is_err());
            assert!(!optional_started.get());

            let cleanup = Rc::new(Cell::new(false));
            let closed = executor
                .finish_required("complete synthetic owed cleanup", |required| {
                    let cleanup = &cleanup;
                    async move {
                        assert!(required.cancellation().is_cancelled());
                        let value = required.execute(OwnedCell).await?;
                        cleanup.set(value.get() == 7);
                        Ok::<_, eyre::Report>(required.cancellation().clone())
                    }
                })
                .await?;
            assert!(cleanup.get());
            assert!(closed.bail_if_cancelled().is_err());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn provider_scope_helpers_cannot_enter_semantic_cache_or_recursive_authorization()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let authentication = Arc::new(CredentialLineageProbe::default());
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                authentication.clone(),
                scope,
            );
            executor
                .authorization(&AUTHENTICATION, AzureCredentialResource::AzureDevOps)
                .await?;
            assert_eq!(authentication.calls.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

async fn assert_waiter_revocation_is_enforced(cancel_owner: bool) -> Result<()> {
    // A provider may reject the selection outright or admit it under a new
    // identity. Neither permits returning work bound to the previous partition.
    for change_partition in [false, true] {
        let temporary = tempfile::tempdir()?;
        let cache = WorkCache::new(temporary.path())?;
        let owner_authentication = Arc::new(TestAuthenticationProvider::new(
            "synthetic-test-session",
            Authorization::personal_access_token("synthetic-offline-credential")?,
        )?);
        let waiter_authentication = Arc::new(TestAuthenticationProvider::new(
            "synthetic-test-session",
            Authorization::personal_access_token("synthetic-offline-credential")?,
        )?);
        let owner_token = CancellationToken::new();
        let waiter_token = CancellationToken::new();
        let owner_initializations = Arc::new(AtomicUsize::new(0));
        let waiter_initializations = AtomicUsize::new(0);
        let owner_starts = owner_initializations.clone();
        let (release, released) = oneshot::channel();

        let mut owner = Box::pin(owner_token.cancellable_scope(|scope| async {
            let executor =
                CloudTerrastodonExecutor::new(cache.clone(), owner_authentication.clone(), scope);
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key("pending-admission"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    NonEmptyString::new("synthetic-pending-input")?,
                    WorkReuse::ReusableRead,
                    move |_| async move {
                        owner_starts.fetch_add(1, Ordering::SeqCst);
                        released.await?;
                        Ok(output(42_u64))
                    },
                )?
                .await
        }));
        assert_pending(owner.as_mut()).await;
        assert_eq!(owner_initializations.load(Ordering::SeqCst), 1);

        let waiter = waiter_token.with_async_scope(|scope| async {
            let executor =
                CloudTerrastodonExecutor::new(cache.clone(), waiter_authentication.clone(), scope);
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    key("pending-admission"),
                    CacheOperationOwner::of::<NumberOperation>(),
                    &AUTHENTICATION,
                    NonEmptyString::new("synthetic-pending-input")?,
                    WorkReuse::ReusableRead,
                    |_| async {
                        waiter_initializations.fetch_add(1, Ordering::SeqCst);
                        Ok(output(99_u64))
                    },
                )?
                .await
        });
        tokio::pin!(waiter);
        assert_pending(waiter.as_mut()).await;

        if change_partition {
            waiter_authentication
                .partition_generation
                .store(1, Ordering::SeqCst);
        } else {
            waiter_authentication
                .allow_authenticated
                .store(false, Ordering::SeqCst);
        }
        if cancel_owner {
            owner_token.request_cancel("synthetic owner abandoned its attempt");
            assert!(timeout(DEADLOCK_GUARD, owner.as_mut()).await?.is_err());
        } else {
            release.send(()).unwrap();
            assert_eq!(*timeout(DEADLOCK_GUARD, owner.as_mut()).await??, 42);
        }
        // The cancellation wrapper retains its inner attempt until dropped,
        // even after reporting cancellation. Release it before retrying work.
        drop(owner);

        let error = timeout(DEADLOCK_GUARD, waiter).await?.unwrap_err();
        let expected_reason = if change_partition {
            "authentication identity changed"
        } else {
            "synthetic policy denies authenticated selection"
        };
        assert!(error.to_string().contains(expected_reason), "{error:?}");
        assert_eq!(waiter_initializations.load(Ordering::SeqCst), 0);
        assert!(!waiter_token.is_cancelled());
    }
    Ok(())
}

#[tokio::test]
async fn revoked_waiters_cannot_receive_results_from_an_admitted_owner() -> Result<()> {
    assert_waiter_revocation_is_enforced(false).await
}

#[tokio::test]
async fn revoked_waiters_cannot_initialize_after_owner_cancellation() -> Result<()> {
    assert_waiter_revocation_is_enforced(true).await
}

#[tokio::test]
async fn catching_child_admission_failure_cannot_publish_a_reusable_parent_fallback() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let authentication = Arc::new(TestAuthenticationProvider::new(
                "synthetic-test-session",
                Authorization::personal_access_token("synthetic-offline-credential")?,
            )?);
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::new(temporary.path())?,
                authentication.clone(),
                scope,
            );
            let compositions = AtomicUsize::new(0);
            let decodes = AtomicUsize::new(0);
            for denied in [true, false] {
                authentication
                    .allow_authenticated
                    .store(!denied, Ordering::SeqCst);
                // Anonymous admission permits the composition itself. Its
                // authenticated child is rejected before cache binding while
                // denied, so the executor must mark the parent's lineage.
                let result = executor
                    .cached_work(
                        ExecutorKind::InProcess,
                        key("fallback-composition"),
                        CacheOperationOwner::of::<CompositionOperation>(),
                        &AuthenticationSelection::Anonymous,
                        NonEmptyString::new("synthetic-fallback-input")?,
                        WorkReuse::ReusableRead,
                        |child| {
                            let compositions = &compositions;
                            let decodes = &decodes;
                            async move {
                                compositions.fetch_add(1, Ordering::SeqCst);
                                let value =
                                    match child.execute(CachedNumber::new("21", decodes)).await {
                                        Ok(value) => *value,
                                        Err(_) => 0,
                                    };
                                Ok(output(value))
                            }
                        },
                    )?
                    .await?;
                assert_eq!(*result, if denied { 0 } else { 21 });
            }
            assert_eq!(compositions.load(Ordering::SeqCst), 2);
            assert_eq!(decodes.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn admission_changes_during_initialization_prevent_publication_under_the_previous_identity()
-> Result<()> {
    for change_partition in [false, true] {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let temporary = tempfile::tempdir()?;
                let authentication = Arc::new(TestAuthenticationProvider::new(
                    "synthetic-test-session",
                    Authorization::personal_access_token("synthetic-offline-credential")?,
                )?);
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::new(temporary.path())?,
                    authentication.clone(),
                    scope,
                );
                let initializations = AtomicUsize::new(0);
                let first = executor
                    .cached_work(
                        ExecutorKind::InProcess,
                        key("publication-admission"),
                        CacheOperationOwner::of::<NumberOperation>(),
                        &AUTHENTICATION,
                        NonEmptyString::new("synthetic-publication-input")?,
                        WorkReuse::ReusableRead,
                        |_| async {
                            initializations.fetch_add(1, Ordering::SeqCst);
                            if change_partition {
                                authentication
                                    .partition_generation
                                    .store(1, Ordering::SeqCst);
                            } else {
                                authentication
                                    .allow_authenticated
                                    .store(false, Ordering::SeqCst);
                            }
                            Ok(output(11_u64))
                        },
                    )?
                    .await;
                assert!(first.is_err());

                authentication
                    .allow_authenticated
                    .store(true, Ordering::SeqCst);
                authentication
                    .partition_generation
                    .store(0, Ordering::SeqCst);
                let second = executor
                    .cached_work(
                        ExecutorKind::InProcess,
                        key("publication-admission"),
                        CacheOperationOwner::of::<NumberOperation>(),
                        &AUTHENTICATION,
                        NonEmptyString::new("synthetic-publication-input")?,
                        WorkReuse::ReusableRead,
                        |_| async {
                            initializations.fetch_add(1, Ordering::SeqCst);
                            Ok(output(22_u64))
                        },
                    )?
                    .await?;
                // A receipt-only admission check would reject the first caller
                // after publishing 11, allowing this call to reuse that value.
                assert_eq!(*second, 22);
                assert_eq!(initializations.load(Ordering::SeqCst), 2);
                Ok::<_, eyre::Report>(())
            })
            .await?;
    }
    Ok(())
}
