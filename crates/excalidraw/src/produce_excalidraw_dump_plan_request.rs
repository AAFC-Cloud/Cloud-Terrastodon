use crate::ExcalidrawScene;
use crate::ExcalidrawSharedFile;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_registry::DiscoveryPlan;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Export a discovery plan's dependency graph as an editable Excalidraw drawing.
///
/// The object explorer supplies `plan` and `filename` through its ordinary value
/// picker and field editor. Invocation lays out requests, required input shapes,
/// and outputs; it does not execute the plan or acquire credentials.
///
/// The destination's parent directory must exist. An existing drawing is replaced,
/// including its canvas edits, using the cooperative lock and atomic publication
/// protocol. Element revisions and deletion markers allow an open editor to see
/// the replacement. Waiting for another writer is cancellable; once publication
/// starts, invocation retains the blocking worker until it finishes.
#[derive(Clone, Debug, Facet)]
pub struct ProduceExcalidrawDumpPlanRequest {
    pub plan: DiscoveryPlan,
    pub filename: PathBuf,
}

impl ExecutableRequest for ProduceExcalidrawDumpPlanRequest {
    type Output = PathBuf;
    type Error = eyre::Report;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
        Self::Output: 'a,
        Self::Error: 'a,
    {
        Box::pin(async move {
            // Layout and filesystem access must not occupy the explorer's async
            // worker. Preparation is read-only, so cancellation may abandon it.
            let (file, scene) = tokio::task::spawn_blocking(move || -> Result<_> {
                let file = ExcalidrawSharedFile::new(&self.filename).wrap_err_with(|| {
                    format!("Opening Excalidraw destination {}", self.filename.display())
                })?;
                let scene = ExcalidrawScene::from_discovery_plan(&self.plan)?;
                Ok((file, Arc::new(scene)))
            })
            .await
            .wrap_err("Preparing the discovery plan drawing failed")??;

            loop {
                let writing = file.clone();
                let rendering = Arc::clone(&scene);
                let result = executor
                    .cancellation()
                    .deferred(
                        "publishing the discovery plan drawing",
                        |_scope| async move {
                            // Acquire protection before spawning: otherwise cancellation
                            // could drop this future while a queued worker starts writing.
                            tokio::task::spawn_blocking(move || {
                                writing.update(|current| {
                                    Ok(Some(rendering.replacement_json(current)?))
                                })
                            })
                            .await
                            .wrap_err("Discovery plan drawing worker failed")?
                        },
                    )
                    .await;
                match result {
                    Ok(_) => return Ok(file.path().to_owned()),
                    Err(error)
                        if error
                            .downcast_ref::<io::Error>()
                            .is_some_and(|error| error.kind() == io::ErrorKind::WouldBlock) =>
                    {
                        // Release the deferral while waiting. Never remove a lock
                        // owned by the editor or another CT process.
                        executor
                            .cancellation()
                            .cancellable_scope(|_scope| async {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                eyre::Ok(())
                            })
                            .await?;
                    }
                    Err(error) => {
                        return Err(error).wrap_err_with(|| {
                            format!("Publishing discovery plan to {}", file.path().display())
                        });
                    }
                }
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(ProduceExcalidrawDumpPlanRequest);
cloud_terrastodon_registry::register_arbitrary!(PathBuf);
cloud_terrastodon_registry_execution::register_executable_request!(ProduceExcalidrawDumpPlanRequest => PathBuf, effects = [Write]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::excalidraw_file_lock::ExcalidrawFileLock;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
    use std::fs;
    use teamy_cancellation::CancellationReportExt;
    use teamy_cancellation::CancellationScope;
    use teamy_cancellation::CancellationToken;

    fn executor(cancellation: CancellationScope) -> Result<CloudTerrastodonExecutor> {
        Ok(CloudTerrastodonExecutor::new(
            WorkCache::temporary()?,
            Arc::new(AnonymousAuthenticationProvider),
            cancellation,
        ))
    }

    fn request(filename: PathBuf) -> ProduceExcalidrawDumpPlanRequest {
        ProduceExcalidrawDumpPlanRequest {
            plan: DiscoveryPlan {
                nodes: vec![],
                edges: vec![],
                diagnostics: vec![],
            },
            filename,
        }
    }

    #[tokio::test]
    async fn malformed_destination_is_preserved_and_export_lock_is_released() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let filename = directory.path().join("plan.excalidraw");
        fs::write(&filename, "incomplete existing drawing")?;
        CancellationToken::new()
            .with_async_scope(|scope| async {
                let error = request(filename.clone())
                    .execute(&executor(scope)?)
                    .await
                    .unwrap_err();
                assert!(error.to_string().contains("Publishing discovery plan"));
                assert_eq!(
                    fs::read_to_string(&filename)?,
                    "incomplete existing drawing"
                );
                assert!(!directory.path().join("plan.excalidraw.sync-lock").exists());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn export_retries_when_the_editor_releases_its_lock() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let filename = directory.path().join("plan.excalidraw");
        let lock_path = directory.path().join("plan.excalidraw.sync-lock");
        let editor_lock = ExcalidrawFileLock::acquire(lock_path.clone())?;
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let executor = executor(scope)?;
                let mut export =
                    Box::pin(request(filename.clone()).execute(&executor).into_future());
                assert!(
                    tokio::time::timeout(Duration::from_millis(200), &mut export)
                        .await
                        .is_err(),
                    "an active editor lock keeps publication pending"
                );
                assert!(!filename.exists());
                drop(editor_lock);
                let written = tokio::time::timeout(Duration::from_secs(5), export).await??;
                assert_eq!(written, fs::canonicalize(&filename)?);
                assert!(fs::read_to_string(filename)?.contains("offline discovery plan"));
                assert!(!lock_path.exists());
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn cancelling_a_waiting_export_preserves_the_editor_lock_and_drawing() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let filename = directory.path().join("plan.excalidraw");
        let before = ExcalidrawScene::from_objects("original view", &[])?.to_json()?;
        fs::write(&filename, &before)?;
        let lock_path = directory.path().join("plan.excalidraw.sync-lock");
        let _editor_lock = ExcalidrawFileLock::acquire(lock_path.clone())?;
        let lock_before = fs::read_to_string(&lock_path)?;
        let cancellation = CancellationToken::new();
        cancellation
            .with_async_scope(|scope| async {
                let executor = executor(scope)?;
                let mut export =
                    Box::pin(request(filename.clone()).execute(&executor).into_future());
                assert!(
                    tokio::time::timeout(Duration::from_millis(200), &mut export)
                        .await
                        .is_err()
                );
                cancellation.request_cancel("test cancelled while editor owns the drawing");
                let error = tokio::time::timeout(Duration::from_secs(5), export)
                    .await?
                    .unwrap_err();
                assert!(error.is_cancellation(), "{error:?}");
                assert_eq!(fs::read_to_string(&filename)?, before);
                assert_eq!(fs::read_to_string(&lock_path)?, lock_before);
                Ok(())
            })
            .await
    }
}
