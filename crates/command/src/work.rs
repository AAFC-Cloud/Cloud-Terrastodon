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
    pub async fn join(mut self) -> eyre::Result<Vec<T>> {
        let mut rtn = Vec::new();
        let count = self.join_set.len();
        while let Some(x) = self.join_set.join_next().await {
            info!("{}, {} tasks remain", self.description, self.join_set.len());
            rtn.push(x??);
        }
        let end = Instant::now();
        match self.start {
            Some(start) => {
                let duration = end.duration_since(start);
                let duration = humantime::format_duration(duration);
                info!("Finished {} tasks in {} seconds", count, duration);
            }
            None => {
                info!("Finished {} tasks", self.description);
            }
        }
        Ok(rtn)
    }
}

#[cfg(test)]
mod test {
    use crate::ParallelFallibleWorkQueue;

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
