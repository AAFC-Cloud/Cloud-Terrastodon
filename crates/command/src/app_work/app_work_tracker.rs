use super::TrackedAppWork;
use super::WorkHandle;
use eyre::Result;
use eyre::eyre;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use teamy_cancellation::CancellationScope;
use tokio::runtime::Handle;
use tracing::info;

/// Owns admitted background tasks until their outcomes have been observed.
///
/// Shutdown closes admission for every clone, requests each task's linked
/// cancellation signal, and drains all tasks, including after a task panics.
/// Call `finish` while the supplied parent execution scope is still alive.
#[derive(Clone, Debug)]
pub struct AppWorkTracker {
    // The lock makes admission and closing atomic. None means shutdown owns
    // the tasks; clones cannot enqueue more work behind the drain.
    work: Arc<Mutex<Option<TrackedAppWork>>>,
}

impl Default for AppWorkTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl AppWorkTracker {
    pub fn new() -> Self {
        Self {
            work: Arc::new(Mutex::new(Some(TrackedAppWork::default()))),
        }
    }

    /// Admit a task on the supplied runtime and retain it until its outcome is observed.
    ///
    /// The factory receives an automatically cancellable child of `cancellation`.
    /// Deferrals within that child protect required completion during shutdown.
    /// Shutdown closes scheduling admission. Cancellation is checked against the
    /// supplied scope before scheduling and again by the child on its first
    /// poll; a cancelled child never invokes the factory. Existing deferrals
    /// retain their normal scope semantics. Keep the parent scope alive until
    /// [`Self::finish`] has drained every admitted task.
    #[track_caller]
    pub fn spawn<C, F>(
        &self,
        cancellation: &CancellationScope,
        runtime: &Handle,
        description: String,
        factory: C,
    ) -> Result<()>
    where
        C: FnOnce(CancellationScope) -> F + Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let mut work = self
            .work
            .lock()
            .map_err(|error| eyre!("Failed to lock background work: {error}"))?;
        let work = work
            .as_mut()
            .ok_or_else(|| eyre!("Cannot enqueue background work after shutdown has begun"))?;
        // Scheduling owns admission; the factory itself receives the child
        // scope and needs no redundant entry cancellation check.
        cancellation.bail_if_cancelled()?;
        let (cancellation, future) = cancellation.cancellable_child_scope(factory);
        let task = work.tasks.spawn_on(future, runtime);
        work.handles.insert(
            task.id(),
            WorkHandle {
                description,
                cancellation,
            },
        );
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        let work = self
            .work
            .lock()
            .map_err(|error| eyre!("Failed to lock background work: {error}"))?;
        Ok(work.as_ref().map_or(0, |work| work.tasks.len()))
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Reap completed tasks without losing errors or panics before shutdown.
    pub fn prune(&self) -> Result<()> {
        let mut work = self
            .work
            .lock()
            .map_err(|error| eyre!("Failed to lock background work: {error}"))?;
        if let Some(work) = work.as_mut() {
            while let Some(result) = work.tasks.try_join_next_with_id() {
                work.record_completion(result);
            }
        }
        Ok(())
    }

    /// Cancel pending work and await every admitted task before reporting errors.
    ///
    /// A task's own deferrals protect required completion. The drain is itself
    /// required cleanup so an automatic ancestor cannot drop it mid-shutdown.
    pub async fn finish(self, cancellation: &CancellationScope) -> Result<()> {
        cancellation
            .finish_required("drain admitted background work", async move |_cleanup| {
                let work = self
                    .work
                    .lock()
                    .map_err(|error| eyre!("Failed to lock background work: {error}"))?
                    .take();
                let Some(mut work) = work else {
                    return Ok(());
                };

                for handle in work.handles.values() {
                    handle
                        .cancellation
                        .request_cancel("background work shutdown");
                }
                info!(remaining = work.tasks.len(), "Draining background work");
                while let Some(result) = work.tasks.join_next_with_id().await {
                    work.record_completion(result);
                }
                match work.first_error {
                    Some(error) => Err(error),
                    None => Ok(()),
                }
            })
            .await
    }
}
