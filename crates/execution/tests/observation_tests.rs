#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkCacheStatus;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::CollectionProgress;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionEvent;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::cell::RefCell;
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
use teamy_cancellation::CancellationToken;
use tokio::sync::oneshot;

struct CollectionRead<'a> {
    starts: &'a AtomicUsize,
    release: Option<oneshot::Receiver<()>>,
}

impl HasCacheKey for CollectionRead<'_> {
    fn cache_key(&self) -> Result<CacheKey> {
        CacheKey::from_relative_path("synthetic/observed-collection")
    }
}

impl InvalidatesOwnCacheKey for CollectionRead<'_> {}

impl ExecutableRequest for CollectionRead<'_> {
    type Output = Arc<Vec<usize>>;
    type Error = eyre::Report;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            executor
                .cached_work(
                    ExecutorKind::InProcess,
                    self.cache_key()?,
                    CacheOperationOwner::of::<CollectionRead<'static>>(),
                    &AuthenticationSelection::Anonymous,
                    NonEmptyString::new("synthetic-collection")?,
                    WorkReuse::ReusableRead,
                    move |child| async move {
                        let value = self.starts.fetch_add(1, Ordering::SeqCst);
                        if let Some(release) = self.release {
                            release.await?;
                        }
                        child.report_progress(CollectionProgress {
                            total: Some(1),
                            accumulated: 1,
                            received_this_page: 1,
                            time_remaining: Duration::ZERO,
                        });
                        Ok(WorkOutput::new(vec![value], ArtifactObservation::now()))
                    },
                )?
                .await
        })
    }
}

fn labels(events: &[ExecutionEvent]) -> Vec<&'static str> {
    events
        .iter()
        .map(|event| match event {
            ExecutionEvent::Admitted => "admitted",
            ExecutionEvent::Cache { status, .. } => match status {
                WorkCacheStatus::Hit => "hit",
                WorkCacheStatus::Waiting => "waiting",
                WorkCacheStatus::Initializing => "initializing",
            },
            ExecutionEvent::Progress(_) => "progress",
            ExecutionEvent::Completed => "completed",
            ExecutionEvent::Failed => "failed",
        })
        .collect()
}

async fn assert_pending<F: Future + ?Sized>(mut future: Pin<&mut F>) {
    poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[tokio::test]
async fn initializer_joiner_and_hit_have_separate_observers_and_shared_results() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            // Observe initializer coordination after lazy root preparation.
            executor.cache()?.acquire(executor.cancellation()).await?;
            let starts = AtomicUsize::new(0);
            let (release, released) = oneshot::channel();
            // Callbacks borrow non-Send state; neither callbacks nor their state are
            // retained by cache values or moved into detached work.
            let leader_events = Rc::new(RefCell::new(Vec::new()));
            let waiter_events = Rc::new(RefCell::new(Vec::new()));
            let mut leader = Box::pin(
                executor
                    .execute(CollectionRead {
                        starts: &starts,
                        release: Some(released),
                    })
                    .observe(|event| leader_events.borrow_mut().push(event)),
            );
            assert_pending(leader.as_mut()).await;
            let mut waiter = Box::pin(
                executor
                    .execute(CollectionRead {
                        starts: &starts,
                        release: None,
                    })
                    .observe(|event| waiter_events.borrow_mut().push(event)),
            );
            assert_pending(waiter.as_mut()).await;
            assert_eq!(labels(&waiter_events.borrow()), ["admitted", "waiting"]);
            release.send(()).unwrap();
            let first = leader.await?;
            let second = waiter.await?;
            assert!(Arc::ptr_eq(&first, &second));
            assert_eq!(starts.load(Ordering::SeqCst), 1);
            assert_eq!(
                labels(&leader_events.borrow()),
                ["admitted", "initializing", "progress", "completed"]
            );
            assert_eq!(
                labels(&waiter_events.borrow()),
                ["admitted", "waiting", "hit", "completed"]
            );

            let mut hit_events = Vec::new();
            let hit = executor
                .execute(CollectionRead {
                    starts: &starts,
                    release: None,
                })
                .with_invalidation(false)
                .observe(|event| hit_events.push(event))
                .await?;
            assert!(Arc::ptr_eq(&first, &hit));
            assert_eq!(labels(&hit_events), ["admitted", "hit", "completed"]);
            assert_eq!(Rc::strong_count(&leader_events), 1);

            let mut refresh_events = Vec::new();
            let refreshed = executor
                .execute(CollectionRead {
                    starts: &starts,
                    release: None,
                })
                .with_invalidation(true)
                .observe(|event| refresh_events.push(event))
                .await?;
            assert!(!Arc::ptr_eq(&first, &refreshed));
            assert_eq!(*first, [0]);
            assert_eq!(*refreshed, [1]);
            assert_eq!(
                labels(&refresh_events),
                ["admitted", "initializing", "progress", "completed"]
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancelled_initializer_reports_failure_and_waiter_retries() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            executor.cache()?.acquire(executor.cancellation()).await?;
            let starts = AtomicUsize::new(0);
            let (_release, released) = oneshot::channel();
            let events = RefCell::new(Vec::new());
            let (cancel, leader) = executor.with_async_child_scope(async |child| {
                child
                    .execute(CollectionRead {
                        starts: &starts,
                        release: Some(released),
                    })
                    .observe(|event| events.borrow_mut().push(event))
                    .await
            });
            let mut leader = Box::pin(leader);
            assert_pending(leader.as_mut()).await;
            let mut waiter = Box::pin(
                executor
                    .execute(CollectionRead {
                        starts: &starts,
                        release: None,
                    })
                    .into_future(),
            );
            assert_pending(waiter.as_mut()).await;
            cancel.request_cancel("synthetic initializer cancellation");
            assert!(leader.await.is_err());
            let value = waiter.await?;
            assert_eq!(*value, [1]);
            assert_eq!(starts.load(Ordering::SeqCst), 2);
            assert_eq!(
                labels(&events.borrow()),
                ["admitted", "initializing", "failed"]
            );
            Ok(())
        })
        .await
}

