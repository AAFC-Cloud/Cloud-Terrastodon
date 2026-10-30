#![cfg(test)]

use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::CacheEntryIdentity;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkAttempt;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use std::future::Future;
use std::future::IntoFuture;
use std::future::poll_fn;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::Barrier;
use tokio::sync::oneshot;
use tokio::time::timeout;

const VALID_FOR: Duration = Duration::from_secs(3600);
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

struct SyntheticRead;
struct ConflictingRead;

fn identity(name: &str) -> CacheEntryIdentity {
    CacheEntryIdentity {
        kind: ExecutorKind::InProcess,
        scope: CacheScope::try_from_components(["synthetic", name]).unwrap(),
        auth_partition: AuthPartition::new("test-session").unwrap(),
        fingerprint: NonEmptyString::new("test-inputs").unwrap(),
    }
}

fn output<T>(value: T) -> WorkOutput<T> {
    WorkOutput::new(value, ArtifactObservation::now())
}

async fn read(
    cache: &WorkCache,
    name: &str,
    scope: &CancellationScope,
    value: u64,
) -> eyre::Result<Arc<u64>> {
    cache
        .cached_work(
            identity(name),
            CacheOperationOwner::of::<SyntheticRead>(),
            WorkReuse::ReusableRead,
            VALID_FOR,
            None,
            scope,
            move |_, _| async move { Ok(output(value)) },
        )?
        .await
}

// Poll once to prove the caller has actually reached a pending wait. This makes
// cancellation, eviction, and release ordering independent of scheduler timing.
async fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
    poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn concurrent_and_warm_reads_reuse_one_execution_decode_and_arc() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let executions = Arc::new(AtomicUsize::new(0));
            let decodes = Arc::new(AtomicUsize::new(0));
            let (started, ready) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let leader = {
                let cache = cache.clone();
                let executions = executions.clone();
                let decodes = decodes.clone();
                tokio::spawn(
                    CancellationToken::new().with_async_scope(move |child| async move {
                        cache
                            .cached_work(
                                identity("shared"),
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ReusableRead,
                                VALID_FOR,
                                None,
                                &child,
                                move |attempt, _| async move {
                                    assert!(attempt.is_some());
                                    executions.fetch_add(1, Ordering::SeqCst);
                                    started.send(()).unwrap();
                                    released.await?;
                                    let observation = ArtifactObservation::now();
                                    decodes.fetch_add(1, Ordering::SeqCst);
                                    Ok(WorkOutput::new(42_u64, observation))
                                },
                            )?
                            .await
                    }),
                )
            };
            timeout(DEADLOCK_GUARD, ready).await??;
            let follower = read(&cache, "shared", &scope, 99);
            tokio::pin!(follower);
            assert_pending(follower.as_mut()).await;
            release.send(()).unwrap();
            let first = timeout(DEADLOCK_GUARD, leader).await???;
            let second = timeout(DEADLOCK_GUARD, follower).await??;
            let warm = read(&cache, "shared", &scope, 100).await?;
            assert_eq!(*first, 42);
            assert!(Arc::ptr_eq(&first, &second));
            assert!(Arc::ptr_eq(&first, &warm));
            assert_eq!(executions.load(Ordering::SeqCst), 1);
            assert_eq!(decodes.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

async fn barrier_read(
    cache: &WorkCache,
    key: CacheEntryIdentity,
    barrier: Arc<Barrier>,
) -> eyre::Result<Arc<u64>> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            cache
                .cached_work(
                    key,
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    move |_, _| async move {
                        barrier.wait().await;
                        Ok(output(7))
                    },
                )?
                .await
        })
        .await
}

