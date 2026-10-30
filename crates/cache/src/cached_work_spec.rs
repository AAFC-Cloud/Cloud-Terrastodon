use crate::ArtifactFreshness;
use crate::CacheEntryIdentity;
use crate::WorkAttempt;
use crate::WorkCache;
use crate::WorkOutput;
use crate::artifact_age::ArtifactAge;
use crate::artifact_io_guard::ArtifactIoGuard;
use crate::cached_work_value::CachedWorkValue;
use crate::work_attempt_guard::WorkAttemptGuard;
use crate::work_coordination::WorkCoordination;
use crate::work_wait_guard::WorkWaitGuard;
use eyre::Result;
use eyre::ensure;
use std::future::Future;
use std::future::IntoFuture;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use teamy_cancellation::CancellationScope;

/// Awaitable canonical work bound to an explicit cache owner and caller scope.
///
/// Construct with `WorkCache::cached_work` to claim the operation/output contract
/// before any lookup. Only successful fresh values are shared. The callback owns
/// execution and canonical decoding; caller projections belong after this await.
/// This layer currently coordinates memory results, not generation/dependency
/// validity or the transport/artifact lifecycle. Production adapter cutover waits
/// for those subsequent migration tasks.
#[must_use = "Cached work does nothing until awaited"]
pub struct CachedWorkSpec<'a, T, F> {
    // Bound fields cannot be changed after the canonical ownership claim.
    pub(crate) cache: WorkCache,
    pub(crate) identity: CacheEntryIdentity,
    pub(crate) validity: Duration,
    pub(crate) parent: Option<&'a WorkAttempt>,
    pub(crate) cancellation: &'a CancellationScope,
    pub(crate) initialize: F,
    pub(crate) output: PhantomData<fn() -> T>,
}

impl<'a, T, F, Fut> IntoFuture for CachedWorkSpec<'a, T, F>
where
    T: Send + Sync + 'static,
    F: FnOnce(Option<WorkAttempt>, CancellationScope) -> Fut + Send + 'a,
    Fut: Future<Output = Result<WorkOutput<T>>> + Send + 'a,
{
    type Output = Result<Arc<T>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(
            self.cancellation
                .cancellable_scope(move |scope| self.run(scope)),
        )
    }
}

