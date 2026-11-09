#![cfg(test)]

use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::RetryBehaviour;
use eyre::Result;
use facet::Facet;
use std::cell::Cell;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::panic::Location;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tracing::Instrument;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;

#[derive(Debug, Facet)]
#[facet(proxy = u64)]
struct OwnedNumber(#[facet(opaque)] Cell<u64>);

impl TryFrom<u64> for OwnedNumber {
    type Error = eyre::Report;

    fn try_from(value: u64) -> Result<Self> {
        tracing::info!(target: "plain_command_decode_probe", "decoded owned number");
        Ok(Self(Cell::new(value)))
    }
}

impl From<&OwnedNumber> for u64 {
    fn from(value: &OwnedNumber) -> Self {
        value.0.get()
    }
}

struct CommandTrace {
    decoder: Arc<Mutex<Vec<String>>>,
    callback: Arc<Mutex<Vec<String>>>,
}

impl Layer<Registry> for CommandTrace {
    fn on_event(&self, event: &tracing::Event<'_>, context: Context<'_, Registry>) {
        let traces = match event.metadata().target() {
            "plain_command_decode_probe" => &self.decoder,
            "plain_command_callback_probe" => &self.callback,
            _ => return,
        };
        if let Some(scope) = context.event_scope(event) {
            traces
                .lock()
                .unwrap()
                .extend(scope.from_root().map(|span| span.name().to_owned()));
        }
    }
}

fn counted_command(directory: &Path) -> CommandBuilder {
    let mut command = CommandBuilder::new(CommandKind::Pwsh);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"
$ErrorActionPreference = 'Stop'
$count = 0
if ([IO.File]::Exists('count')) { $count = [int][IO.File]::ReadAllText('count') }
[IO.File]::WriteAllText('count', [string]($count + 1))
[Console]::Write('9')
"#,
        ])
        .use_run_dir(directory)
        .use_retry_behaviour(RetryBehaviour::Fail)
        .use_timeout(Duration::from_secs(15));
    command
}

#[tokio::test]
async fn plain_execution_needs_only_a_scope_and_keeps_owned_and_borrowed_outputs() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let command = counted_command(directory.path());
            let first = command.run::<OwnedNumber>(&scope).await?;
            first.0.set(100);
            let second = command.run::<OwnedNumber>(&scope).await?;
            assert_eq!(second.0.get(), 9);

            let calls = Rc::new(Cell::new(0));
            let label = String::from("borrowed caller output");
            let borrowed = command
                .run_with_mapper::<OwnedNumber, _, _>(
                    |output| {
                        assert_eq!(output.0.get(), 9);
                        calls.set(calls.get() + 1);
                        Ok(label.as_str())
                    },
                    &scope,
                )
                .await?;
            assert!(std::ptr::eq(borrowed.as_ptr(), label.as_ptr()));
            assert_eq!(calls.get(), 1);
            assert_eq!(
                std::fs::read_to_string(directory.path().join("count"))?,
                "3"
            );
            assert_eq!(std::fs::read_dir(directory.path())?.count(), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn plain_execution_rejects_explicit_cache_intent_before_starting_the_process() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let directory = tempfile::tempdir()?;
            let mut command = counted_command(directory.path());
            command.cache(CacheKey {
                scope: CacheScope::try_from_components(["synthetic", "plain-rejection"])?,
                valid_for: Duration::ZERO,
            });
            let error = command.run_raw(&scope).await.unwrap_err();
            assert!(format!("{error:#}").contains("cache"), "{error:#}");
            assert!(!directory.path().join("count").exists());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn plain_decoding_keeps_the_parent_trace_and_caller_policy_error() -> Result<()> {
    let traces = Arc::new(Mutex::new(Vec::new()));
    let callback_traces = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CommandTrace {
        decoder: Arc::clone(&traces),
        callback: Arc::clone(&callback_traces),
    });
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let mut command = CommandBuilder::new(CommandKind::Echo);
            command.args(["9"]);
            let calls = Rc::new(Cell::new(0));
            let caller = Location::caller();
            let error = async {
                command.run_with_validator_from(
                    |output: OwnedNumber| {
                        tracing::info!(target: "plain_command_callback_probe", "validating plain output");
                        assert_eq!(output.0.get(), 9);
                        calls.set(calls.get() + 1);
                        Err(IoError::new(ErrorKind::InvalidData, "synthetic plain policy rejection").into())
                    },
                    caller,
                    &scope,
                ).await
            }
            .instrument(tracing::info_span!("plain_command_parent"))
            .await
            .unwrap_err();
            assert_eq!(calls.get(), 1);
            assert_eq!(error.downcast_ref::<IoError>().unwrap().kind(), ErrorKind::InvalidData);
            assert!(format!("{error:#}").contains(&format!("plain_command_tests.rs:{}:", caller.line())));
            eyre::Ok(())
        })
        .with_subscriber(subscriber)
        .await?;
    assert!(
        traces
            .lock()
            .unwrap()
            .iter()
            .any(|span| span == "plain_command_parent")
    );
    let callback_traces = callback_traces.lock().unwrap();
    assert!(
        callback_traces
            .iter()
            .any(|span| span == "plain_command_parent")
    );
    assert!(
        callback_traces
            .iter()
            .any(|span| span == "command_output_policy")
    );
    Ok(())
}

#[tokio::test]
async fn plain_sensitive_decode_failures_suppress_the_captured_payload() -> Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let error = CommandBuilder::new(CommandKind::Echo)
                .args(["synthetic-sensitive-plain-output"])
                .artifact_retention(ArtifactRetention::Disabled)
                .run::<u64>(&scope)
                .await
                .unwrap_err();
            assert!(!format!("{error:?}").contains("synthetic-sensitive-plain-output"));
            Ok(())
        })
        .await
}