#[tokio::test]
async fn distinct_identities_can_initialize_concurrently() -> eyre::Result<()> {
    let temporary = tempfile::tempdir()?;
    let base = identity("independent");
    let variants = [
        identity("different-scope"),
        CacheEntryIdentity {
            fingerprint: NonEmptyString::new("different-inputs")?,
            ..base.clone()
        },
        CacheEntryIdentity {
            auth_partition: AuthPartition::new("different-session")?,
            ..base.clone()
        },
        CacheEntryIdentity {
            kind: ExecutorKind::Command,
            ..base.clone()
        },
    ];
    for variant in variants {
        let cache = WorkCache::new(temporary.path())?;
        let barrier = Arc::new(Barrier::new(2));
        let (first, second) = timeout(DEADLOCK_GUARD, async {
            tokio::join!(
                barrier_read(&cache, base.clone(), barrier.clone()),
                barrier_read(&cache, variant, barrier),
            )
        })
        .await?;
        assert!(!Arc::ptr_eq(&first?, &second?));
    }
    Ok(())
}

fn assert_conflicts(cache: &WorkCache, scope: &CancellationScope) {
    let changed_fingerprint = CacheEntryIdentity {
        fingerprint: NonEmptyString::new("cannot-hide-contract-conflict").unwrap(),
        ..identity("claimed")
    };
    assert!(
        cache
            .cached_work(
                changed_fingerprint.clone(),
                CacheOperationOwner::of::<ConflictingRead>(),
                WorkReuse::ReusableRead,
                VALID_FOR,
                None,
                scope,
                |_, _| async { Ok(output(1_u64)) },
            )
            .is_err()
    );
    assert!(
        cache
            .cached_work(
                changed_fingerprint,
                CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead,
                VALID_FOR,
                None,
                scope,
                |_, _| async { Ok(output("conflicting output".to_owned())) },
            )
            .is_err()
    );
}

#[tokio::test]
async fn owner_and_output_conflicts_fail_during_initialization_and_after_reuse() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let (release, released) = oneshot::channel();
            let leader = cache
                .cached_work(
                    identity("claimed"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |_, _| async move {
                        released.await?;
                        Ok(output(1_u64))
                    },
                )?
                .into_future();
            tokio::pin!(leader);
            assert_pending(leader.as_mut()).await;
            assert_conflicts(&cache, &scope);
            release.send(()).unwrap();
            leader.await?;
            assert_conflicts(&cache, &scope);
            cache.release_memory()?;
            assert_conflicts(&cache, &scope);
            Ok(())
        })
        .await
}

#[derive(Clone, Copy)]
enum LeaderExit {
    Error,
    Cancel,
    Drop,
}

async fn retry_after_failed_leader(exit: LeaderExit) -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let leader_token = CancellationToken::new();
            let (started, ready) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let leader = {
                let cache = cache.clone();
                tokio::spawn(leader_token.with_async_scope(move |child| async move {
                    cache
                        .cached_work::<u64, _, _>(
                            identity("retry"),
                            CacheOperationOwner::of::<SyntheticRead>(),
                            WorkReuse::ReusableRead,
                            VALID_FOR,
                            None,
                            &child,
                            move |_, _| async move {
                                started.send(()).unwrap();
                                released.await?;
                                Err(eyre::eyre!("synthetic initializer failed"))
                            },
                        )?
                        .await
                }))
            };
            timeout(DEADLOCK_GUARD, ready).await??;
            let follower = read(&cache, "retry", &scope, 91);
            tokio::pin!(follower);
            assert_pending(follower.as_mut()).await;
            match exit {
                LeaderExit::Error => {
                    release.send(()).unwrap();
                    assert!(timeout(DEADLOCK_GUARD, leader).await??.is_err());
                }
                LeaderExit::Cancel => {
                    leader_token.request_cancel("cancel synthetic leader only");
                    assert!(timeout(DEADLOCK_GUARD, leader).await??.is_err());
                    drop(release);
                }
                LeaderExit::Drop => {
                    leader.abort();
                    assert!(
                        timeout(DEADLOCK_GUARD, leader)
                            .await?
                            .unwrap_err()
                            .is_cancelled()
                    );
                    drop(release);
                }
            }
            let retried = timeout(DEADLOCK_GUARD, follower).await??;
            assert_eq!(*retried, 91);
            let warm = read(&cache, "retry", &scope, 92).await?;
            assert!(Arc::ptr_eq(&retried, &warm));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn initializer_error_is_not_cached_and_a_waiter_retries() -> eyre::Result<()> {
    retry_after_failed_leader(LeaderExit::Error).await
}

