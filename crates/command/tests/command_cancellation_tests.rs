#![cfg(test)]

use facet::Facet;
mod support;

use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_command::CancellationError;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::RetryBehaviour;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use std::cell::RefCell;
use std::future::Future;
use std::future::poll_fn;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::panic::Location;
use std::path::Path;
use std::path::PathBuf;
use std::task::Poll;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use tokio::time::timeout;

const LIMIT: Duration = Duration::from_secs(15);

// Only the fixture directory is accessed. Its release gate keeps the child from
// completing until the test has requested cancellation and observed the result.
const CHILD_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
[System.IO.File]::WriteAllText('started', 'ready')
$deadline = [DateTime]::UtcNow.AddSeconds(30)
while (-not [System.IO.File]::Exists('release')) {
    if ([DateTime]::UtcNow -ge $deadline) { throw 'fixture release timed out' }
    [System.Threading.Thread]::Sleep(5)
}
[System.IO.File]::WriteAllText('completed', 'completed')
[Console]::WriteLine('{"value":"completed"}')
"#;

#[derive(Debug, Facet)]
struct ChildOutput {
    value: String,
}

// Release a waiting child even if an assertion or timeout ends a test early.
// CommandBuilder's kill-on-drop remains responsible for abandoning the process.
struct ReleaseChildOnDrop(PathBuf);

impl Drop for ReleaseChildOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.0, b"release");
    }
}

fn fixture_command(directory: &Path) -> Result<CommandBuilder> {
    ensure!(
        directory.is_absolute(),
        "fixture directory must be absolute"
    );

    let mut command = CommandBuilder::new(CommandKind::Pwsh);
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        CHILD_SCRIPT,
    ]);
    command.use_run_dir(directory);
    command.use_retry_behaviour(RetryBehaviour::Fail);
    command.use_timeout(LIMIT);
    Ok(command)
}

async fn wait_for_file(path: &Path) -> Result<()> {
    timeout(LIMIT, async {
        while !fs::try_exists(path).await? {
            sleep(Duration::from_millis(5)).await;
        }
        Ok(())
    })
    .await
    .wrap_err_with(|| format!("waiting for fixture file {}", path.display()))?
}

async fn assert_fixture_artifacts(directory: &Path) -> Result<()> {
    let outputs = support::files_named(directory, "stdout.json")?;
    assert_eq!(outputs.len(), 1);
    assert_eq!(
        fs::read_to_string(&outputs[0]).await?.trim(),
        r#"{"value":"completed"}"#
    );
    assert_eq!(
        fs::read_to_string(outputs[0].with_file_name("status.txt")).await?,
        "0"
    );
    Ok(())
}

#[tokio::test]
async fn default_command_finishes_process_mapping_and_artifacts_after_cancellation() -> Result<()> {
    let token = CancellationToken::new();
    let requester = token.clone();
    token
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let executor = support::executor_in(cancellation.clone(), &directory.path().join("artifacts"))?;
            let release = ReleaseChildOnDrop(directory.path().join("release"));
            let started = directory.path().join("started");
            let command = fixture_command(directory.path())?;
            let mapped = std::cell::Cell::new(false);
            let mut running = Box::pin(executor.cancellable_scope(async |child| {
                command
                    .run_with_artifacts_and_mapper::<ChildOutput, String, _>(
                        |output| {
                            mapped.set(true);
                            Ok(output.value)
                        },
                        &child,
                    )
                    .await
            }));

            tokio::select! {
                result = &mut running => { bail!("command finished before fixture release: {result:?}"); },
                result = wait_for_file(&started) => result?,
            }
            requester.request_cancel("cancel while the submitted command is running");
            poll_fn(|context| match Future::poll(running.as_mut(), context) {
                Poll::Pending => Poll::Ready(()),
                Poll::Ready(result) => panic!("default command abandoned its child: {result:?}"),
            })
            .await;
            fs::write(&release.0, b"release").await?;

            assert_eq!(timeout(LIMIT, running).await??, "completed");
            assert!(mapped.get());
            assert!(fs::try_exists(directory.path().join("completed")).await?);
            assert_fixture_artifacts(&directory.path().join("artifacts")).await?;
            Ok(())
        })
        .await
}

