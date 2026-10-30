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
use std::task::Poll;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;
use tokio::time::timeout;

const VALID_FOR: Duration = Duration::from_secs(3600);
const DEADLOCK_GUARD: Duration = Duration::from_secs(10);

struct SyntheticRead;
struct ConflictingRead;

fn identity(components: &[&str]) -> CacheEntryIdentity {
    CacheEntryIdentity {
        kind: ExecutorKind::InProcess,
        scope: CacheScope::try_from_components(components.iter().copied()).unwrap(),
        auth_partition: AuthPartition::new("synthetic-session").unwrap(),
        fingerprint: NonEmptyString::new("synthetic-inputs").unwrap(),
    }
}

fn output<T>(value: T) -> WorkOutput<T> {
    WorkOutput::new(value, ArtifactObservation::now())
}

async fn read(
    cache: &WorkCache,
    key: CacheEntryIdentity,
    parent: Option<&WorkAttempt>,
    scope: &CancellationScope,
    value: u64,
) -> eyre::Result<Arc<u64>> {
    cache
        .cached_work(
            key,
            CacheOperationOwner::of::<SyntheticRead>(),
            WorkReuse::ReusableRead,
            VALID_FOR,
            parent,
            scope,
            |_, _| async move { Ok(output(value)) },
        )?
        .await
}

async fn compose(
    cache: &WorkCache,
    key: CacheEntryIdentity,
    child_key: CacheEntryIdentity,
    scope: &CancellationScope,
) -> eyre::Result<Arc<u64>> {
    cache
        .cached_work(
            key,
            CacheOperationOwner::of::<SyntheticRead>(),
            WorkReuse::ReusableRead,
            VALID_FOR,
            None,
            scope,
            |parent, child| async move {
                let value = read(cache, child_key, parent.as_ref(), &child, 100).await?;
                Ok(output(*value + 1))
            },
        )?
        .await
}

// Assert an actual pending poll before changing state. No scheduler timing or
// sleep is used to arrange invalidation, cancellation, or future-drop races.
async fn assert_pending<F: Future + ?Sized>(mut future: Pin<&mut F>) {
    poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn warm_child_hits_carry_transitive_dependencies_across_branches() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let users = identity(&["directory", "users"]);
            let principals = identity(&["inventory", "principals"]);
            let audit = identity(&["audits", "access"]);
            let unrelated = identity(&["directory", "groups"]);
            let users_value = read(&cache, users.clone(), None, &scope, 1).await?;
            let principals_value =
                compose(&cache, principals.clone(), users.clone(), &scope).await?;
            let audit_value = compose(&cache, audit.clone(), principals.clone(), &scope).await?;
            let unrelated_value = read(&cache, unrelated.clone(), None, &scope, 8).await?;
            assert_eq!((*users_value, *principals_value, *audit_value), (1, 2, 3));

            cache.invalidate(&users.scope, &scope).await?;

            assert_eq!(*read(&cache, users, None, &scope, 4).await?, 4);
            assert_eq!(*read(&cache, principals, None, &scope, 5).await?, 5);
            assert_eq!(*read(&cache, audit, None, &scope, 6).await?, 6);
            assert!(Arc::ptr_eq(
                &unrelated_value,
                &read(&cache, unrelated, None, &scope, 9).await?
            ));
            // Invalidation does not revoke an already returned snapshot.
            assert_eq!((*users_value, *principals_value, *audit_value), (1, 2, 3));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn exact_rejection_preserves_variants_while_prefix_invalidation_covers_descendants()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["projects", "alpha"]);
            let variants = [
                CacheEntryIdentity {
                    fingerprint: NonEmptyString::new("other-inputs")?,
                    ..key.clone()
                },
                CacheEntryIdentity {
                    auth_partition: AuthPartition::new("other-session")?,
                    ..key.clone()
                },
                CacheEntryIdentity {
                    kind: ExecutorKind::Rest,
                    ..key.clone()
                },
                identity(&["projects", "alpha", "pages", "1"]),
            ];
            let sibling = identity(&["projects", "alphabet"]);
            let sibling_value = read(&cache, sibling.clone(), None, &scope, 20).await?;
            read(&cache, key.clone(), None, &scope, 1).await?;
            let mut original = Vec::new();
            for variant in &variants {
                original.push(read(&cache, variant.clone(), None, &scope, 2).await?);
            }

            cache.reject(&key, &scope).await?;
            assert_eq!(*read(&cache, key.clone(), None, &scope, 3).await?, 3);
            for (variant, value) in variants.iter().zip(&original) {
                assert!(Arc::ptr_eq(
                    value,
                    &read(&cache, variant.clone(), None, &scope, 4).await?
                ));
            }

            cache.invalidate(&key.scope, &scope).await?;
            for variant in variants {
                assert_eq!(*read(&cache, variant, None, &scope, 5).await?, 5);
            }
            assert!(Arc::ptr_eq(
                &sibling_value,
                &read(&cache, sibling, None, &scope, 21).await?
            ));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn invalidation_during_a_fill_returns_the_snapshot_but_a_waiter_must_retry()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["projects", "alpha", "inventory"]);
            let (release, released) = oneshot::channel();
            let leader = cache
                .cached_work(
                    key.clone(),
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
            let waiter = read(&cache, key.clone(), None, &scope, 2);
            tokio::pin!(leader, waiter);
            assert_pending(leader.as_mut()).await;
            assert_pending(waiter.as_mut()).await;
            cache
                .invalidate(
                    &CacheScope::try_from_components(["projects", "alpha"])?,
                    &scope,
                )
                .await?;
            release.send(()).unwrap();
            let (first, next) =
                timeout(DEADLOCK_GUARD, async { tokio::join!(leader, waiter) }).await?;
            assert_eq!(*first?, 1);
            let next = next?;
            assert_eq!(*next, 2);
            assert!(Arc::ptr_eq(
                &next,
                &read(&cache, key, None, &scope, 3).await?
            ));
            Ok(())
        })
        .await
}