#[tokio::test]
async fn cancelled_leader_does_not_cancel_an_independent_waiters_retry() -> eyre::Result<()> {
    retry_after_failed_leader(LeaderExit::Cancel).await
}

#[tokio::test]
async fn dropped_leader_releases_its_slot_for_a_waiters_retry() -> eyre::Result<()> {
    retry_after_failed_leader(LeaderExit::Drop).await
}

#[tokio::test]
async fn cancelling_a_waiter_leaves_the_active_initializer_running() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let (release, released) = oneshot::channel();
            let leader = cache
                .cached_work(
                    identity("waiter-cancel"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |_, _| async move {
                        released.await?;
                        Ok(output(6_u64))
                    },
                )?
                .into_future();
            tokio::pin!(leader);
            assert_pending(leader.as_mut()).await;
            let waiter_token = CancellationToken::new();
            let waiter = waiter_token
                .with_async_scope(|child| {
                    let cache = &cache;
                    async move { read(cache, "waiter-cancel", &child, 9).await }
                })
                .into_future();
            tokio::pin!(waiter);
            assert_pending(waiter.as_mut()).await;
            waiter_token.request_cancel("stop only the waiting caller");
            assert!(timeout(DEADLOCK_GUARD, waiter).await?.is_err());
            assert_pending(leader.as_mut()).await;
            release.send(()).unwrap();
            let value = leader.await?;
            assert_eq!(*value, 6);
            assert!(Arc::ptr_eq(
                &value,
                &read(&cache, "waiter-cancel", &scope, 9).await?
            ));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn common_root_cancellation_stops_the_initializer_and_waiter() -> eyre::Result<()> {
    let root = CancellationToken::new();
    root.with_async_scope(|scope| {
        let root = &root;
        async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let leader = cache
                .cached_work::<u64, _, _>(
                    identity("root-cancel"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |_, _| std::future::pending::<eyre::Result<WorkOutput<u64>>>(),
                )?
                .into_future();
            let waiter = read(&cache, "root-cancel", &scope, 10);
            tokio::pin!(leader, waiter);
            assert_pending(leader.as_mut()).await;
            assert_pending(waiter.as_mut()).await;
            root.request_cancel("stop the whole synthetic operation");
            let (first, second) =
                timeout(DEADLOCK_GUARD, async { tokio::join!(leader, waiter) }).await?;
            assert!(first.is_err());
            assert!(second.is_err());
            Ok(())
        }
    })
    .await
}

#[tokio::test]
async fn direct_recursion_returns_an_error_instead_of_waiting_for_itself() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let result = cache
                .cached_work::<u64, _, _>(
                    identity("recursive"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        async move {
                            let nested = cache
                                .cached_work(
                                    identity("recursive"),
                                    CacheOperationOwner::of::<SyntheticRead>(),
                                    WorkReuse::ReusableRead,
                                    VALID_FOR,
                                    parent.as_ref(),
                                    &child,
                                    |_, _| async { Ok(output(1_u64)) },
                                )?
                                .await?;
                            Ok(output(*nested))
                        }
                    },
                )?
                .into_future();
            assert!(timeout(DEADLOCK_GUARD, result).await?.is_err());
            assert_eq!(*read(&cache, "recursive", &scope, 5).await?, 5);
            Ok(())
        })
        .await
}

async fn cyclic_read(
    cache: &WorkCache,
    own_key: &'static str,
    other_key: &'static str,
    barrier: Arc<Barrier>,
) -> eyre::Result<Arc<u64>> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            cache
                .cached_work(
                    identity(own_key),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| async move {
                        barrier.wait().await;
                        let other = cache
                            .cached_work(
                                identity(other_key),
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ReusableRead,
                                VALID_FOR,
                                parent.as_ref(),
                                &child,
                                |_, _| async { Ok(output(8_u64)) },
                            )?
                            .await?;
                        Ok(output(*other))
                    },
                )?
                .await
        })
        .await
}

