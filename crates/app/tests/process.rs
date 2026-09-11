//! Exercise process-global startup in isolated test processes, never this parent.

use cloud_terrastodon_app::App;
use cloud_terrastodon_app::ApplicationCli;
use cloud_terrastodon_app::CancellationToken;
use cloud_terrastodon_app::GlobalArgs;
use cloud_terrastodon_app::Result;
use std::cell::Cell;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::process::Command;
use std::process::Output;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

const PROBE_MODE: &str = "CLOUD_TERRASTODON_APP_PROCESS_TEST_MODE";
const HANDLER_ERROR: &str = "consumer-handler-error-sentinel";
const HANDLER_PANIC: &str = "consumer-handler-panic-sentinel";
const VERIFIED: &str = "runner probe verified environment and shutdown";
const CONSUMER_REASON: &str = "consumer-selected cancellation reason";

// Deliberately no Debug or Arbitrary requirement on the consumer-owned CLI.
#[derive(facet::Facet)]
struct ProbeCli {
    #[facet(flatten)]
    global: GlobalArgs,
}

impl ApplicationCli for ProbeCli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global
    }
}

fn cli(debug: bool) -> ProbeCli {
    ProbeCli {
        global: GlobalArgs {
            debug,
            #[cfg(feature = "auth")]
            auth_source: cloud_terrastodon_app::AuthSource::AzureCli,
            ..GlobalArgs::default()
        },
    }
}

fn app() -> App {
    App::new("runner-process-probe", "0.0.0")
}