struct ReadTwice<'a>(&'a AtomicUsize);

impl ExecutableRequest for ReadTwice<'_> {
    type Output = Arc<Vec<usize>>;
    type Error = eyre::Report;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            let first = executor
                .execute(CollectionRead {
                    starts: self.0,
                    release: None,
                })
                .await?;
            let second = executor
                .execute(CollectionRead {
                    starts: self.0,
                    release: None,
                })
                .await?;
            assert!(Arc::ptr_eq(&first, &second));
            Ok(second)
        })
    }
}

#[tokio::test]
async fn observing_composition_reports_one_invocation_lifecycle_and_child_work() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let starts = AtomicUsize::new(0);
            let mut events = Vec::new();
            executor
                .execute(ReadTwice(&starts))
                .observe(|event| events.push(event))
                .await?;
            assert_eq!(
                labels(&events),
                ["admitted", "initializing", "progress", "hit", "completed"]
            );
            assert_eq!(starts.load(Ordering::SeqCst), 1);
            Ok(())
        })
        .await
}

struct ProgressBurst {
    count: usize,
}

impl ExecutableRequest for ProgressBurst {
    type Output = ();
    type Error = eyre::Report;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
    {
        Box::pin(async move {
            for accumulated in 1..=self.count {
                executor.report_progress(CollectionProgress {
                    total: Some(self.count),
                    accumulated,
                    received_this_page: 1,
                    time_remaining: Duration::ZERO,
                });
            }
            Ok(())
        })
    }
}

#[tokio::test]
async fn completion_drains_a_progress_burst_beyond_the_cooperative_poll_budget() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let count = 1_024;
            let mut events = Vec::new();
            executor
                .execute(ProgressBurst { count })
                .observe(|event| events.push(event))
                .await?;
            assert_eq!(events.len(), count + 2);
            assert!(matches!(events.first(), Some(ExecutionEvent::Admitted)));
            for (index, event) in events[1..=count].iter().enumerate() {
                let ExecutionEvent::Progress(progress) = event else {
                    panic!(
                        "progress was skipped or terminal notification arrived early: {event:?}"
                    );
                };
                assert_eq!(progress.accumulated, index + 1);
            }
            assert!(matches!(events.last(), Some(ExecutionEvent::Completed)));
            Ok(())
        })
        .await
}