#[tokio::test]
async fn cross_task_cycle_is_rejected_and_leaves_the_cache_usable() -> eyre::Result<()> {
    let temporary = tempfile::tempdir()?;
    let cache = WorkCache::new(temporary.path())?;
    let barrier = Arc::new(Barrier::new(2));
    let first = {
        let cache = cache.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move { cyclic_read(&cache, "cycle-a", "cycle-b", barrier).await })
    };
    let second = {
        let cache = cache.clone();
        tokio::spawn(async move { cyclic_read(&cache, "cycle-b", "cycle-a", barrier).await })
    };
    let (first, second) = timeout(DEADLOCK_GUARD, async { tokio::join!(first, second) }).await?;
    let first = first?;
    let second = second?;
    assert!(first.is_err() || second.is_err());
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            read(&cache, "cycle-a", &scope, 1).await?;
            read(&cache, "cycle-b", &scope, 2).await?;
            Ok(())
        })
        .await
}

#[tokio::test]
async fn zero_validity_executes_each_call_without_replacing_a_reusable_result() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let warm = read(&cache, "zero-validity", &scope, 3).await?;
            let executions = AtomicUsize::new(0);
            for value in [4_u64, 5] {
                let fresh = cache
                    .cached_work(
                        identity("zero-validity"),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        WorkReuse::ReusableRead,
                        Duration::ZERO,
                        None,
                        &scope,
                        |parent, _| {
                            let executions = &executions;
                            async move {
                                assert!(parent.is_none());
                                executions.fetch_add(1, Ordering::SeqCst);
                                Ok(output(value))
                            }
                        },
                    )?
                    .await?;
                assert_eq!(*fresh, value);
                assert!(!Arc::ptr_eq(&warm, &fresh));
            }
            assert_eq!(executions.load(Ordering::SeqCst), 2);
            assert!(Arc::ptr_eq(
                &warm,
                &read(&cache, "zero-validity", &scope, 6).await?
            ));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn zero_validity_child_is_rejected_before_execution_and_prevents_parent_reuse()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let result = cache
                .cached_work::<u64, _, _>(
                    identity("ancestor"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        async move {
                            let intermediary = cache.cached_work::<u64, _, _>(
                                identity("uncached-intermediary"),
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ReusableRead,
                                Duration::ZERO,
                                parent.as_ref(),
                                &child,
                                |_, _| async { panic!("an incompatible child must not execute") },
                            );
                            assert!(intermediary.is_err());
                            Ok(output(1_u64))
                        }
                    },
                )?
                .into_future();
            assert_eq!(*timeout(DEADLOCK_GUARD, result).await??, 1);
            assert_eq!(*read(&cache, "ancestor", &scope, 2).await?, 2);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn releasing_memory_during_a_fill_preserves_the_live_gate() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let (release, released) = oneshot::channel();
            let leader = cache
                .cached_work(
                    identity("eviction"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |_, _| async move {
                        released.await?;
                        Ok(output(12_u64))
                    },
                )?
                .into_future();
            let first_waiter = read(&cache, "eviction", &scope, 13);
            tokio::pin!(leader, first_waiter);
            assert_pending(leader.as_mut()).await;
            assert_pending(first_waiter.as_mut()).await;
            cache.release_memory()?;
            let second_waiter = read(&cache, "eviction", &scope, 14);
            tokio::pin!(second_waiter);
            assert_pending(second_waiter.as_mut()).await;
            release.send(()).unwrap();
            let (leader, first, second) = timeout(DEADLOCK_GUARD, async {
                tokio::join!(leader, first_waiter, second_waiter)
            })
            .await?;
            let leader = leader?;
            assert!(Arc::ptr_eq(&leader, &first?));
            assert!(Arc::ptr_eq(&leader, &second?));
            cache.release_memory()?;
            let refilled = read(&cache, "eviction", &scope, 15).await?;
            assert_eq!(*refilled, 15);
            assert!(!Arc::ptr_eq(&leader, &refilled));
            assert_eq!(*leader, 12);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn finished_attempt_handles_cannot_be_reused_as_active_parents() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let (retained, attempt) = oneshot::channel::<WorkAttempt>();
            cache
                .cached_work(
                    identity("finished-parent"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, _| async move {
                        retained.send(parent.unwrap()).unwrap();
                        Ok(output(1_u64))
                    },
                )?
                .await?;
            let stale = attempt.await?;
            let result = cache.cached_work(
                identity("new-child"),
                CacheOperationOwner::of::<SyntheticRead>(),
                WorkReuse::ReusableRead,
                VALID_FOR,
                Some(&stale),
                &scope,
                |_, _| async { Ok(output(2_u64)) },
            );
            if let Ok(spec) = result {
                assert!(spec.await.is_err());
            }
            Ok(())
        })
        .await
}

