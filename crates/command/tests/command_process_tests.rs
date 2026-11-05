#![cfg(test)]

mod support;

use bstr::ByteSlice;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_command::PathMapper;
use cloud_terrastodon_command::RetryBehaviour;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use tokio::time::timeout;

const LIMIT: Duration = Duration::from_secs(20);

// PowerShell parses arguments following -Command as source. Quote only the
// generated file path; fixtures never interpolate credentials or user input.
#[derive(Debug)]
struct PowerShellPathArgument;

impl PathMapper for PowerShellPathArgument {
    fn map_path<'a>(&self, path: &'a Path) -> Cow<'a, Path> {
        Cow::Owned(PathBuf::from(format!(
            "'{}'",
            path.to_string_lossy().replace('\'', "''")
        )))
    }
}

// Release gated children on early assertion failure as well as normal completion.
// CommandBuilder's kill-on-drop owns terminating abandoned processes.
struct ReleaseChildOnDrop(PathBuf);

impl Drop for ReleaseChildOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.0, b"release");
    }
}

fn fixture_command(directory: &Path, script: &str) -> Result<CommandBuilder> {
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
        script.trim(),
    ]);
    command.use_run_dir(directory);
    command.env("CT_PROCESS_FIXTURE", directory.to_string_lossy());
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