#[tokio::test]
async fn new_descendants_after_invalidation_do_not_refresh_an_older_parent_stamp()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["projects", "alpha", "inventory"]);
            let page = identity(&["projects", "alpha", "pages", "new"]);
            let (release, released) = oneshot::channel();
            let attempt = cache
                .cached_work(
                    key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        let page = page.clone();
                        async move {
                            released.await?;
                            let value = read(cache, page, parent.as_ref(), &child, 7).await?;
                            Ok(output(*value))
                        }
                    },
                )?
                .into_future();
            tokio::pin!(attempt);
            assert_pending(attempt.as_mut()).await;
            cache
                .invalidate(
                    &CacheScope::try_from_components(["projects", "alpha"])?,
                    &scope,
                )
                .await?;
            release.send(()).unwrap();
            assert_eq!(*timeout(DEADLOCK_GUARD, attempt).await??, 7);
            assert_eq!(*read(&cache, key, None, &scope, 8).await?, 8);
            assert_eq!(*read(&cache, page.clone(), None, &scope, 9).await?, 7);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn self_invalidation_can_recover_without_publishing_under_its_original_generation()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["directory", "users"]);
            let page = identity(&["directory", "users", "pages", "1"]);
            let recovered = cache
                .cached_work(
                    key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        let key = key.clone();
                        async move {
                            assert_eq!(
                                *read(cache, page.clone(), parent.as_ref(), &child, 1).await?,
                                1
                            );
                            cache.invalidate(&key.scope, &child).await?;
                            let retry = read(cache, page, parent.as_ref(), &child, 2).await?;
                            Ok(output(*retry))
                        }
                    },
                )?
                .into_future();
            assert_eq!(*timeout(DEADLOCK_GUARD, recovered).await??, 2);
            assert_eq!(*read(&cache, key, None, &scope, 3).await?, 3);
            Ok(())
        })
        .await
}

#[derive(Clone, Copy)]
enum ChildExit {
    Error,
    Cancel,
    DropPending,
    DropUnpolled,
    ClaimConflict,
}

#[tokio::test]
async fn a_new_child_generation_cannot_replace_an_earlier_consumed_observation() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["inventory", "composed"]);
            let child_key = identity(&["directory", "users"]);
            let mixed = cache
                .cached_work(
                    key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        async move {
                            let earlier =
                                read(cache, child_key.clone(), parent.as_ref(), &child, 1).await?;
                            cache.invalidate(&child_key.scope, &child).await?;
                            let later = read(cache, child_key, parent.as_ref(), &child, 2).await?;
                            Ok(output(*earlier + *later))
                        }
                    },
                )?
                .await?;
            assert_eq!(*mixed, 3);
            assert_eq!(*read(&cache, key, None, &scope, 4).await?, 4);
            Ok(())
        })
        .await
}

