use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::Instant;
use tracing::info;

pub struct ParallelFallibleWorkQueue<T> {
    join_set: JoinSet<eyre::Result<T>>,
    rate_limit: Arc<Semaphore>,
    description: String,
    start: Option<Instant>,
}
impl<T> ParallelFallibleWorkQueue<T>
where
    T: Send + 'static,
{
    pub fn new(description: impl Into<String>, rate_limit: usize) -> Self {
        let description = description.into();
        ParallelFallibleWorkQueue {
            join_set: JoinSet::new(),
            rate_limit: Arc::new(Semaphore::new(rate_limit)),
            description,
            start: None,
        }
    }

    pub fn enqueue(
        &mut self,
        task: impl Future<Output = eyre::Result<T>> + Send + 'static,
    ) -> &mut Self
    where
        T: Send + 'static,
    {
        if self.start.is_none() {
            self.start = Some(Instant::now());
        }
        let rate_limt = self.rate_limit.clone();
        self.join_set.spawn(async move {
            let permit = rate_limt.acquire().await;
            let rtn = task.await?;
            drop(permit);
            Ok(rtn)
        });
        self
    }
    /// Wait for every spawned job before returning the first failure.
    ///
    /// A sibling may be completing a mutation or its required artifacts under
    /// a cancellation deferral. Returning early would drop the JoinSet and
    /// forcibly abort that work. Dropping this join future still aborts jobs;
    /// owners must keep polling it while required completion is in progress.
    pub async fn join(mut self) -> eyre::Result<Vec<T>> {
        let mut rtn = Vec::new();
        let mut first_error = None;
        let count = self.join_set.len();
        while let Some(x) = self.join_set.join_next().await {
            info!("{}, {} tasks remain", self.description, self.join_set.len());
            match x {
                Ok(Ok(value)) => rtn.push(value),
                Ok(Err(error)) => {
                    first_error.get_or_insert(error);
                }
                Err(error) => {
                    first_error.get_or_insert_with(|| eyre::Report::from(error));
                }
            }
        }
        let end = Instant::now();
        match self.start {
            Some(start) => {
                let duration = end.duration_since(start);
                let duration = humantime::format_duration(duration);
                info!("Finished {} tasks in {}", count, duration);
            }
            None => {
                info!("Finished {} tasks", self.description);
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(rtn),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CancellationError;
    use crate::CancellationToken;
    use crate::ParallelFallibleWorkQueue;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::task::Poll;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn cancellation_failure_waits_for_a_sibling_to_finish_required_work() {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let finished = Arc::new(AtomicBool::new(false));
                let task_finished = Arc::clone(&finished);
                let (started, started_receiver) = oneshot::channel();
                let (may_finish, may_finish_receiver) = oneshot::channel();
                let (failed, failed_receiver) = oneshot::channel();
                let mut queue = ParallelFallibleWorkQueue::new("synthetic completion", 2);
                queue.enqueue(
                    cancellation.cancellable_scope(move |completing| async move {
                        completing
                            .deferred("finishing synthetic mutation", |_completion| async move {
                                started.send(()).unwrap();
                                may_finish_receiver.await?;
                                task_finished.store(true, Ordering::SeqCst);
                                Ok(())
                            })
                            .await
                    }),
                );
                queue.enqueue(cancellation.cancellable_scope(move |failing| async move {
                    started_receiver.await?;
                    failing.request_cancel("synthetic sibling cancellation");
                    let error = failing.bail_if_cancelled().unwrap_err();
                    failed.send(()).unwrap();
                    Err(error.into())
                }));

                // The current-thread runtime finishes the failing job's poll before
                // resuming this receiver, so its error is ready before join is polled.
                failed_receiver.await.unwrap();
                let mut join = Box::pin(queue.join());
                std::future::poll_fn(|context| {
                    assert!(
                        join.as_mut().poll(context).is_pending(),
                        "join must retain the error while required sibling work is pending",
                    );
                    Poll::Ready(())
                })
                .await;
                assert!(!finished.load(Ordering::SeqCst));
                may_finish.send(()).unwrap();

                let error = join.await.unwrap_err();
                assert!(finished.load(Ordering::SeqCst));
                assert_eq!(
                    error.downcast_ref::<CancellationError>().unwrap().reason,
                    "synthetic sibling cancellation",
                );
                Ok::<_, eyre::Report>(())
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let debug = false;
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(
                        match debug {
                            true => tracing::level_filters::LevelFilter::DEBUG,
                            false => tracing::level_filters::LevelFilter::INFO,
                        }
                        .into(),
                    )
                    .from_env_lossy(),
            )
            .with_file(true)
            .with_target(false)
            .with_line_number(true)
            .without_time()
            .init();

        let mut work = ParallelFallibleWorkQueue::new("Test Work", 10);
        for i in 0..100 {
            work.enqueue(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(i)
            });
        }
        let rtn = work.join().await?;
        assert_eq!(rtn, (0..100).collect::<Vec<usize>>());
        Ok(())
    }
}
