use std::ffi::OsStr;
use std::process::{Command, Output};

// Import the real single-file consumer to test its public CLI without adding a
// library target or requiring Debug/Arbitrary on any consumer argument type.
#[allow(dead_code)]
#[path = "../src/main.rs"]
mod consumer;

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_field-notes"));
    command
        .env_remove("RUST_LOG")
        .env_remove("RUST_BACKTRACE")
        .env_remove("RUST_LIB_BACKTRACE")
        .env("NO_COLOR", "1");
    command
}

fn run(args: &[&str]) -> Output {
    command().args(args).output().expect("run consumer binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr")
}

fn normalized_stdout(output: &Output) -> String {
    stdout(output)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", stderr(output));
}

fn assert_shared_options_once(help: &str) {
    for spelling in [
        "--[no-]debug",
        "--log-filter ",
        "--log-file-filter ",
        "--log-file ",
        "--[no-]help",
        "--[no-]html-help",
        "--[no-]version",
        "--completions ",
        "--export-jsonschemas ",
    ] {
        assert_eq!(help.matches(spelling).count(), 1, "{spelling}:\n{help}");
    }
    assert!(!help.contains("--auth-source"), "auth is opt-in: {help}");
}

#[test]
fn root_and_nested_help_inherit_all_shared_options_once() {
    for args in [
        vec!["--help"],
        vec!["echo", "--help"],
        vec!["inspect", "--help"],
        vec!["inspect", "words", "--help"],
    ] {
        let output = run(&args);
        assert_success(&output);
        let help = stdout(&output);
        assert!(help.contains("field-notes"), "{help}");
        assert!(!help.contains("AAFC-Cloud/Cloud-Terrastodon"));
        assert_shared_options_once(&help);
        assert!(stderr(&output).is_empty(), "{}", stderr(&output));
    }
}

#[test]
fn help_preserves_multiline_consumer_and_shared_documentation() {
    let root = run(&["--help"]);
    assert_success(&root);
    assert!(
        normalized_stdout(&root).contains("The commands and their data belong to this consumer")
    );
    let echo = run(&["echo", "--help"]);
    assert_success(&echo);
    assert!(normalized_stdout(&echo).contains("Quote this argument in your shell"));
    let words = run(&["inspect", "words", "--help"]);
    assert_success(&words);
    let help = normalized_stdout(&words);
    assert!(help.contains("Tabs and newlines separate words just like spaces."));
    assert!(help.contains("This local work limit does not require a timer or a signal."));
    assert!(help.contains("is also used for the file output."));
}

#[test]
fn version_uses_the_consumers_identity() {
    for args in [vec!["--version"], vec!["inspect", "words", "--version"]] {
        let output = run(&args);
        assert_success(&output);
        let version = stdout(&output);
        assert!(version.contains("field-notes"), "{version}");
        assert!(version.contains(env!("CARGO_PKG_VERSION")), "{version}");
        assert!(!version.contains("cloud_terrastodon"), "{version}");
        assert!(stderr(&output).is_empty());
    }
}

#[test]
fn dispatch_keeps_results_on_stdout_and_logging_on_stderr() {
    let output = run(&["echo", "a message with spaces"]);
    assert_success(&output);
    assert_eq!(stdout(&output), "a message with spaces\n");
    assert!(stderr(&output).contains("echo dispatched"));
    assert!(!stderr(&output).contains("echo diagnostic"));

    let nested = run(&["inspect", "words", "one\ttwo\nthree", "--expect", "3"]);
    assert_success(&nested);
    assert_eq!(stdout(&nested), "3\n");
}

#[test]
fn global_filters_work_after_the_leaf_command() {
    let quiet = run(&["echo", "quiet", "--log-filter", "off"]);
    assert_success(&quiet);
    assert_eq!(stdout(&quiet), "quiet\n");
    assert!(stderr(&quiet).is_empty());

    let debug = run(&["echo", "verbose", "--debug"]);
    assert_success(&debug);
    assert!(stderr(&debug).contains("echo diagnostic"));

    let alias = run(&["echo", "quiet", "--log-level", "off"]);
    assert_success(&alias);
    assert!(stderr(&alias).is_empty());
}

#[test]
fn module_level_directives_select_the_consumers_log_target() {
    let (output, events) = run_with_log_file("field_notes=debug", Some("field_notes=info"));
    assert!(stderr(&output).contains("echo diagnostic"));
    assert!(has_event(&events, "echo dispatched", "INFO"));
    assert!(!has_event(&events, "echo diagnostic", "DEBUG"));

    let unrelated = run(&["echo", "scoped", "--log-filter", "unrelated_module=debug"]);
    assert_success(&unrelated);
    assert!(stderr(&unrelated).is_empty());
}