#[tokio::test]
async fn an_active_parent_cannot_be_transferred_to_an_isolated_cache_handle() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let isolated = WorkCache::new(temporary.path())?;
            cache
                .cached_work(
                    identity("parent-owner"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| async move {
                        let result = isolated.cached_work(
                            identity("foreign-child"),
                            CacheOperationOwner::of::<SyntheticRead>(),
                            WorkReuse::ReusableRead,
                            VALID_FOR,
                            parent.as_ref(),
                            &child,
                            |_, _| async { Ok(output(2_u64)) },
                        );
                        if let Ok(spec) = result {
                            assert!(spec.await.is_err());
                        }
                        Ok(output(1_u64))
                    },
                )?
                .await?;
            assert_eq!(*read(&cache, "parent-owner", &scope, 3).await?, 3);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn wrapper_and_transport_can_nest_under_the_same_scope_in_separate_namespaces()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let wrapper = cache
                .cached_work(
                    identity("wrapper-transport"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        async move {
                            let transport = cache
                                .cached_work(
                                    CacheEntryIdentity {
                                        kind: ExecutorKind::Rest,
                                        ..identity("wrapper-transport")
                                    },
                                    CacheOperationOwner::of::<ConflictingRead>(),
                                    WorkReuse::ReusableRead,
                                    VALID_FOR,
                                    parent.as_ref(),
                                    &child,
                                    |_, _| async { Ok(output(7_u64)) },
                                )?
                                .await?;
                            Ok(output(format!("items={transport}")))
                        }
                    },
                )?
                .await?;
            assert_eq!(wrapper.as_str(), "items=7");
            let warm = cache
                .cached_work(
                    identity("wrapper-transport"),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |_, _| async { Ok(output("should not execute".to_owned())) },
                )?
                .await?;
            assert!(Arc::ptr_eq(&wrapper, &warm));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancellation_waits_for_required_completion_before_a_waiter_retries() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let leader_token = CancellationToken::new();
            let completed = Arc::new(AtomicUsize::new(0));
            let (started, ready) = oneshot::channel();
            let (release, released) = oneshot::channel();
            let leader = leader_token
                .with_async_scope(|child| {
                    let cache = &cache;
                    let completed = completed.clone();
                    async move {
                        cache
                            .cached_work(
                                identity("required-completion"),
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ReusableRead,
                                VALID_FOR,
                                None,
                                &child,
                                |_, execution| async move {
                                    execution
                                        .deferred(
                                            "finish a synthetic observed outcome",
                                            |_protected| async move {
                                                started.send(()).unwrap();
                                                released.await?;
                                                completed.fetch_add(1, Ordering::SeqCst);
                                                Ok(output(1_u64))
                                            },
                                        )
                                        .await
                                },
                            )?
                            .await
                    }
                })
                .into_future();
            tokio::pin!(leader);
            assert_pending(leader.as_mut()).await;
            ready.await?;
            let follower = read(&cache, "required-completion", &scope, 2);
            tokio::pin!(follower);
            assert_pending(follower.as_mut()).await;
            leader_token.request_cancel("cancel after synthetic required work starts");
            assert_pending(leader.as_mut()).await;
            assert_pending(follower.as_mut()).await;
            assert_eq!(completed.load(Ordering::SeqCst), 0);
            release.send(()).unwrap();
            assert!(timeout(DEADLOCK_GUARD, leader).await?.is_err());
            assert_eq!(completed.load(Ordering::SeqCst), 1);
            assert_eq!(*timeout(DEADLOCK_GUARD, follower).await??, 2);
            Ok(())
        })
        .await
}
