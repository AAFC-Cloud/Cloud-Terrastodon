#![cfg(test)]

use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_cache::WorkOutput;
use cloud_terrastodon_cache::WorkReuse;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::RetryBehaviour;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use eyre::Result;
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

struct SyntheticNumber;
struct OtherOperation;
struct ParentRead;

struct ManagedCommandRequest(CommandBuilder);

impl ExecutableRequest for ManagedCommandRequest {
    type Output = CommandOutput;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move { self.0.run_raw_with_artifacts(&executor).await })
    }
}

static REJECT_DISK_VALUE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, facet::Facet)]
#[facet(proxy = u64)]
struct RevisableNumber(u64);

impl TryFrom<u64> for RevisableNumber {
    type Error = eyre::Report;

    fn try_from(value: u64) -> Result<Self> {
        eyre::ensure!(
            !REJECT_DISK_VALUE.load(Ordering::SeqCst),
            "canonical schema no longer accepts the stored value"
        );
        Ok(Self(value))
    }
}

impl From<&RevisableNumber> for u64 {
    fn from(value: &RevisableNumber) -> Self {
        value.0
    }
}

static DECODES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, facet::Facet)]
#[facet(proxy = u64)]
struct CountedNumber(u64);

impl TryFrom<u64> for CountedNumber {
    type Error = eyre::Report;

    fn try_from(value: u64) -> Result<Self> {
        DECODES.fetch_add(1, Ordering::SeqCst);
        Ok(Self(value))
    }
}

impl From<&CountedNumber> for u64 {
    fn from(value: &CountedNumber) -> Self {
        value.0
    }
}

#[derive(facet::Facet)]
#[facet(proxy = u64)]
struct OwnedNumber(#[facet(opaque)] Cell<u64>);

impl TryFrom<u64> for OwnedNumber {
    type Error = eyre::Report;

    fn try_from(value: u64) -> Result<Self> {
        Ok(Self(Cell::new(value)))
    }
}

impl From<&OwnedNumber> for u64 {
    fn from(value: &OwnedNumber) -> Self {
        value.0.get()
    }
}

fn key() -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", "number"]).unwrap(),
        valid_for: Duration::from_secs(3600),
    }
}

fn command(directory: &Path, output: &str) -> CommandBuilder {
    let mut command = CommandBuilder::new(CommandKind::Pwsh);
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        r#"
$ErrorActionPreference = 'Stop'
$count = 0
if ([IO.File]::Exists('count')) { $count = [int][IO.File]::ReadAllText('count') }
[IO.File]::WriteAllText('count', [string]($count + 1))
[Console]::Write($env:CT_SYNTHETIC_OUTPUT)
"#,
    ]);
    command.use_run_dir(directory);
    command.env("CT_SYNTHETIC_OUTPUT", output);
    command.use_retry_behaviour(RetryBehaviour::Fail);
    command.use_timeout(Duration::from_secs(15));
    command.cache(key());
    command
}

