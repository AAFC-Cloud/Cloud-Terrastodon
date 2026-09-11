use super::*;

/// A public root owned by the consuming application, deliberately without Debug.
#[derive(facet::Facet)]
struct ConsumerCli {
    #[facet(flatten)]
    global: GlobalArgs,
    #[facet(flatten)]
    builtins: figue::FigueBuiltins,
    #[facet(figue::subcommand)]
    command: Command,
}

#[derive(facet::Facet)]
#[repr(u8)]
enum Command {
    /// Inspect a local value.
    Inspect(Inspect),
}

#[derive(facet::Facet)]
struct Inspect {
    /// The value to inspect, with a description
    /// continued on the next source line.
    #[facet(figue::positional)]
    value: String,
}

impl ApplicationCli for ConsumerCli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global
    }
}

fn app() -> App {
    App::new("consumer-example", "1.2.3")
}

#[test]
fn consumer_owns_root_and_subcommands_without_debug() {
    let cli = app()
        .parse_from::<ConsumerCli>(["inspect", "hello", "--debug", "--log-level", "warn"])
        .into_result()
        .unwrap()
        .value;
    assert!(cli.global.debug);
    assert_eq!(cli.global.log_filter, "warn");
    let Command::Inspect(inspect) = cli.command;
    assert_eq!(inspect.value, "hello");
}

#[test]
fn shared_defaults_match_explicit_construction() {
    let cli = app()
        .parse_from::<ConsumerCli>(["inspect", "hello"])
        .into_result()
        .unwrap()
        .value;
    let defaults = GlobalArgs::default();
    assert_eq!(cli.global.debug, defaults.debug);
    assert_eq!(cli.global.log_filter, defaults.log_filter);
    assert_eq!(cli.global.log_file_filter, defaults.log_file_filter);
    assert_eq!(cli.global.log_file, defaults.log_file);
    #[cfg(feature = "auth")]
    assert_eq!(cli.global.auth_source, defaults.auth_source);
}

#[test]
fn help_inherits_shared_flags_and_complete_prose() {
    for args in [vec!["--help"], vec!["inspect", "--help"]] {
        let figue::DriverError::Help { text, .. } =
            app().parse_from::<ConsumerCli>(args).unwrap_err()
        else {
            panic!("expected help");
        };
        for flag in [
            "--[no-]debug",
            "--log-filter <DIRECTIVE>",
            "--log-file-filter <DIRECTIVE>",
            "--log-file <FILE|DIR>",
            "--[no-]help",
            "--[no-]version",
            "--[no-]html-help",
            "--completions",
            "--export-jsonschemas",
        ] {
            assert_eq!(text.matches(flag).count(), 1, "{text}");
        }
        assert!(text.starts_with("consumer-example"), "{text}");
        assert!(!text.contains("AAFC-Cloud/Cloud-Terrastodon"), "{text}");
        let prose = text.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            prose.contains("If omitted, the regular log filter is also used for the file output"),
            "{text}"
        );
        #[cfg(not(feature = "auth"))]
        assert!(!text.contains("--auth-source"));
        #[cfg(feature = "auth")]
        assert_eq!(text.matches("--auth-source <SOURCE>").count(), 1);
    }
}

#[test]
fn caller_version_and_source_identity_are_used() {
    let figue::DriverError::Version { text } =
        app().parse_from::<ConsumerCli>(["--version"]).unwrap_err()
    else {
        panic!("expected version");
    };
    assert!(text.contains("consumer-example 1.2.3"), "{text}");
    let app = app().implementation_github("example/consumer", "abcdef");
    let figue::DriverError::Help { text, .. } = app
        .parse_from::<ConsumerCli>(["inspect", "--help"])
        .unwrap_err()
    else {
        panic!("expected help");
    };
    assert!(
        text.contains("https://github.com/example/consumer/blob/abcdef/"),
        "{text}"
    );
}

#[test]
fn parse_errors_are_non_exiting_and_do_not_prevent_another_parse() {
    assert!(
        app()
            .parse_from::<ConsumerCli>(["inspect", "value", "--unknown"])
            .is_err()
    );
    assert!(
        app()
            .parse_from::<ConsumerCli>(["inspect", "value"])
            .is_ok()
    );
}

#[test]
fn invalid_schema_returns_typed_builder_error() {
    #[derive(facet::Facet)]
    struct Invalid {
        global: GlobalArgs,
    }
    impl ApplicationCli for Invalid {
        fn global_args(&self) -> &GlobalArgs {
            &self.global
        }
    }
    assert!(matches!(
        app()
            .parse_from::<Invalid>(Vec::<String>::new())
            .unwrap_err(),
        figue::DriverError::Builder { .. }
    ));
}

#[test]
fn module_directives_and_independent_file_filter_are_supported() {
    let globals = GlobalArgs {
        log_filter: "consumer_example=trace".into(),
        log_file_filter: Some("warn".into()),
        ..GlobalArgs::default()
    };
    let (console, file) = logging_filters(&globals).unwrap();
    assert_eq!(console.to_string(), "consumer_example=trace");
    assert_eq!(file.unwrap().to_string(), "warn");
}

#[test]
fn debug_selects_console_debug_but_preserves_file_filter() {
    let globals = GlobalArgs {
        debug: true,
        log_filter: "not a valid directive".into(),
        log_file_filter: Some("error".into()),
        ..GlobalArgs::default()
    };
    let (console, file) = logging_filters(&globals).unwrap();
    assert_eq!(console.to_string(), "debug");
    assert_eq!(file.unwrap().to_string(), "error");
}

#[test]
fn exit_guard_requests_cooperative_cancellation() {
    let token = CancellationToken::new();
    let guard = CancelOnDrop(token.clone());
    assert!(!token.is_cancelled());
    drop(guard);
    assert!(token.is_cancelled());
}

#[test]
fn nested_runtime_is_rejected_without_invoking_handler() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(async {
        let cli = app()
            .parse_from::<ConsumerCli>(["inspect", "value"])
            .into_result()
            .unwrap()
            .value;
        let error = app()
            .run_parsed(cli, |_, _| async { panic!("must not invoke") })
            .unwrap_err();
        assert!(error.to_string().contains("outside a Tokio runtime"));
    });
}
