use eyre::Context;
use eyre::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::OwnedMutexGuard;

/// Owns exclusive artifact I/O until its blocking worker and finalization finish.
/// Dropping an awaiting future cannot release a still-running worker's lock.
#[derive(Debug)]
pub(crate) struct ArtifactIoGuard(OwnedMutexGuard<()>);

impl ArtifactIoGuard {
    pub async fn acquire(gate: &Arc<Mutex<()>>) -> Self {
        Self(gate.clone().lock_owned().await)
    }

    /// Return ownership even on ordinary failure so callers can finalize state
    /// under the same lock. This guard never clears publication quarantine.
    pub async fn run_blocking<T: Send + 'static>(
        self,
        operation: impl FnOnce() -> Result<T> + Send + 'static,
    ) -> Result<(Self, Result<T>)> {
        let gate = self.0;
        tokio::task::spawn_blocking(move || {
            let result = operation();
            (Self(gate), result)
        })
        .await
        .wrap_err("artifact filesystem worker failed")
    }
}

#[cfg(test)]
mod tests {
    use super::ArtifactIoGuard;
    use std::fs;
    use std::sync::Arc;
    use std::sync::mpsc;
    use tokio::sync::Mutex;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn dropped_awaiter_cannot_release_gate_before_its_worker_finishes_writing()
    -> eyre::Result<()> {
        let temporary = tempfile::tempdir()?;
        let selection = temporary.path().join("selection.json");
        let gate = Arc::new(Mutex::new(()));
        let owned = ArtifactIoGuard::acquire(&gate).await;
        let (started, ready) = oneshot::channel();
        let (release, blocked) = mpsc::channel();
        let old_selection = selection.clone();
        let awaiting = tokio::spawn(async move {
            owned
                .run_blocking(move || {
                    let _ = started.send(());
                    blocked.recv()?;
                    fs::write(old_selection, b"old selection")?;
                    Ok(())
                })
                .await
        });
        ready.await?;
        awaiting.abort();
        assert!(awaiting.await.unwrap_err().is_cancelled());
        // Aborting the async task must not unlock a still-running filesystem
        // worker. Otherwise the following newer write could be overwritten.
        assert!(gate.try_lock().is_err());
        release.send(())?;
        let owned = ArtifactIoGuard::acquire(&gate).await;
        assert_eq!(fs::read(&selection)?, b"old selection");
        let new_selection = selection.clone();
        let (_gate, written) = owned
            .run_blocking(move || {
                fs::write(new_selection, b"new selection")?;
                Ok(())
            })
            .await?;
        written?;
        assert_eq!(fs::read(selection)?, b"new selection");
        Ok(())
    }
}