async fn caught_child_failure(exit: ChildExit) -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let child_key = identity(&["directory", "failing-child"]);
            if matches!(exit, ChildExit::ClaimConflict) {
                read(&cache, child_key.clone(), None, &scope, 1).await?;
            }
            let parent_key = identity(&["inventory", "parent"]);
            let grandparent_key = identity(&["audits", "grandparent"]);
            let fallback = cache
                .cached_work(
                    grandparent_key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |grandparent, child| {
                        let cache = &cache;
                        let parent_key = parent_key.clone();
                        async move {
                            let parent = cache
                                .cached_work(
                                    parent_key,
                                    CacheOperationOwner::of::<SyntheticRead>(),
                                    WorkReuse::ReusableRead,
                                    VALID_FOR,
                                    grandparent.as_ref(),
                                    &child,
                                    |parent, child| async move {
                                        match exit {
                                            ChildExit::ClaimConflict => {
                                                let conflict = cache.cached_work::<u64, _, _>(
                                                    child_key,
                                                    CacheOperationOwner::of::<ConflictingRead>(),
                                                    WorkReuse::ReusableRead,
                                                    VALID_FOR,
                                                    parent.as_ref(),
                                                    &child,
                                                    |_, _| async {
                                                        panic!("conflicting child executed")
                                                    },
                                                );
                                                assert!(conflict.is_err());
                                            }
                                            ChildExit::Error => {
                                                let result = cache
                                                    .cached_work::<u64, _, _>(
                                                        child_key,
                                                        CacheOperationOwner::of::<SyntheticRead>(),
                                                        WorkReuse::ReusableRead,
                                                        VALID_FOR,
                                                        parent.as_ref(),
                                                        &child,
                                                        |_, _| async {
                                                            Err(eyre::eyre!(
                                                                "synthetic child failure"
                                                            ))
                                                        },
                                                    )?
                                                    .await;
                                                assert!(result.is_err());
                                            }
                                            ChildExit::DropPending | ChildExit::DropUnpolled => {
                                                let spec = cache.cached_work::<u64, _, _>(
                                                    child_key,
                                                    CacheOperationOwner::of::<SyntheticRead>(),
                                                    WorkReuse::ReusableRead,
                                                    VALID_FOR,
                                                    parent.as_ref(),
                                                    &child,
                                                    |_, _| {
                                                        std::future::pending::<
                                                            eyre::Result<WorkOutput<u64>>,
                                                        >(
                                                        )
                                                    },
                                                )?;
                                                if matches!(exit, ChildExit::DropUnpolled) {
                                                    drop(spec);
                                                } else {
                                                    let mut future = spec.into_future();
                                                    assert_pending(future.as_mut()).await;
                                                    drop(future);
                                                }
                                            }
                                            ChildExit::Cancel => {
                                                let token = CancellationToken::new();
                                                let future = token
                                                    .with_async_scope(|cancel_scope| async move {
                                                        cache
                                                            .cached_work::<u64, _, _>(
                                                                child_key,
                                                                CacheOperationOwner::of::<
                                                                    SyntheticRead,
                                                                >(
                                                                ),
                                                                WorkReuse::ReusableRead,
                                                                VALID_FOR,
                                                                parent.as_ref(),
                                                                &cancel_scope,
                                                                |_, _| {
                                                                    std::future::pending::<
                                                                        eyre::Result<
                                                                            WorkOutput<u64>,
                                                                        >,
                                                                    >(
                                                                    )
                                                                },
                                                            )?
                                                            .await
                                                    })
                                                    .into_future();
                                                tokio::pin!(future);
                                                assert_pending(future.as_mut()).await;
                                                token.request_cancel(
                                                    "cancel only the synthetic semantic child",
                                                );
                                                assert!(
                                                    timeout(DEADLOCK_GUARD, future).await?.is_err()
                                                );
                                            }
                                        }
                                        Ok(output(10_u64))
                                    },
                                )?
                                .await?;
                            Ok(output(*parent + 1))
                        }
                    },
                )?
                .await?;
            assert_eq!(*fallback, 11);
            assert_eq!(*read(&cache, parent_key, None, &scope, 20).await?, 20);
            assert_eq!(*read(&cache, grandparent_key, None, &scope, 30).await?, 30);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn caught_child_error_prevents_parent_and_grandparent_reuse() -> eyre::Result<()> {
    caught_child_failure(ChildExit::Error).await
}

#[tokio::test]
async fn caught_child_cancellation_prevents_parent_and_grandparent_reuse() -> eyre::Result<()> {
    caught_child_failure(ChildExit::Cancel).await
}

#[tokio::test]
async fn dropped_pending_child_prevents_parent_and_grandparent_reuse() -> eyre::Result<()> {
    caught_child_failure(ChildExit::DropPending).await
}