#[tokio::test]
async fn canonical_cache_shares_decode_and_runs_each_callers_policy() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let command = command(directory.path(), "41");
            let owner = CacheOperationOwner::of::<SyntheticNumber>();
            DECODES.store(0, Ordering::SeqCst);
            let (first, second) = tokio::try_join!(
                command.run_cached::<CountedNumber>(&executor, owner),
                command.run_cached::<CountedNumber>(&executor, owner),
            )?;
            assert!(Arc::ptr_eq(&first, &second));
            assert_eq!(first.0, 41);
            assert_eq!(DECODES.load(Ordering::SeqCst), 1);

            let calls = Rc::new(Cell::new(0));
            let label = String::from("caller-owned label");
            let borrowed = command
                .run_cached_with_mapper::<CountedNumber, _, _>(
                    |value| {
                        calls.set(calls.get() + 1);
                        assert_eq!(value.0, 41);
                        Ok(label.as_str())
                    },
                    &executor,
                    owner,
                )
                .await?;
            assert!(std::ptr::eq(borrowed.as_ptr(), label.as_ptr()));
            let rejected = command
                .run_cached_with_validator::<CountedNumber, _>(
                    |_| {
                        calls.set(calls.get() + 1);
                        eyre::bail!("synthetic caller policy rejected the canonical value");
                    },
                    &executor,
                    owner,
                )
                .await;
            assert!(format!("{:#}", rejected.unwrap_err()).contains("synthetic caller policy"));
            assert_eq!(calls.get(), 2);
            assert!(Arc::ptr_eq(
                &first,
                &command
                    .run_cached::<CountedNumber>(&executor, owner)
                    .await?,
            ));
            assert_eq!(DECODES.load(Ordering::SeqCst), 1);

            executor.cache()?.release_memory()?;
            let loaded = command
                .run_cached::<CountedNumber>(&executor, owner)
                .await?;
            assert_eq!(loaded.0, 41);
            assert_eq!(DECODES.load(Ordering::SeqCst), 2);
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "1"
            );
            assert!(command.run_cached::<u64>(&executor, owner).await.is_err());
            assert!(
                command
                    .run_cached::<CountedNumber>(
                        &executor,
                        CacheOperationOwner::of::<OtherOperation>()
                    )
                    .await
                    .is_err()
            );

            executor
                .cache()?
                .invalidate(&key().scope, executor.cancellation())
                .await?;
            command
                .run_cached::<CountedNumber>(&executor, owner)
                .await?;
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "2"
            );
            assert_eq!(DECODES.load(Ordering::SeqCst), 3);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn owned_non_sync_outputs_execute_independently_even_with_a_reusable_artifact_key()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let command = command(directory.path(), "9");
            let first = command.run_with_artifacts::<OwnedNumber>(&executor).await?;
            first.0.set(100);
            let second = command.run_with_artifacts::<OwnedNumber>(&executor).await?;
            assert_eq!(second.0.get(), 9);
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "2"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn managed_command_request_cannot_hide_nonreusable_work_beneath_a_cached_parent() -> Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let command = command(directory.path(), "9");
            let starts = AtomicUsize::new(0);
            for expected in 0..2 {
                let starts = &starts;
                let command = &command;
                let output = executor
                    .cached_work(
                        ExecutorKind::InProcess,
                        CacheKey {
                            scope: CacheScope::try_from_components([
                                "synthetic",
                                "managed-parent",
                            ])?,
                            valid_for: Duration::from_secs(3600),
                        },
                        CacheOperationOwner::of::<ParentRead>(),
                        &AuthenticationSelection::Anonymous,
                        NonEmptyString::new("synthetic-parent-input")?,
                        WorkReuse::ReusableRead,
                        move |parent| async move {
                            let fallback = starts.fetch_add(1, Ordering::SeqCst);
                            let error = parent
                                .execute(ManagedCommandRequest(command.clone()))
                                .await
                                .unwrap_err();
                            assert!(format!("{error:#}").contains("nonreusable"), "{error:#}");
                            Ok(WorkOutput::new(fallback, ArtifactObservation::now()))
                        },
                    )?
                    .await?;
                assert_eq!(*output, expected);
            }
            assert_eq!(starts.load(Ordering::SeqCst), 2);
            assert!(!directory.path().join("count").exists());
            assert!(
                std::fs::read_dir(executor.cache()?.root())?
                    .next()
                    .is_none()
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn rejected_canonical_output_is_retained_but_cannot_replace_later_execution() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let command = command(directory.path(), "not-json");
            for _ in 0..2 {
                let error = command
                    .run_cached::<u64>(&executor, CacheOperationOwner::of::<SyntheticNumber>())
                    .await
                    .unwrap_err();
                assert!(format!("{error:#}").contains("canonical decoding failed"));
            }
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "2"
            );
            assert!(
                std::fs::read_dir(executor.cache()?.root())?
                    .next()
                    .is_some()
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn sensitive_commands_retain_no_raw_files_and_suppress_failed_output() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let mut command = command(directory.path(), "synthetic-secret-output");
            command.artifact_retention(ArtifactRetention::Disabled);
            let error = command
                .run_with_artifacts::<u64>(&executor)
                .await
                .unwrap_err();
            assert!(!format!("{error:?}").contains("synthetic-secret-output"));
            assert!(
                std::fs::read_dir(executor.cache()?.root())?
                    .next()
                    .is_none()
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn rejecting_a_disk_value_preserves_other_variants_and_child_pages() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let owner = CacheOperationOwner::of::<SyntheticNumber>();
            let first = command(directory.path(), "1");
            let sibling = command(directory.path(), "2");
            let mut page = command(directory.path(), "3");
            page.cache(CacheKey {
                scope: CacheScope::try_from_components(["synthetic", "number", "page"])?,
                ..key()
            });
            first
                .run_cached::<RevisableNumber>(&executor, owner)
                .await?;
            sibling
                .run_cached::<RevisableNumber>(&executor, owner)
                .await?;
            page.run_cached::<RevisableNumber>(&executor, owner).await?;
            executor.cache()?.release_memory()?;
            REJECT_DISK_VALUE.store(true, Ordering::SeqCst);
            let rejected = first.run_cached::<RevisableNumber>(&executor, owner).await;
            REJECT_DISK_VALUE.store(false, Ordering::SeqCst);
            assert!(
                format!("{:#}", rejected.unwrap_err())
                    .contains("canonical schema no longer accepts")
            );
            // Rejection reports the original failure without executing a replacement.
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "3"
            );
            assert_eq!(
                sibling
                    .run_cached::<RevisableNumber>(&executor, owner)
                    .await?
                    .0,
                2
            );
            assert_eq!(
                page.run_cached::<RevisableNumber>(&executor, owner)
                    .await?
                    .0,
                3
            );
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "3"
            );
            first
                .run_cached::<RevisableNumber>(&executor, owner)
                .await?;
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "4"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn shared_identity_distinguishes_command_argument_semantics_and_requires_capture()
-> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let executor = CloudTerrastodonExecutor::new(
                WorkCache::temporary()?,
                Arc::new(AnonymousAuthenticationProvider),
                scope,
            );
            let owner = CacheOperationOwner::of::<SyntheticNumber>();
            let mut echo = CommandBuilder::new(CommandKind::Echo);
            echo.args(["-NoProfile", "-Command", "[Console]::Write(7)"])
                .cache(key());
            let echoed = echo.run_raw_cached(&executor, owner).await?;
            let mut script = CommandBuilder::new(CommandKind::Pwsh);
            script
                .args(["-NoProfile", "-Command", "[Console]::Write(7)"])
                .cache(key());
            let executed = script.run_raw_cached(&executor, owner).await?;
            assert!(String::from_utf8_lossy(&echoed.stdout).contains("[Console]::Write(7)"));
            assert_eq!(executed.stdout.as_slice(), b"7");
            script.use_output_behaviour(cloud_terrastodon_command::OutputBehaviour::Display);
            let error = script.run_raw_cached(&executor, owner).await.unwrap_err();
            assert!(format!("{error:#}").contains("requires captured output"));
            Ok(())
        })
        .await
}
