use crate::BuildTimestamp;
use crate::Version;
use crate::build_timestamp::set_build_timestamp;
use crate::cli::Cli;
use crate::cli::CloudTerrastodonCommand;
use crate::git_revision::GitRevision;
use crate::git_revision::set_git_revision;
use crate::version::full_version;
use crate::version::set_version;
use cloud_terrastodon_tracing::init_tracing;
use eyre::Result;
use figue::Driver;
use std::str::FromStr;
use teamy_cancellation::CtrlCHandler;
use tracing::level_filters::LevelFilter;

pub fn entrypoint(
    version: Version,
    git_rev: GitRevision,
    build_timestamp: BuildTimestamp,
) -> Result<()> {
    // Track version information globally
    set_git_revision(git_rev);
    set_build_timestamp(build_timestamp);
    set_version(version);

    color_eyre::install()?;

    // Parse command line arguments.
    let cli: Cli = Driver::new(
        figue::builder::<Cli>()
            .expect("CLI schema should be valid")
            .cli(|cli| cli.args_os(std::env::args_os().skip(1)).strict())
            .help(|help| help.version(full_version().to_string()))
            .build(),
    )
    .run()
    .unwrap();

    // Configure backtrace-always
    if cli.global_args.debug {
        unsafe { std::env::set_var("RUST_BACKTRACE", "full") };
        // std::env::set_var("RUST_BACKTRACE", "1");
    }

    // Configure tracing
    init_tracing(
        match cli.global_args.debug {
            true => LevelFilter::DEBUG,
            false => LevelFilter::from_str(&cli.global_args.log_filter)?,
        },
        cli.global_args.log_file.as_ref(),
        matches!(cli.command.as_ref(), Some(CloudTerrastodonCommand::Egui(_))),
    )?;

    // Configure terminal colour support
    #[cfg(windows)]
    let _ = crate::windows_support::windows_ansi::enable_ansi_support();

    // Warn if UTF-8 support is not enabled on Windows.
    #[cfg(windows)]
    if !crate::windows_support::windows_utf8::is_system_utf8() {
        tracing::warn!("The current system codepage is not UTF-8. This may cause '�' problems.");
        tracing::warn!(
            "See https://github.com/Azure/azure-cli/issues/22616#issuecomment-1147061949"
        );
        tracing::warn!(
            "Control panel -> Clock and Region -> Region -> Administrative -> Change system locale -> Check Beta: Use Unicode UTF-8 for worldwide language support."
        );
    }

    let cancellation_token = CtrlCHandler::default().install()?;

    // Build async runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(cli.invoke(&cancellation_token))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_registry::{describe_shape, known_functions, known_things, Function, Thing};
    use std::collections::BTreeMap;

    #[test]
    fn cli_schema_builds() {
        let _ = figue::builder::<Cli>()
            .expect("CLI schema should be valid")
            .help(|help| help.version(full_version().to_string()))
            .build();
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

    fn collect_duplicates(by_key: BTreeMap<String, Vec<String>>) -> Vec<String> {
        by_key
            .into_iter()
            .filter_map(|(key, registrations)| {
                (registrations.len() > 1).then(|| {
                    format!("{key}\n  {}", registrations.join("\n  "))
                })
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