impl<T, F, Fut> CachedWorkSpec<'_, T, F>
where
    T: Send + Sync + 'static,
    F: FnOnce(Option<WorkAttempt>, CancellationScope) -> Fut,
    Fut: Future<Output = Result<WorkOutput<T>>>,
{
    async fn run(self, scope: CancellationScope) -> Result<Arc<T>> {
        let mut retired = None;
        let (coordination, _waiting) = {
            let mut state = self.cache.lock_state()?;
            let parent = self
                .parent
                .map(|parent| parent.entry(&self.cache, &state).cloned())
                .transpose()?;
            if self.validity.is_zero() {
                // No lock or artificial ancestry vertex for nonreusable work.
                (None, None)
            } else {
                ensure!(
                    parent.as_ref() != Some(&self.identity),
                    "recursive cache initialization for {:?}",
                    self.identity
                );
                if state
                    .values
                    .get(&self.identity)
                    .is_some_and(|value| !value.is_reusable(Duration::MAX))
                {
                    retired = state.values.remove(&self.identity);
                }
                if let Some(value) = state.values.get(&self.identity)
                    && let Some(value) = value.load::<T>(self.validity)?
                {
                    return Ok(value);
                }
                let waiting = parent
                    .map(|parent| {
                        WorkWaitGuard::new(&self.cache, &mut state, parent, self.identity.clone())
                    })
                    .transpose()?;
                state
                    .coordination
                    .retain(|_, entry| entry.strong_count() > 0);
                let coordination = state
                    .coordination
                    .get(&self.identity)
                    .and_then(|entry| entry.upgrade())
                    .unwrap_or_else(|| {
                        let entry = Arc::new(WorkCoordination::default());
                        state
                            .coordination
                            .insert(self.identity.clone(), Arc::downgrade(&entry));
                        entry
                    });
                (Some(coordination), waiting)
            }
        };
        drop(retired);

        let Some(coordination) = coordination else {
            let output = (self.initialize)(self.parent.cloned(), scope.clone()).await?;
            scope.bail_if_cancelled()?;
            return Ok(Arc::new(output.value));
        };
        let _initializer = coordination.initializer.lock().await;
        scope.bail_if_cancelled()?;
        let mut retired = None;
        {
            let mut state = self.cache.lock_state()?;
            if let Some(parent) = self.parent {
                parent.entry(&self.cache, &state)?;
            }
            // A completed leader's value must satisfy this waiting caller's bound.
            if state
                .values
                .get(&self.identity)
                .is_some_and(|value| !value.is_reusable(Duration::MAX))
            {
                retired = state.values.remove(&self.identity);
            }
            if let Some(value) = state.values.get(&self.identity)
                && let Some(value) = value.load::<T>(self.validity)?
            {
                return Ok(value);
            }
        }
        drop(retired);
        let attempt = WorkAttemptGuard::new(&self.cache, self.identity.clone())?;
        let output = (self.initialize)(Some(attempt.attempt()), scope.clone()).await?;
        scope.bail_if_cancelled()?;
        let value = Arc::new(output.value);
        let age = ArtifactAge {
            freshness: ArtifactFreshness::new(output.observation.wall, self.validity)?,
            initial_age: Duration::ZERO,
            anchored_at: output.observation.monotonic,
        };
        let retained = CachedWorkValue {
            value: value.clone(),
            age,
        };
        let gate = ArtifactIoGuard::acquire(&self.cache.state.artifact_gate).await;
        scope.bail_if_cancelled()?;
        let retired = {
            let mut state = self.cache.lock_state()?;
            if let Some(parent) = self.parent {
                parent.entry(&self.cache, &state)?;
            }
            if retained.is_reusable(self.validity) {
                state.values.insert(self.identity, retained)
            } else {
                // An outcome that expired during decoding still belongs to its
                // caller, but cannot become a reusable result.
                Some(retained)
            }
        };
        drop(gate);
        // User value destructors run outside both cache locks.
        drop(retired);
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactObservation;
    use crate::AuthPartition;
    use crate::CacheOperationOwner;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use crate::NonEmptyString;
    use chrono::TimeDelta;
    use chrono::Utc;
    use std::future::poll_fn;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::task::Poll;
    use std::time::Instant;
    use teamy_cancellation::CancellationToken;
    use tokio::sync::oneshot;
    use tokio::time::timeout;

    struct SyntheticRead;
    struct ConflictingRead;

    fn identity() -> CacheEntryIdentity {
        CacheEntryIdentity {
            kind: ExecutorKind::InProcess,
            scope: CacheScope::try_from_components(["synthetic", "freshness"]).unwrap(),
            auth_partition: AuthPartition::new("test-session").unwrap(),
            fingerprint: NonEmptyString::new("test-inputs").unwrap(),
        }
    }

    fn observed_before(age: Duration) -> ArtifactObservation {
        ArtifactObservation {
            wall: Utc::now() - TimeDelta::from_std(age).unwrap(),
            monotonic: Instant::now() - age,
        }
    }

    async fn assert_pending<F: Future>(mut future: Pin<&mut F>) {
        poll_fn(|context| {
            assert!(future.as_mut().poll(context).is_pending());
            Poll::Ready(())
        })
        .await;
    }

    #[tokio::test]
    async fn an_outcome_that_expired_before_decode_completed_is_returned_without_reuse()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let temporary = tempfile::tempdir()?;
                let cache = WorkCache::new(temporary.path())?;
                let executions = AtomicUsize::new(0);
                let expired = cache
                    .cached_work(
                        identity(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        Duration::from_secs(30),
                        None,
                        &scope,
                        |_, _| async {
                            let observation = observed_before(Duration::from_secs(60));
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(WorkOutput {
                                value: 1_u64,
                                observation,
                            })
                        },
                    )?
                    .await?;
                assert_eq!(*expired, 1);
                assert!(!cache.lock_state()?.values.contains_key(&identity()));

                let fresh = cache
                    .cached_work(
                        identity(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        Duration::from_secs(30),
                        None,
                        &scope,
                        |_, _| async {
                            let observation = ArtifactObservation::now();
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(WorkOutput {
                                value: 2_u64,
                                observation,
                            })
                        },
                    )?
                    .await?;
                assert_eq!(*fresh, 2);
                assert!(!Arc::ptr_eq(&expired, &fresh));
                assert_eq!(executions.load(Ordering::SeqCst), 2);
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn waiter_rechecks_its_stricter_age_bound_and_longer_hits_do_not_renew_expiry()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let temporary = tempfile::tempdir()?;
                let cache = WorkCache::new(temporary.path())?;
                let executions = AtomicUsize::new(0);
                let observation = observed_before(Duration::from_secs(600));
                let stored_validity = Duration::from_secs(3600);
                let (release, released) = oneshot::channel();
                let leader = cache
                    .cached_work(
                        identity(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        stored_validity,
                        None,
                        &scope,
                        |_, _| {
                            let executions = &executions;
                            async move {
                                executions.fetch_add(1, Ordering::SeqCst);
                                released.await?;
                                Ok(WorkOutput {
                                    value: 1_u64,
                                    observation,
                                })
                            }
                        },
                    )?
                    .into_future();
                tokio::pin!(leader);
                assert_pending(leader.as_mut()).await;

                let waiter = cache
                    .cached_work(
                        identity(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        Duration::from_secs(300),
                        None,
                        &scope,
                        |_, _| async {
                            let observation = ArtifactObservation::now();
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(WorkOutput {
                                value: 2_u64,
                                observation,
                            })
                        },
                    )?
                    .into_future();
                tokio::pin!(waiter);
                assert_pending(waiter.as_mut()).await;
                release.send(()).unwrap();
                let first = timeout(Duration::from_secs(10), leader).await??;

                // The stricter waiter is still unpolled after the leader's fill.
                // A permissive caller can use that fill, but cannot renew its age.
                let longer_validity = Duration::from_secs(86_400);
                let permissive = cache
                    .cached_work(
                        identity(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        longer_validity,
                        None,
                        &scope,
                        |_, _| async {
                            executions.fetch_add(1, Ordering::SeqCst);
                            Ok(WorkOutput {
                                value: 3_u64,
                                observation: ArtifactObservation::now(),
                            })
                        },
                    )?
                    .await?;
                assert!(Arc::ptr_eq(&first, &permissive));
                {
                    let state = cache.lock_state()?;
                    let retained = &state.values[&identity()];
                    assert_eq!(retained.age.freshness.observed_at, observation.wall);
                    assert_eq!(
                        retained.age.freshness.expires_at,
                        Some(observation.wall + TimeDelta::seconds(3600)),
                    );
                    assert!(!retained.age.reusable(
                        &retained.age.freshness,
                        longer_validity,
                        observation.wall + TimeDelta::seconds(3601),
                        observation.monotonic + Duration::from_secs(3601),
                    ));
                }
                assert_eq!(executions.load(Ordering::SeqCst), 1);

                let stricter = timeout(Duration::from_secs(10), waiter).await??;
                assert_eq!(*stricter, 2);
                assert!(!Arc::ptr_eq(&first, &stricter));
                assert_eq!(executions.load(Ordering::SeqCst), 2);
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn memory_release_reclaims_dead_coordination_without_releasing_ownership_claims()
    -> Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let temporary = tempfile::tempdir()?;
                let cache = WorkCache::new(temporary.path())?;
                let value = cache
                    .cached_work(
                        identity(),
                        CacheOperationOwner::of::<SyntheticRead>(),
                        Duration::from_secs(3600),
                        None,
                        &scope,
                        |_, _| async {
                            Ok(WorkOutput {
                                value: 7_u64,
                                observation: ArtifactObservation::now(),
                            })
                        },
                    )?
                    .await?;
                {
                    let state = cache.lock_state()?;
                    assert!(state.coordination[&identity()].upgrade().is_none());
                    assert_eq!(state.values.len(), 1);
                    assert_eq!(state.claims.len(), 1);
                }
                cache.release_memory()?;
                {
                    let state = cache.lock_state()?;
                    assert!(state.coordination.is_empty());
                    assert!(state.values.is_empty());
                    assert_eq!(state.claims.len(), 1);
                }
                assert!(
                    cache
                        .cached_work(
                            CacheEntryIdentity {
                                fingerprint: NonEmptyString::new("another-variant")?,
                                ..identity()
                            },
                            CacheOperationOwner::of::<ConflictingRead>(),
                            Duration::from_secs(3600),
                            None,
                            &scope,
                            |_, _| async {
                                Ok(WorkOutput {
                                    value: 8_u64,
                                    observation: ArtifactObservation::now(),
                                })
                            },
                        )
                        .is_err()
                );
                assert_eq!(*value, 7);
                Ok(())
            })
            .await
    }
}