fn subprocess(mode: &str) -> Output {
    Command::new(std::env::current_exe().expect("locate the integration test executable"))
        .args([
            "--exact",
            "runner_process_probe",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(PROBE_MODE, mode)
        // Configure only the child environment. No unsafe process-wide mutation
        // is needed in either the parent test process or the application runner.
        .env_remove("RUST_BACKTRACE")
        .env_remove("RUST_LIB_BACKTRACE")
        .env_remove("RUST_LOG")
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("CLICOLOR_FORCE", "0")
        .env("CLOUD_TERRASTODON_REAUTH", "DENY")
        .output()
        .expect("run the dedicated process probe")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "probe failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn handler_constructor_has_runtime_and_terminal_scopes_and_exit_cancels() {
    let output = subprocess("success");
    assert_success(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(VERIFIED));
    assert!(!stderr.contains("Cancellation requested"));
}

#[test]
fn handler_error_returns_without_losing_cancellation_cleanup() {
    assert_success(&subprocess("error"));
}

#[test]
fn shutdown_preserves_the_consumers_existing_cancellation_reason() {
    assert_success(&subprocess("already_cancelled"));
}

#[test]
fn debug_forces_a_panic_backtrace_without_changing_the_environment() {
    let output = subprocess("panic_debug");
    assert!(!output.status.success(), "the panic probe must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(HANDLER_PANIC), "{stderr}");
    assert!(stderr.contains(VERIFIED), "{stderr}");
    let (_, backtrace) = stderr
        .split_once("Debug backtrace:")
        .expect("debug should explicitly force-capture a panic backtrace");
    assert!(
        backtrace
            .lines()
            .any(|line| line.trim_start().starts_with("0:")),
        "expected a captured frame, not merely a backtrace heading:\n{stderr}",
    );
}

#[test]
fn normal_mode_does_not_force_a_panic_backtrace() {
    let output = subprocess("panic_plain");
    assert!(!output.status.success(), "the panic probe must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(HANDLER_PANIC), "{stderr}");
    assert!(stderr.contains(VERIFIED), "{stderr}");
    assert!(!stderr.contains("Debug backtrace:"), "{stderr}");
}

#[test]
fn a_second_runner_initialization_returns_an_error() {
    assert_success(&subprocess("second_run"));
}

#[test]
fn an_invalid_logging_filter_returns_an_error_without_invoking_the_handler() {
    assert_success(&subprocess("invalid_filter"));
}

#[test]
fn invalid_parse_outcomes_do_not_occupy_the_process_error_hook() {
    assert_success(&subprocess("parse_then_run"));
}

/// Only the subprocess selected by PROBE_MODE performs process-global startup.
#[test]
fn runner_process_probe() {
    let Ok(mode) = std::env::var(PROBE_MODE) else {
        return;
    };
    assert_backtrace_environment_unchanged();
    match mode.as_str() {
        "invalid_filter" => {
            let mut cli = cli(false);
            cli.global.log_filter = "consumer=not-a-level".into();
            let error = app()
                .run_parsed(cli, |_, _| async {
                    panic!("invalid filter invoked the handler")
                })
                .expect_err("the invalid directive should be rejected");
            assert!(
                error.to_string().contains("error parsing level filter"),
                "{error:#}"
            );
        }
        "parse_then_run" => {
            assert!(
                app()
                    .parse_from::<ProbeCli>(["--definitely-unknown-argument"])
                    .into_result()
                    .is_err()
            );
            #[derive(facet::Facet)]
            struct InvalidCli {
                global: GlobalArgs,
            }
            impl ApplicationCli for InvalidCli {
                fn global_args(&self) -> &GlobalArgs {
                    &self.global
                }
            }
            assert!(matches!(
                app()
                    .parse_from::<InvalidCli>(Vec::<String>::new())
                    .into_result(),
                Err(figue::DriverError::Builder { .. })
            ));
            run_handler_probe("success");
        }
        "second_run" => {
            run_handler_probe("success");
            let error = app()
                .run_parsed(cli(false), |_, _| async {
                    panic!("duplicate startup must not invoke the handler")
                })
                .expect_err("installing process-global hooks twice must return an error");
            assert!(!error.to_string().is_empty());
        }
        "success" | "error" | "already_cancelled" | "panic_debug" | "panic_plain" => {
            run_handler_probe(&mode);
        }
        unexpected => panic!("unknown probe mode {unexpected}"),
    }
    assert_backtrace_environment_unchanged();
}

fn assert_backtrace_environment_unchanged() {
    assert_eq!(std::env::var_os("RUST_BACKTRACE"), None);
    assert_eq!(std::env::var_os("RUST_LIB_BACKTRACE"), None);
}

fn run_handler_probe(mode: &str) {
    let token_slot: Arc<Mutex<Option<CancellationToken>>> = Arc::new(Mutex::new(None));
    let progress = Rc::new(Cell::new(0));
    // Catch only to inspect shutdown after a panic, then resume the original
    // payload so the child process still reports a failing test.
    let output = catch_unwind(AssertUnwindSafe(|| {
        app().run_parsed(cli(mode == "panic_debug"), |_, context| {
            *token_slot.lock().expect("token slot") = Some(context.cancellation.clone());
            assert!(!context.cancellation.is_cancelled());
            assert!(
                tokio::runtime::Handle::try_current().is_ok(),
                "the synchronous future constructor must already be inside the runtime",
            );
            #[cfg(feature = "terminal")]
            {
                assert!(
                    cloud_terrastodon_user_input::TerminalCoordinator::try_current().is_some(),
                    "the synchronous constructor must inherit the terminal coordinator",
                );
                assert!(
                    cloud_terrastodon_user_input::try_current_picker_log_buffer().is_some(),
                    "the synchronous constructor must inherit the picker log buffer",
                );
            }
            progress.set(1);
            let progress = Rc::clone(&progress);
            async move {
                // Retaining Rc across an await also proves the runner accepts a
                // borrowed, non-Send invocation rather than spawning it.
                tokio::task::yield_now().await;
                progress.set(2);
                match mode {
                    "panic_debug" | "panic_plain" => panic!("{HANDLER_PANIC}"),
                    "error" => Err(eyre::eyre!(HANDLER_ERROR).wrap_err("consumer handler failed")),
                    "already_cancelled" => {
                        context.cancellation.request_cancel(CONSUMER_REASON);
                        Ok(())
                    }
                    _ => Result::<()>::Ok(()),
                }
            }
        })
    }));

    assert_eq!(progress.get(), 2, "the handler must have been polled");
    let token = token_slot
        .lock()
        .expect("token slot")
        .clone()
        .expect("the constructor should expose its cancellation token");
    assert!(
        token.is_cancelled(),
        "runner exit must request cancellation"
    );
    if mode == "already_cancelled" {
        assert_eq!(
            token.cancellation_reason().as_deref(),
            Some(CONSUMER_REASON)
        );
    }
    assert_backtrace_environment_unchanged();
    eprintln!("{VERIFIED}");

    match mode {
        "panic_debug" | "panic_plain" => {
            let payload = output.expect_err("the panic must escape the runner");
            std::panic::resume_unwind(payload);
        }
        "error" => {
            let error = output
                .expect("a returned error must not panic")
                .expect_err("the consumer error must propagate");
            assert!(format!("{error:#}").contains(HANDLER_ERROR));
        }
        _ => output
            .expect("successful invocation must not panic")
            .expect("successful invocation must return success"),
    }
}
