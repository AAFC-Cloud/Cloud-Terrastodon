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
use eyre::Result;

pub fn entrypoint(
    version: Version,
    git_rev: GitRevision,
    build_timestamp: BuildTimestamp,
) -> Result<()> {
    cloud_terrastodon_app::install_error_hook()?;

    let implementation_revision = git_rev.to_string();
    set_git_revision(git_rev);
    set_build_timestamp(build_timestamp);
    set_version(version);

    let executable_name = std::env::args_os()
        .next()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "cloud_terrastodon".into());
    let app = App::new(executable_name, full_version().to_string())
        .implementation_github("AAFC-Cloud/Cloud-Terrastodon", implementation_revision);
    let cli: Cli = app.parse_from(std::env::args_os().skip(1)).unwrap();
    let enable_egui = matches!(cli.command.as_ref(), Some(CloudTerrastodonCommand::Egui(_)));
    app.egui_collector(enable_egui)
        .run_parsed(cli, |cli, context| async move {
            cli.invoke(&context.cancellation, &context.auth).await
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::EntraUser;
    use cloud_terrastodon_azure::EntraUserPickRequest;
    use cloud_terrastodon_registry::ArbitraryBytes;
    use cloud_terrastodon_registry::Function;
    use cloud_terrastodon_registry::FunctionKind;
    use cloud_terrastodon_registry::Thing;
    use cloud_terrastodon_registry::describe_shape;
    use cloud_terrastodon_registry::functions_from_to;
    use cloud_terrastodon_registry::known_functions;
    use cloud_terrastodon_registry::known_things;
    use facet::Facet;
    use std::collections::BTreeMap;

    #[test]
    fn cli_schema_builds() {
        let _ = figue::builder::<Cli>()
            .expect("CLI schema should be valid")
            .help(|help| {
                help.version(full_version().to_string())
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
        for arguments in [vec!["--help"], vec!["az", "tenant", "login", "--help"]] {
            let help = rendered_cli_help(&arguments);
            for spelling in [
                "--auth-source <SOURCE>",
                "--[no-]debug",
                "--log-filter <DIRECTIVE>",
                "--log-file-filter <DIRECTIVE>",
                "--log-file <FILE|DIR>",
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
            assert!(
                prose.contains("configured and otherwise defaults to Azure CLI"),
                "{help}"
            );
            assert!(
                prose.contains("can be selected explicitly or configured per tenant"),
                "{help}"
            );
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
            "login",
            "example-tenant",
            "--auth-source",
            "workload-identity",
            "--debug",
            "--log-level",
            "trace",
            "--log-file-filter",
            "debug",
            "--log-file",
            "logs.ndjson",
        ])
        .unwrap();
        assert_eq!(
            cli.global_args.auth_source,
            cloud_terrastodon_credentials::AuthSource::WorkloadIdentity
        );
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
            .into_iter()
            .filter(|function| {
                function.input_shape.is_shape(EntraUserPickRequest::SHAPE)
                    && function.output_shape.is_shape(Vec::<EntraUser>::SHAPE)
                    && function.is_async()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            registrations.len(),
            1,
            "expected one Entra user picker registration"
        );
        assert_eq!(registrations[0].label, "invoke");
    }

    #[test]
    fn async_request_outputs_have_fake_response_generators() {
        let mut missing = known_functions()
            .into_iter()
            .filter(|function| function.kind == FunctionKind::AsyncInvoke)
            .filter(|function| {
                !known_functions()
                    .into_iter()
                    .filter(|candidate| candidate.kind == FunctionKind::Constructor)
                    .any(|candidate| {
                        describe_shape(candidate.input_shape) == describe_shape(ArbitraryBytes::SHAPE)
                            && candidate.production_kind(function.output_shape).is_some()
                    })
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
    fn registry_lists_missing_arbitrary_companion_registrations() {
        let mut missing = known_things()
            .into_iter()
            .filter(|thing| !thing.shape.is_shape(ArbitraryBytes::SHAPE))
            .filter(|thing| {
                functions_from_to(ArbitraryBytes::SHAPE, thing.shape)
                    .into_iter()
                    .next()
                    .is_none()
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

    fn function_signature(function: &Function) -> String {
        format!(
            "{} {} -> {} ({:?} | {:?} | {})",
            function.label,
            describe_shape(function.input_shape),
            describe_shape(function.output_shape),
            function.receiver_mode,
            function.kind,
            function.origin,
        )
    }

    fn format_function_registration(function: &Function) -> String {
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
