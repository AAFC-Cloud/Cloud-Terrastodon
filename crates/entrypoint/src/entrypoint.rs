use crate::BuildTimestamp;
use crate::Version;
use crate::build_timestamp::set_build_timestamp;
use crate::cli::Cli;
use crate::cli::CloudTerrastodonCommand;
use crate::git_revision::GitRevision;
use crate::git_revision::set_git_revision;
use crate::version::full_version;
use crate::version::set_version;
use cloud_terrastodon_app::App;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use eyre::Result;
use std::process::ExitCode;
use std::process::Termination;
use std::sync::Arc;

pub fn entrypoint(
    version: Version,
    git_rev: GitRevision,
    build_timestamp: BuildTimestamp,
) -> Result<()> {
    cloud_terrastodon_app::prepare_process()?;

    let implementation_revision = git_rev.to_string();
    set_git_revision(git_rev)?;
    set_build_timestamp(build_timestamp)?;
    set_version(version)?;

    let executable_name = std::env::args_os()
        .next()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "cloud_terrastodon".into());
    // Interactive commands await terminal restoration and background-job shutdown.
    // Keep this owner cooperative; requests and scoped jobs cancel independently.
    let app = App::new(executable_name, full_version()?.to_string())
        .authentication_provider(Arc::new(CredentialAuthenticationProvider::configured()))
        .automatic_cancellation(false)
        .implementation_github("AAFC-Cloud/Cloud-Terrastodon", implementation_revision);
    let cli: Cli = match app.parse_from(std::env::args_os().skip(1)).into_result() {
        Ok(output) => output.get(),
        Err(outcome) => {
            let status = outcome.report();
            std::process::exit(if status == ExitCode::SUCCESS { 0 } else { 1 });
        }
    };
    let enable_egui = matches!(cli.command.as_ref(), Some(CloudTerrastodonCommand::Egui(_)));
    let capture_canvas_logs = matches!(
        cli.command.as_ref(),
        Some(CloudTerrastodonCommand::Excalidraw(_))
    );
    app.egui_collector(enable_egui)
        .terminal_debug_capture(capture_canvas_logs)
        .run_parsed(cli, |cli, context| async move {
            cli.invoke(&context.cancellation, &context.executor()).await
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::CostManagementQueryResult;
    use cloud_terrastodon_azure::EntraUser;
    use cloud_terrastodon_azure::EntraUserPickRequest;
    use cloud_terrastodon_azure::UnifiedRoleDefinitionsAndAssignments;
    use cloud_terrastodon_registry::ARBITRARY_REGISTRATION_POINTS;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use cloud_terrastodon_registry::DiscoveryPlan;
    use cloud_terrastodon_registry::FunctionKind;
    use cloud_terrastodon_registry::THING_REGISTRATION_POINTS;
    use cloud_terrastodon_registry::Thing;
    use cloud_terrastodon_registry::describe_shape;
    use cloud_terrastodon_registry::known_arbitrary_values;
    use cloud_terrastodon_registry::known_things;
    use cloud_terrastodon_registry_execution::Function;
    use cloud_terrastodon_registry_execution::known_functions;
    use facet::Facet;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[test]
    fn cli_schema_builds() {
        let _ = figue::builder::<Cli>()
            .expect("CLI schema should be valid")
            .help(|help| {
                help.version(full_version().unwrap().to_string())
                    .include_implementation_source_file(true)
                    .include_implementation_github_url("AAFC-Cloud/Cloud-Terrastodon", "unknown")
            })
            .build();
    }

    fn rendered_cli_help(arguments: &[&str]) -> String {
        let app = App::new("cloud_terrastodon", "test")
            .implementation_github("AAFC-Cloud/Cloud-Terrastodon", "test");
        match app
            .parse_from::<Cli>(arguments.iter().copied())
            .into_result()
        {
            Err(figue::DriverError::Help { text, .. }) => text,
            other => panic!("expected parse-only help, got {other:?}"),
        }
    }

    #[test]
    fn root_and_tenant_leaf_help_include_all_globals_and_complete_docs() {
        for arguments in [vec!["--help"], vec!["az", "tenant", "default", "--help"]] {
            let help = rendered_cli_help(&arguments);
            for spelling in [
                "--[no-]debug",
                "--log-filter <DIRECTIVE>",
                "--log-file-filter <DIRECTIVE>",
                "--log-file <FILE|DIR>",
                "--output-format <FORMAT>",
                "--[no-]help",
                "--[no-]html-help",
                "--[no-]version",
                "--completions",
                "--export-jsonschemas",
            ] {
                assert_eq!(
                    help.matches(spelling).count(),
                    1,
                    "{arguments:?}: {spelling}\n{help}"
                );
            }
            let prose = help.split_whitespace().collect::<Vec<_>>().join(" ");
            assert!(!prose.contains("--auth-source"), "{help}");
            assert!(
                help.contains("AAFC-Cloud/Cloud-Terrastodon/blob/test/"),
                "{help}"
            );
        }
    }

    #[test]
    fn tenant_leaf_parses_all_global_arguments_without_invocation() {
        let cli: Cli = figue::from_slice(&[
            "az",
            "tenant",
            "default",
            "example-tenant",
            "--debug",
            "--log-level",
            "trace",
            "--log-file-filter",
            "debug",
            "--log-file",
            "logs.ndjson",
        ])
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get();
        assert!(cli.global_args.debug);
        assert_eq!(cli.global_args.log_filter, "trace");
        assert_eq!(cli.global_args.log_file_filter.as_deref(), Some("debug"));
        assert_eq!(
            cli.global_args.log_file.as_deref(),
            Some(std::path::Path::new("logs.ndjson"))
        );
    }

    #[test]
    fn registry_things_have_unique_shapes() {
        let mut by_shape = BTreeMap::<String, Vec<String>>::new();

        for thing in known_things() {
            by_shape
                .entry(describe_shape(thing.shape))
                .or_default()
                .push(format_thing_registration(thing));
        }

        let duplicates = collect_duplicates(by_shape);
        assert!(
            duplicates.is_empty(),
            "duplicate thing registrations found:\n{}",
            duplicates.join("\n")
        );
    }

    #[test]
    fn registry_functions_have_unique_signatures() {
        let mut by_signature = BTreeMap::<String, Vec<String>>::new();

        for function in known_functions() {
            by_signature
                .entry(function_signature(function))
                .or_default()
                .push(format_function_registration(function));
        }

        let duplicates = collect_duplicates(by_signature);
        assert!(
            duplicates.is_empty(),
            "duplicate function registrations found:\n{}",
            duplicates.join("\n")
        );
    }

    #[test]
    fn entra_user_pick_request_is_registered_with_typed_many_output() {
        let registrations = known_functions()
            .iter()
            .filter(|function| {
                function.input_shape.is_shape(EntraUserPickRequest::SHAPE)
                    && function.output_shape.is_shape(Vec::<Arc<EntraUser>>::SHAPE)
                    && function.is_async()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            registrations.len(),
            1,
            "expected one Entra user picker registration"
        );
        assert_eq!(registrations[0].label, "execute");
    }

    #[test]
    fn async_requests_declare_effects() {
        let mut missing = known_functions()
            .iter()
            .filter(|function| function.kind == FunctionKind::AsyncInvoke)
            .filter(|function| function.effects.is_empty())
            .map(|function| {
                format!(
                    "{} at {}",
                    function_signature(function),
                    function.registration_site,
                )
            })
            .collect::<Vec<_>>();
        missing.sort();
        assert!(
            missing.is_empty(),
            "async requests must explicitly declare at least one effect:\n{}",
            missing.join("\n"),
        );
    }

    #[test]
    fn async_request_outputs_have_fake_response_generators() {
        // These semantic results are deliberately outside response simulation.
        // A derived generator would fabricate invalid relationships. Keep this
        // list typed and narrow so new service outputs still require review.
        let unsupported = [
            (
                DiscoveryPlan::SHAPE,
                "a discovery graph must reference the supplied linked catalog",
            ),
            (
                CostManagementQueryResult::SHAPE,
                "cost rows must conform to their corresponding column schema",
            ),
            (
                UnifiedRoleDefinitionsAndAssignments::SHAPE,
                "role assignments must reference the included role definitions",
            ),
        ];
        for (shape, reason) in unsupported {
            assert!(
                !known_arbitrary_values()
                    .iter()
                    .any(|generator| generator.output_shape.is_shape(shape)),
                "{} now has a generator; review its invariant and remove the exception: {reason}",
                describe_shape(shape),
            );
        }
        let mut missing = known_functions()
            .iter()
            .filter(|function| function.kind == FunctionKind::AsyncInvoke)
            .filter(|function| {
                !unsupported
                    .iter()
                    .any(|(shape, _)| function.output_shape.is_shape(shape))
            })
            .filter(|function| {
                !known_arbitrary_values()
                    .iter()
                    .any(|candidate| candidate.output_shape.is_shape(function.output_shape))
            })
            .map(|function| {
                let output_shape = describe_shape(function.output_shape);
                (
                    output_shape.clone(),
                    format!(
                        "{}\n  missing fake response generator for async output {}\n  async request: {} at {}",
                        output_shape,
                        output_shape,
                        function_signature(function),
                        function.registration_site,
                    ),
                )
            })
            .collect::<Vec<_>>();
        missing.sort_by(|left, right| left.0.cmp(&right.0));
        let missing = missing
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "missing fake response generators for async request outputs:\n{}",
            missing.join("\n")
        );
    }
    #[test]
    fn arbitrary_value_registrations_include_all_automatic_collection_forms() {
        // Registering metadata or an editable value does not promise that it
        // can be fabricated. In particular, plans and registered handles depend
        // on the linked catalog. Once both a value and its arbitrary constructor
        // are explicitly registered, all supplied companions must be usable.
        let mut missing = THING_REGISTRATION_POINTS
            .iter()
            .filter(|thing| {
                ARBITRARY_REGISTRATION_POINTS
                    .iter()
                    .any(|generator| generator.output_shape.is_shape(thing.shape))
            })
            .flat_map(|thing| thing.collection_forms())
            .filter(|thing| {
                !known_arbitrary_values()
                    .iter()
                    .any(|generator| generator.output_shape.is_shape(thing.shape))
            })
            .map(|thing| {
                let shape_name = describe_shape(thing.shape);
                (
                    shape_name.clone(),
                    format!(
                        "{}\n  missing: arbitrary {} -> {}\n  registered: {}",
                        shape_name,
                        describe_shape(ArbitraryBytes::SHAPE),
                        describe_shape(thing.shape),
                        format_thing_registration(thing),
                    ),
                )
            })
            .collect::<Vec<_>>();
        missing.sort_by(|left, right| left.0.cmp(&right.0));
        let missing = missing
            .into_iter()
            .map(|(_, message)| message)
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "missing arbitrary companion registrations:\n{}",
            missing.join("\n")
        );
    }

    fn collect_duplicates(by_key: BTreeMap<String, Vec<String>>) -> Vec<String> {
        by_key
            .into_iter()
            .filter_map(|(key, registrations)| {
                (registrations.len() > 1)
                    .then(|| format!("{key}\n  {}", registrations.join("\n  ")))
            })
            .collect()
    }

    fn function_signature(function: Function) -> String {
        format!(
            "{} {} -> {} ({:?} | {:?} | {})",
            function.label,
            describe_shape(function.input_shape),
            describe_shape(function.output_shape),
            function.receiver_mode,
            function.kind,
            function.trait_contract,
        )
    }

    fn format_function_registration(function: Function) -> String {
        format!(
            "effects: {} at {}",
            format_effects(function.effects),
            function.registration_site,
        )
    }

    fn format_thing_registration(thing: &Thing) -> String {
        format!("at {}", thing.registration_site)
    }

    fn format_effects(effects: &[cloud_terrastodon_registry::Effect]) -> String {
        if effects.is_empty() {
            "[]".to_string()
        } else {
            format!("{:?}", effects)
        }
    }
}