#[tokio::test]
async fn dropped_unpolled_child_prevents_parent_and_grandparent_reuse() -> eyre::Result<()> {
    caught_child_failure(ChildExit::DropUnpolled).await
}

#[tokio::test]
async fn caught_child_claim_failure_prevents_parent_and_grandparent_reuse() -> eyre::Result<()> {
    caught_child_failure(ChildExit::ClaimConflict).await
}

#[tokio::test]
async fn a_refilling_child_cannot_consume_a_warm_ancestor_certificate() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let child_key = identity(&["directory", "child"]);
            let parent_key = identity(&["inventory", "parent"]);
            read(&cache, child_key.clone(), None, &scope, 1).await?;
            compose(&cache, parent_key.clone(), child_key.clone(), &scope).await?;
            // A short caller bound forces a refill without invalidating the
            // earlier long-lived certificate retained by the parent.
            let result = cache
                .cached_work(
                    child_key,
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    Duration::from_nanos(1),
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        async move {
                            let cycle = read(cache, parent_key, parent.as_ref(), &child, 99).await;
                            assert!(cycle.is_err());
                            Ok(output(3_u64))
                        }
                    },
                )?
                .into_future();
            assert_eq!(*timeout(DEADLOCK_GUARD, result).await??, 3);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn explicit_nonreuse_bypasses_a_warm_hit_and_is_rejected_under_a_reusable_parent()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["workflows", "conditional-effect"]);
            let warm = read(&cache, key.clone(), None, &scope, 1).await?;
            for value in [2_u64, 3] {
                let executed = cache
                    .cached_work(
                        key.clone(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        WorkReuse::ExecuteEveryTime,
                        VALID_FOR,
                        None,
                        &scope,
                        |_, _| async move { Ok(output(value)) },
                    )?
                    .await?;
                assert_eq!(*executed, value);
                assert!(!Arc::ptr_eq(&warm, &executed));
            }
            let parent_key = identity(&["audits", "incorrectly-memoized"]);
            let fallback = cache
                .cached_work(
                    parent_key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, child| {
                        let cache = &cache;
                        async move {
                            let rejected = cache.cached_work::<u64, _, _>(
                                key,
                                CacheOperationOwner::of::<SyntheticRead>(),
                                WorkReuse::ExecuteEveryTime,
                                VALID_FOR,
                                parent.as_ref(),
                                &child,
                                |_, _| async { panic!("incompatible semantic child executed") },
                            );
                            assert!(rejected.is_err());
                            Ok(output(4_u64))
                        }
                    },
                )?
                .await?;
            assert_eq!(*fallback, 4);
            assert_eq!(*read(&cache, parent_key, None, &scope, 5).await?, 5);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_parent_cannot_publish_with_an_outstanding_semantic_child() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let key = identity(&["inventory", "unfinished"]);
            let (release, released) = oneshot::channel();
            let (handle_sender, handle_receiver) = oneshot::channel();
            let result = cache
                .cached_work(
                    key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    WorkReuse::ReusableRead,
                    VALID_FOR,
                    None,
                    &scope,
                    |parent, _| {
                        let cache = cache.clone();
                        async move {
                            let (started, ready) = oneshot::channel();
                            let handle = tokio::spawn(CancellationToken::new().with_async_scope(
                                |child| async move {
                                    cache
                                        .cached_work(
                                            identity(&["directory", "outstanding"]),
                                            CacheOperationOwner::of::<SyntheticRead>(),
                                            WorkReuse::ReusableRead,
                                            VALID_FOR,
                                            parent.as_ref(),
                                            &child,
                                            |_, _| async move {
                                                started.send(()).unwrap();
                                                released.await?;
                                                Ok(output(1_u64))
                                            },
                                        )?
                                        .await
                                },
                            ));
                            timeout(DEADLOCK_GUARD, ready).await??;
                            handle_sender.send(handle).unwrap();
                            Ok(output(2_u64))
                        }
                    },
                )?
                .await?;
            assert_eq!(*result, 2);
            assert_eq!(*read(&cache, key.clone(), None, &scope, 3).await?, 3);
            release.send(()).unwrap();
            // The child's owner has finished. Whether its own value is returned
            // or rejected, completing it cannot revive the old parent attempt.
            let _child_result = timeout(DEADLOCK_GUARD, handle_receiver.await?).await??;
            assert_eq!(*read(&cache, key, None, &scope, 4).await?, 3);
            Ok(())
        })
        .await
}