fn run_with_log_file(console: &str, file: Option<&str>) -> (Output, Vec<serde_json::Value>) {
    let directory = tempfile::tempdir().expect("temporary test directory");
    let log_path = directory.path().join("events.ndjson");
    let mut command = command();
    command.args([
        "echo",
        "logged message",
        "--log-filter",
        console,
        "--log-file",
    ]);
    command.arg(&log_path);
    if let Some(filter) = file {
        command.args(["--log-file-filter", filter]);
    }
    let output = command.output().expect("run consumer with file logging");
    assert_success(&output);
    let text = std::fs::read_to_string(log_path).expect("read completed NDJSON log");
    let events = text
        .lines()
        .map(|line| {
            assert!(!line.contains('\u{1b}'), "JSON contains ANSI escape codes");
            serde_json::from_str(line).expect("every log line is a JSON value")
        })
        .collect();
    (output, events)
}

fn has_event(events: &[serde_json::Value], message: &str, level: &str) -> bool {
    events
        .iter()
        .any(|event| event["fields"]["message"] == message && event["level"] == level)
}

#[test]
fn ndjson_filter_is_independent_of_stderr_in_both_directions() {
    let (quiet, detailed_file) = run_with_log_file("off", Some("debug"));
    assert_eq!(stdout(&quiet), "logged message\n");
    assert!(stderr(&quiet).is_empty());
    assert!(
        has_event(&detailed_file, "echo diagnostic", "DEBUG"),
        "{detailed_file:#?}"
    );
    assert!(has_event(&detailed_file, "echo dispatched", "INFO"));

    let (detailed, quiet_file) = run_with_log_file("debug", Some("off"));
    assert!(stderr(&detailed).contains("echo diagnostic"));
    assert!(stderr(&detailed).contains("echo dispatched"));
    assert!(quiet_file.is_empty());
}

#[test]
fn ndjson_filter_defaults_to_the_console_filter() {
    let (_, events) = run_with_log_file("info", None);
    assert!(has_event(&events, "echo dispatched", "INFO"));
    assert!(!has_event(&events, "echo diagnostic", "DEBUG"));
}

#[test]
fn invalid_input_is_reported_without_dispatch() {
    for args in [
        vec!["unknown-command"],
        vec!["inspect", "words", "text", "--expect", "not-a-number"],
        vec!["echo", "text", "--unknown-option"],
    ] {
        let output = run(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(stdout(&output).is_empty(), "{}", stdout(&output));
        assert!(!stderr(&output).is_empty());
        assert!(!stderr(&output).contains("echo dispatched"));
    }
}

#[test]
fn missing_positional_preserves_figues_help_diagnostic_and_success_exit() {
    let output = run(&["echo"]);
    assert_success(&output);
    let help = stdout(&output);
    assert!(help.contains("field-notes echo"));
    assert!(help.contains("<MESSAGE>"));
    assert!(help.contains("missing required argument"));
    assert!(!help.contains("echo dispatched"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn handler_errors_propagate_to_a_failure_exit() {
    let output = run(&["inspect", "words", "one two", "--expect", "3"]);
    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("word count did not match --expect"));
}

#[test]
fn cancellation_is_cooperative_and_deterministic() {
    let directory = tempfile::tempdir().expect("temporary cancellation log directory");
    let log_path = directory.path().join("cancellation.ndjson");
    let output = command()
        .args([
            "inspect",
            "words",
            "first second third",
            "--stop-after",
            "1",
            "--log-filter",
            "off",
            "--log-file-filter",
            "debug",
            "--log-file",
        ])
        .arg(&log_path)
        .output()
        .expect("run cooperative cancellation");
    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("local word limit reached"));
    let events: Vec<serde_json::Value> = std::fs::read_to_string(log_path)
        .expect("read cancellation log")
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid cancellation JSON log event"))
        .collect();
    let processed: Vec<_> = events
        .iter()
        .filter(|event| event["fields"]["message"] == "inspecting word")
        .map(|event| event["fields"]["word"].as_str().expect("word field"))
        .collect();
    assert_eq!(processed, ["first"]);
}

#[test]
fn public_consumer_cli_parses_without_starting_the_application() {
    use cloud_terrastodon::app::{App, ApplicationCli};
    let cli = App::new("field-notes", "test-version")
        .parse_from::<consumer::Cli>(
            [
                "inspect",
                "words",
                "one two",
                "--expect",
                "2",
                "--log-filter",
                "warn",
            ]
            .map(OsStr::new),
        )
        .into_result()
        .expect("parse consumer-owned CLI")
        .value;
    assert_eq!(cli.global_args().log_filter, "warn");
    let consumer::Command::Inspect(args) = cli.command else {
        panic!("expected consumer inspect command");
    };
    let consumer::InspectCommand::Words(args) = args.command;
    assert_eq!(args.text, "one two");
    assert_eq!(args.expect, Some(2));
}

#[test]
fn parse_only_help_returns_an_outcome_instead_of_exiting() {
    use cloud_terrastodon::app::{App, figue};
    let result = App::new("field-notes", "test-version")
        .parse_from::<consumer::Cli>([OsStr::new("--help")])
        .into_result();
    assert!(matches!(result, Err(figue::DriverError::Help { .. })));
}
