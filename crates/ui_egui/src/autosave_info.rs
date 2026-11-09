use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_config::Config;
use eyre::Result;
use std::any::type_name;
use std::time::Duration;
use std::time::Instant;
use teamy_cancellation::CancellationScope;
use tokio::runtime::Handle;
use tracing::debug;

#[derive(Debug)]
pub struct AutoSaveBehaviour<T: Config> {
    pub interval: Duration,
    pub last_save: Option<(Instant, T)>,
}
impl<T> AutoSaveBehaviour<T>
where
    T: Config,
{
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_save: None,
        }
    }
    pub fn apply(
        &mut self,
        config: &T,
        work_tracker: &AppWorkTracker,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let (last_save, last_save_copy) = self
            .last_save
            .get_or_insert_with(|| (Instant::now(), config.clone()));
        if last_save.elapsed() > self.interval {
            if last_save_copy != config {
                debug!("Detected config change during auto save, scheduling write");
                let config_copy = config.clone();
                let description = format!("Autosave for {}", type_name::<T>());
                work_tracker.spawn(
                    cancellation,
                    &Handle::current(),
                    description,
                    async move |scope| {
                        scope
                            .deferred("finish configuration autosave", async move |_protected| {
                                config_copy.save().await
                            })
                            .await
                    },
                )?;
                // This records admission, not completed persistence. Final
                // settings are flushed by the GUI owner after all jobs drain.
                *last_save_copy = config.clone();
            }
            *last_save = Instant::now();
        }
        Ok(())
    }
}
impl<T> Default for AutoSaveBehaviour<T>
where
    T: Config,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::AutoSaveBehaviour;
    use async_trait::async_trait;
    use cloud_terrastodon_command::app_work::AppWorkTracker;
    use cloud_terrastodon_config::Config;
    use facet::Facet;
    use std::future::poll_fn;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::task::Poll;
    use std::time::Duration;
    use std::time::Instant;
    use teamy_cancellation::CancellationToken;
    use tokio::sync::oneshot;
    use tokio::time::timeout;

    static SAVE_SIGNALS: Mutex<Option<(oneshot::Sender<()>, oneshot::Receiver<()>)>> =
        Mutex::new(None);
    static SAVED_REVISION: AtomicUsize = AtomicUsize::new(0);

    #[derive(Clone, Debug, Default, PartialEq, Facet)]
    struct SyntheticConfig {
        revision: usize,
    }

    #[async_trait]
    impl Config for SyntheticConfig {
        const FILE_SLUG: &'static str = "synthetic-autosave";

        // Exercise the real autosave scheduling without touching configuration
        // directories, files, credentials, or service state.
        async fn save(&self) -> eyre::Result<()> {
            let (started, released) = SAVE_SIGNALS.lock().unwrap().take().unwrap();
            started.send(()).unwrap();
            released.await.unwrap();
            SAVED_REVISION.store(self.revision, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn admitted_save_finishes_and_rejected_work_does_not_advance_snapshot() {
        timeout(Duration::from_secs(5), async {
            let token = CancellationToken::new();
            token
                .with_async_scope(async |scope| {
                    let tracker = AppWorkTracker::new();
                    let mut autosave = AutoSaveBehaviour {
                        interval: Duration::ZERO,
                        last_save: Some((
                            Instant::now() - Duration::from_secs(1),
                            SyntheticConfig { revision: 1 },
                        )),
                    };
                    let (started, ready) = oneshot::channel();
                    let (release, released) = oneshot::channel();
                    *SAVE_SIGNALS.lock().unwrap() = Some((started, released));

                    autosave.apply(&SyntheticConfig { revision: 2 }, &tracker, &scope)?;
                    ready.await.unwrap();
                    token.request_cancel("close synthetic GUI");

                    let finish = tracker.clone().finish(&scope);
                    tokio::pin!(finish);
                    poll_fn(|cx| {
                        assert!(finish.as_mut().poll(cx).is_pending());
                        Poll::Ready(())
                    })
                    .await;
                    assert_eq!(SAVED_REVISION.load(Ordering::SeqCst), 0);

                    let unchanged_timestamp = Instant::now() - Duration::from_secs(1);
                    autosave.last_save.as_mut().unwrap().0 = unchanged_timestamp;
                    assert!(
                        autosave
                            .apply(&SyntheticConfig { revision: 3 }, &tracker, &scope)
                            .is_err()
                    );
                    assert_eq!(
                        autosave.last_save,
                        Some((unchanged_timestamp, SyntheticConfig { revision: 2 }))
                    );

                    release.send(()).unwrap();
                    finish.await?;
                    assert_eq!(SAVED_REVISION.load(Ordering::SeqCst), 2);
                    eyre::Ok(())
                })
                .await
                .unwrap();
        })
        .await
        .expect("autosave shutdown must retain an admitted write until completion");
    }
}
