use cloud_terrastodon_command::CancellationReportExt;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_excalidraw::ExcalidrawScene;
use cloud_terrastodon_excalidraw::ExcalidrawSharedFile;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::task::JoinHandle;

/// Owns one disposable diagram destination and its active render. Keeping the
/// task here makes terminal shutdown await any atomic publication already owed.
pub(super) struct ExcalidrawRenderSession {
    path: Option<PathBuf>,
    cancellation: CancellationScope,
    task: Option<(CancellationToken, JoinHandle<Result<PathBuf>>)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn closing_a_render_waiting_on_another_writer_preserves_its_file_and_lock() -> Result<()>
    {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("render.excalidraw");
        let original = ExcalidrawScene::from_objects("original view", &[]).to_json()?;
        std::fs::write(&path, &original)?;
        let lock = directory.path().join("render.excalidraw.sync-lock");
        std::fs::write(&lock, "another writer owns this")?;
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let mut renderer = ExcalidrawRenderSession::new(Some(path.clone()), &scope);
                let started = Arc::new(AtomicBool::new(false));
                let observed = started.clone();
                renderer.start(move |_| async move {
                    observed.store(true, Ordering::SeqCst);
                    Ok(ExcalidrawScene::from_objects("replacement view", &[]))
                })?;
                tokio::time::timeout(Duration::from_secs(2), async {
                    while !started.load(Ordering::SeqCst) {
                        tokio::task::yield_now().await;
                    }
                })
                .await?;
                tokio::time::sleep(Duration::from_millis(50)).await;
                assert!(
                    renderer.take_result().await.is_none(),
                    "the other writer still holds the file"
                );
                tokio::time::timeout(Duration::from_secs(2), renderer.close()).await?;
                assert_eq!(std::fs::read_to_string(path)?, original);
                assert_eq!(std::fs::read_to_string(lock)?, "another writer owns this");
                Ok(())
            })
            .await
    }
}

impl ExcalidrawRenderSession {
    pub(super) fn is_rendering(&self) -> bool {
        self.task.is_some()
    }
    pub(super) fn new(path: Option<PathBuf>, cancellation: &CancellationScope) -> Self {
        Self {
            path,
            cancellation: cancellation.clone(),
            task: None,
        }
    }

    pub(super) fn start<F>(
        &mut self,
        render: impl FnOnce(CancellationScope) -> F + Send + 'static,
    ) -> Result<PathBuf>
    where
        F: Future<Output = Result<ExcalidrawScene>> + Send + 'static,
    {
        ensure!(
            self.task.is_none(),
            "An Excalidraw render is already in progress"
        );
        if self.path.is_none() {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
            self.path = Some(std::env::current_dir()?.join(format!(
                "object-explorer-{}-{timestamp:x}.excalidraw",
                std::process::id()
            )));
        }
        let path = self.path.as_ref().expect("destination initialized").clone();
        let destination = path.clone();
        let (token, future) = self
            .cancellation
            .with_async_child_scope(move |scope| async move {
                let scene = render(scope.clone()).await?;
                scope.bail_if_cancelled()?;
                let file =
                    tokio::task::spawn_blocking(move || ExcalidrawSharedFile::new(destination))
                        .await??;
                loop {
                    scope.bail_if_cancelled()?;
                    let writing = file.clone();
                    let rendering = scene.clone();
                    let protected = scope.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        protected.with_deferral("publishing the Excalidraw object view", |_| {
                            writing.update(|current| Ok(Some(rendering.replacement_json(current)?)))
                        })
                    })
                    .await
                    .wrap_err("Excalidraw render worker failed")?;
                    match result {
                        Ok(_) => return Ok(file.path().to_owned()),
                        Err(error)
                            if error
                                .downcast_ref::<io::Error>()
                                .is_some_and(|error| error.kind() == io::ErrorKind::WouldBlock) =>
                        {
                            tokio::select! {
                                () = scope.cancelled() => { scope.bail_if_cancelled()?; }
                                () = tokio::time::sleep(Duration::from_millis(100)) => {}
                            }
                        }
                        Err(error) => {
                            return Err(error)
                                .wrap_err_with(|| format!("Publishing {}", file.path().display()));
                        }
                    }
                }
            });
        self.task = Some((token, tokio::spawn(future)));
        Ok(path)
    }

    pub(super) async fn take_result(&mut self) -> Option<Result<PathBuf>> {
        if !self
            .task
            .as_ref()
            .is_some_and(|(_, task)| task.is_finished())
        {
            return None;
        }
        let (_, task) = self.task.take().expect("finished task");
        Some(
            task.await
                .wrap_err("Excalidraw render task failed")
                .and_then(|result| result),
        )
    }

    pub(super) async fn close(mut self) {
        if let Some((token, task)) = self.task.take() {
            token.request_cancel("object explorer closed");
            match task.await {
                Ok(Ok(_)) => {}
                Ok(Err(error)) if error.is_cancellation() => {
                    tracing::debug!(%error, "Excalidraw render ended during shutdown")
                }
                Ok(Err(error)) => {
                    tracing::warn!(%error, "Excalidraw render failed during shutdown")
                }
                Err(error) => {
                    tracing::warn!(%error, "Excalidraw render task failed during shutdown")
                }
            }
        }
    }
}