#[tokio::test]
async fn concurrent_processes_keep_their_own_staged_input_until_the_child_finishes() -> Result<()> {
    const SCRIPT: &str = r#"
& {
    param([string]$InputPath)
    $ErrorActionPreference = 'Stop'
    $started = [IO.Path]::Combine($env:CT_PROCESS_FIXTURE, $env:CT_PROCESS_NAME)
    $release = [IO.Path]::Combine($env:CT_PROCESS_FIXTURE, 'release')
    [IO.File]::WriteAllText($started, 'ready')
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not [IO.File]::Exists($release)) {
        if ([DateTime]::UtcNow -ge $deadline) { throw 'fixture release timed out' }
        [Threading.Thread]::Sleep(5)
    }
    [Console]::WriteLine($InputPath)
    [Console]::Write([IO.File]::ReadAllText($InputPath))
}
"#;
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let release = ReleaseChildOnDrop(directory.path().join("release"));
            let first_started = directory.path().join("first-started");
            let second_started = directory.path().join("second-started");
            let mut first = fixture_command(directory.path(), SCRIPT)?;
            first.env("CT_PROCESS_NAME", "first-started");
            first.file_arg("input.txt", PowerShellPathArgument, "first payload".to_owned());
            let mut second = fixture_command(directory.path(), SCRIPT)?;
            second.env("CT_PROCESS_NAME", "second-started");
            second.file_arg("input.txt", PowerShellPathArgument, "second payload".to_owned());

            let mut first_run = Box::pin(first.run_raw(&cancellation));
            tokio::select! {
                output = &mut first_run => { bail!("first child finished before release: {output:?}"); },
                ready = wait_for_file(&first_started) => ready?,
            }
            let mut second_run = Box::pin(second.run_raw(&cancellation));
            tokio::select! {
                output = &mut first_run => { bail!("first child finished before release: {output:?}"); },
                output = &mut second_run => { bail!("second child finished before release: {output:?}"); },
                ready = wait_for_file(&second_started) => ready?,
            }
            fs::write(&release.0, b"release").await?;
            let (first_output, second_output) =
                timeout(LIMIT, async { tokio::try_join!(first_run, second_run) }).await??;
            let first_text = first_output.stdout.to_str()?;
            let second_text = second_output.stdout.to_str()?;
            let (first_path, first_payload) = first_text.split_once('\n').expect("staged path output");
            let (second_path, second_payload) = second_text.split_once('\n').expect("staged path output");
            assert_eq!(first_payload, "first payload");
            assert_eq!(second_payload, "second payload");
            assert_ne!(first_path, second_path);
            // Ephemeral inputs disappear when their invocation owner is dropped,
            // even when no artifact/cache owner participates in execution.
            assert!(!fs::try_exists(Path::new(first_path.trim_end_matches('\r'))).await?);
            assert!(!fs::try_exists(Path::new(second_path.trim_end_matches('\r'))).await?);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn rate_limit_retry_does_not_reenter_cache_lookup() -> Result<()> {
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$attempt = [IO.Path]::Combine($env:CT_PROCESS_FIXTURE, 'attempt')
if (-not [IO.File]::Exists($attempt)) {
    [IO.File]::WriteAllText($attempt, '1')
    $release = [IO.Path]::Combine($env:CT_PROCESS_FIXTURE, 'release')
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not [IO.File]::Exists($release)) {
        if ([DateTime]::UtcNow -ge $deadline) { throw 'fixture release timed out' }
        [Threading.Thread]::Sleep(5)
    }
    [Console]::Error.WriteLine("ERROR: Too Many Requests 'x-ms-user-quota-resets-after': '00:00:00'")
    exit 29
}
[IO.File]::WriteAllText($attempt, '2')
[Console]::Write('retry executed')
"#;
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let executor =
                support::executor_in(cancellation.clone(), &directory.path().join("artifacts"))?;
            let release = ReleaseChildOnDrop(directory.path().join("release"));
            let artifact_dir = directory.path().join("artifacts");
            let attempted = directory.path().join("attempt");
            let mut command = fixture_command(directory.path(), SCRIPT)?;
            command.cache(support::key(Duration::MAX));
            struct RetryOperation;
            let owner = CacheOperationOwner::of::<RetryOperation>();
            command.use_retry_behaviour(RetryBehaviour::Retry);
            let mut running = Box::pin(command.run_raw_cached(&executor, owner));
            tokio::select! {
                output = &mut running => { bail!("child finished before injection: {output:?}"); },
                ready = wait_for_file(&attempted) => ready?,
            }

            // A second caller joins the same canonical identity while the first
            // attempt is pending. Retrying through cache admission would reacquire
            // the leader's own initializer lock and hang both callers.
            let waiting = command.run_raw_cached(&executor, owner);
            fs::write(&release.0, b"release").await?;
            let (output, joined) =
                timeout(LIMIT, async { tokio::try_join!(running, waiting) }).await??;
            assert!(Arc::ptr_eq(&output, &joined));
            assert_eq!(output.stdout.as_bytes(), b"retry executed");
            assert_eq!(
                fs::read_to_string(directory.path().join("attempt")).await?,
                "2"
            );
            let files = support::files_named(&artifact_dir, "stdout.json")?;
            assert_eq!(files.len(), 1);
            assert_eq!(fs::read(&files[0]).await?, b"retry executed");
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_second_rate_limit_is_reported_with_its_status_and_failure_artifacts() -> Result<()> {
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$attempt = [IO.Path]::Combine($env:CT_PROCESS_FIXTURE, 'attempt')
$number = 1
if ([IO.File]::Exists($attempt)) { $number = [int][IO.File]::ReadAllText($attempt) + 1 }
[IO.File]::WriteAllText($attempt, [string]$number)
[Console]::Write("attempt $number")
[Console]::Error.WriteLine("ERROR: Too Many Requests 'x-ms-user-quota-resets-after': '00:00:00'")
exit 29
"#;
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let executor =
                support::executor_in(cancellation.clone(), &directory.path().join("artifacts"))?;
            let mut command = fixture_command(directory.path(), SCRIPT)?;
            command.use_retry_behaviour(RetryBehaviour::Retry);
            let error = timeout(LIMIT, command.run_raw_with_artifacts(&executor))
                .await?
                .expect_err("the second rate limit must terminate the retry sequence");
            let output = error
                .downcast_ref::<CommandOutput>()
                .expect("process error source");
            assert_eq!(output.status, 29);
            assert_eq!(output.stdout.as_bytes(), b"attempt 2");
            assert_eq!(
                fs::read_to_string(directory.path().join("attempt")).await?,
                "2"
            );
            assert!(
                support::files_named(&directory.path().join("artifacts"), "selection.json")?
                    .is_empty()
            );

            let failures =
                support::files_named(&directory.path().join("artifacts"), "stdout.json")?;
            let mut retained_second_attempt = false;
            for failure in failures {
                if fs::read(&failure).await? == b"attempt 2" {
                    assert_eq!(
                        fs::read_to_string(failure.with_file_name("status.txt")).await?,
                        "29"
                    );
                    retained_second_attempt = true;
                }
            }
            assert!(
                retained_second_attempt,
                "final failed attempt must retain diagnostics"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn large_stdin_and_output_are_drained_concurrently() -> Result<()> {
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
[Console]::Write(('o' * 262144))
[Console]::Error.Write(('e' * 262144))
$inputText = [Console]::In.ReadToEnd()
[Console]::Write("`nreceived=$($inputText.Length)")
"#;
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let mut command = fixture_command(directory.path(), SCRIPT)?;
            command.send_stdin("i".repeat(262_144));
            let output = timeout(LIMIT, command.run_raw(&cancellation)).await??;
            assert_eq!(
                output.stdout.as_bytes(),
                format!("{}\nreceived=262144", "o".repeat(262_144)).as_bytes()
            );
            assert_eq!(output.stderr.as_bytes(), "e".repeat(262_144).as_bytes());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn timeout_ends_a_pending_process_without_publishing_success() -> Result<()> {
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
[IO.File]::WriteAllText([IO.Path]::Combine($env:CT_PROCESS_FIXTURE, 'started'), 'ready')
[Threading.Thread]::Sleep(30000)
[IO.File]::WriteAllText([IO.Path]::Combine($env:CT_PROCESS_FIXTURE, 'completed'), 'unexpected')
"#;
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let directory = tempfile::tempdir()?;
            let executor =
                support::executor_in(cancellation.clone(), &directory.path().join("artifacts"))?;
            let mut command = fixture_command(directory.path(), SCRIPT)?;
            command.use_timeout(Duration::from_secs(5));
            let error = timeout(LIMIT, command.run_raw_with_artifacts(&executor))
                .await?
                .expect_err("the child's wait must be bounded by its execution timeout");
            assert!(
                format!("{error:#}").contains("Command timeout"),
                "{error:#}"
            );
            assert!(fs::try_exists(directory.path().join("started")).await?);
            assert!(!fs::try_exists(directory.path().join("completed")).await?);
            assert!(
                support::files_named(&directory.path().join("artifacts"), "selection.json")?
                    .is_empty()
            );
            Ok(())
        })
        .await
}