#[tokio::test]
async fn plain_default_command_finishes_process_and_mapping_after_cancellation() -> Result<()> {
    let token = CancellationToken::new();
    let requester = token.clone();
    token
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let release = ReleaseChildOnDrop(directory.path().join("release"));
            let started = directory.path().join("started");
            let command = fixture_command(directory.path())?;
            let mapped = std::cell::Cell::new(false);
            let mut running = Box::pin(cancellation.cancellable_scope(async |child| {
                command
                    .run_with_mapper::<ChildOutput, String, _>(
                        |output| {
                            mapped.set(true);
                            Ok(output.value)
                        },
                        &child,
                    )
                    .await
            }));
            tokio::select! {
                result = &mut running => { bail!("command finished before fixture release: {result:?}"); },
                result = wait_for_file(&started) => result?,
            }
            requester.request_cancel("cancel the plain command while it owns completion");
            poll_fn(|context| match Future::poll(running.as_mut(), context) {
                Poll::Pending => Poll::Ready(()),
                Poll::Ready(result) => panic!("plain command abandoned its child: {result:?}"),
            })
            .await;
            fs::write(&release.0, b"release").await?;
            assert_eq!(timeout(LIMIT, running).await??, "completed");
            assert!(mapped.get());
            assert!(fs::try_exists(directory.path().join("completed")).await?);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancellation_safe_command_stops_waiting_for_a_pending_child() -> Result<()> {
    let token = CancellationToken::new();
    let requester = token.clone();
    token
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let _release = ReleaseChildOnDrop(directory.path().join("release"));
            let started = directory.path().join("started");
            let mut command = fixture_command(directory.path())?;
            command.cancellation_safe();
            let mut running = Box::pin(command.run_raw(&cancellation));

            tokio::select! {
                result = &mut running => { bail!("command finished before fixture release: {result:?}"); },
                result = wait_for_file(&started) => result?,
            }
            requester.request_cancel("cancel an explicitly safe pending command");
            let error = timeout(LIMIT, running)
                .await?
                .expect_err("safe commands must observe cancellation while their child is pending");
            assert!(error.downcast_ref::<CancellationError>().is_some(), "{error:#}");
            assert!(!fs::try_exists(directory.path().join("completed")).await?);
            assert!(!fs::try_exists(directory.path().join("release")).await?);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn already_cancelled_commands_never_start_in_either_policy() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let _release = ReleaseChildOnDrop(directory.path().join("release"));
            let mut command = fixture_command(directory.path())?;
            cancellation.request_cancel("cancel before command admission");

            let error = timeout(LIMIT, command.run_raw(&cancellation))
                .await?
                .expect_err("default commands must reject preexisting cancellation");
            assert!(
                error.downcast_ref::<CancellationError>().is_some(),
                "{error:#}"
            );
            command.cancellation_safe();
            let error = timeout(LIMIT, command.run_raw(&cancellation))
                .await?
                .expect_err("safe commands must reject preexisting cancellation");
            assert!(
                error.downcast_ref::<CancellationError>().is_some(),
                "{error:#}"
            );
            assert!(!fs::try_exists(directory.path().join("started")).await?);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancellation_safe_command_respects_a_protected_caller() -> Result<()> {
    let token = CancellationToken::new();
    let requester = token.clone();
    token
        .with_async_scope(|cancellation| async move {
            cancellation
                .deferred(
                    "complete the caller's required local operation",
                    |protected| async move {
                    let directory = tempfile::tempdir()?;
                    let release = ReleaseChildOnDrop(directory.path().join("release"));
                    let started = directory.path().join("started");
                    let mut command = fixture_command(directory.path())?;
                    command.cancellation_safe();
                    let mut running = Box::pin(protected.cancellable_scope(move |child| async move {
                        command.run::<ChildOutput>(&child).await
                    }));

                    tokio::select! {
                        result = &mut running => { bail!("command finished before fixture release: {result:?}"); },
                        result = wait_for_file(&started) => result?,
                    }
                    requester.request_cancel("cancel while the caller owns required completion");
                    poll_fn(|context| match Future::poll(running.as_mut(), context) {
                        Poll::Pending => Poll::Ready(()),
                        Poll::Ready(result) => panic!("safe command bypassed its protected caller: {result:?}"),
                    })
                    .await;
                    fs::write(&release.0, b"release").await?;
                    assert_eq!(timeout(LIMIT, running).await??.value, "completed");
                    assert!(fs::try_exists(directory.path().join("completed")).await?);
                    Ok(())
                    },
                )
                .await
        })
        .await
}

#[tokio::test]
async fn polite_cache_hits_return_without_delaying_or_restarting_the_child() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let executor =
                support::executor_in(cancellation.clone(), &directory.path().join("artifacts"))?;
            let release = ReleaseChildOnDrop(directory.path().join("release"));
            let mut command = fixture_command(directory.path())?;
            command.cache(support::key(Duration::MAX));
            struct ChildOperation;
            let owner = CacheOperationOwner::of::<ChildOperation>();
            fs::write(&release.0, b"release").await?;
            let output =
                timeout(LIMIT, command.run_cached::<ChildOutput>(&executor, owner)).await??;
            assert_eq!(output.value, "completed");

            fs::remove_file(directory.path().join("started")).await?;
            fs::remove_file(directory.path().join("completed")).await?;

            // A cache hit must bypass a delay longer than this bounded wait.
            let cached = timeout(
                LIMIT,
                command.run_cached_polite::<ChildOutput>(Duration::from_secs(30), &executor, owner),
            )
            .await??;
            assert_eq!(cached.value, "completed");
            assert!(!fs::try_exists(directory.path().join("started")).await?);
            assert!(!fs::try_exists(directory.path().join("completed")).await?);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn cancellation_safe_polite_wait_can_stop_before_starting_the_child() -> Result<()> {
    let token = CancellationToken::new();
    let requester = token.clone();
    token
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let _release = ReleaseChildOnDrop(directory.path().join("release"));
            let mut command = fixture_command(directory.path())?;
            command.cancellation_safe();
            let mut running =
                Box::pin(command.run_raw_polite(Duration::from_secs(30), &cancellation));
            poll_fn(|context| match Future::poll(running.as_mut(), context) {
                Poll::Pending => Poll::Ready(()),
                Poll::Ready(result) => {
                    panic!("uncached polite command completed early: {result:?}")
                }
            })
            .await;
            requester.request_cancel("stop the admitted polite wait");

            let error = timeout(LIMIT, running)
                .await?
                .expect_err("safe polite waits must respond to cancellation");
            assert!(
                error.downcast_ref::<CancellationError>().is_some(),
                "{error:#}"
            );
            assert!(!fs::try_exists(directory.path().join("started")).await?);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn rejected_output_preserves_source_caller_and_failure_artifacts_after_cancellation()
-> Result<()> {
    // Both public decoding contracts own source errors and failure artifacts.
    for validate in [false, true] {
        let token = CancellationToken::new();
        let requester = token.clone();
        token
            .with_async_scope(|cancellation| async move {
                let directory = tempfile::tempdir()?;
                let executor = support::executor_in(
                    cancellation.clone(),
                    &directory.path().join("artifacts"),
                )?;
                let release = ReleaseChildOnDrop(directory.path().join("release"));
                let command = fixture_command(directory.path())?;
                fs::write(&release.0, b"release").await?;
                let caller = Location::caller();
                let error = timeout(
                    LIMIT,
                    executor.cancellable_scope(move |child| async move {
                        if validate {
                            // The explicit-caller API also supports a non-Send,
                            // non-'static validator borrowing local state.
                            let calls = RefCell::new(0);
                            let result = command
                                .run_with_artifacts_and_validator_from(
                                    |_output: ChildOutput| {
                                        *calls.borrow_mut() += 1;
                                        requester.request_cancel("cancel during validation");
                                        Err(IoError::new(
                                            ErrorKind::InvalidData,
                                            "synthetic child output rejected",
                                        )
                                        .into())
                                    },
                                    caller,
                                    &child,
                                )
                                .await;
                            assert_eq!(*calls.borrow(), 1);
                            result.map(|_| ())
                        } else {
                            command
                                .run_with_artifacts_and_mapper::<ChildOutput, (), _>(
                                    move |_output| {
                                        requester.request_cancel("cancel during mapping");
                                        Err(IoError::new(
                                            ErrorKind::InvalidData,
                                            "synthetic child output rejected",
                                        )
                                        .into())
                                    },
                                    &child,
                                )
                                .await
                        }
                    }),
                )
                .await?
                .expect_err("rejected child output must retain its original error");

                let source = error
                    .downcast_ref::<IoError>()
                    .expect("output rejection must preserve the typed source error");
                assert_eq!(source.kind(), ErrorKind::InvalidData);
                assert_eq!(source.to_string(), "synthetic child output rejected");
                let diagnostic = format!("{error:#}");
                assert!(
                    diagnostic.contains("command_cancellation_tests.rs:"),
                    "{diagnostic}"
                );
                if validate {
                    assert!(
                        diagnostic
                            .contains(&format!("command_cancellation_tests.rs:{}:", caller.line())),
                        "{diagnostic}"
                    );
                }

                let root = directory.path().join("artifacts");
                let failures = support::files_named(&root, "error.txt")?;
                assert_eq!(failures.len(), 1);
                assert!(
                    fs::read_to_string(&failures[0])
                        .await?
                        .contains("synthetic child output rejected")
                );
                assert_fixture_artifacts(&root).await?;
                eyre::Ok(())
            })
            .await?;
    }
    Ok(())
}
