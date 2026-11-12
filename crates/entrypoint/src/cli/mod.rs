use arbitrary::Arbitrary;
use facet::Facet;
pub mod command;
pub mod global_args;
pub mod output {
    pub use cloud_terrastodon_app::CliOutput;
    pub use cloud_terrastodon_app::CliOutputValue;
    pub use cloud_terrastodon_app::OutputFormat;
}

use crate::menu::menu_loop;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
pub use command::*;
pub use global_args::GlobalArgs;

/// The primary entrypoint for command-line parsing.
#[derive(Facet, Debug)]
#[facet(rename = "cloud_terrastodon")]
pub struct Cli {
    #[facet(flatten)]
    pub global_args: GlobalArgs,

    #[facet(figue::subcommand, default)]
    pub command: Option<CloudTerrastodonCommand>,
}

impl cloud_terrastodon_app::ApplicationCli for Cli {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

impl<'a> Arbitrary<'a> for Cli {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            global_args: GlobalArgs::arbitrary(u)?,
            command: Option::<CloudTerrastodonCommand>::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_thing!(Cli);
cloud_terrastodon_registry::register_arbitrary!(Cli);
impl Cli {
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<CliOutput> {
        let retain_report = matches!(&self.command, Some(CloudTerrastodonCommand::Test(_)));
        // Keep command and interactive-menu futures out of this outer future's
        // inline state; nested debug poll frames otherwise exhaust small stacks.
        let output = match self.command {
            Some(cmd) => Box::pin(cmd.invoke(cancellation, executor)).await?,
            None => {
                Box::pin(menu_loop(executor)).await?;
                CliOutput::none()
            }
        };
        Ok(if retain_report {
            output.with_required_emission("emit runtime verification report")
        } else {
            output
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::azure::AzureArgs;
    use crate::cli::azure::azure_command_cli::AzureCommand;
    use crate::cli::azure::pim::AzurePimCommand;
    use crate::cli::azure::subscription::AzureSubscriptionCommand;
    use crate::cli::azure_devops::AzureDevOpsArgs;
    use crate::cli::azure_devops::azure_devops_command_cli::AzureDevOpsCommand;
    use crate::cli::azure_devops::build::AzureDevOpsBuildArgs;
    use crate::cli::azure_devops::build::AzureDevOpsBuildCommand;
    use crate::cli::azure_devops::build::definition::AzureDevOpsBuildDefinitionArgs;
    use crate::cli::azure_devops::build::definition::AzureDevOpsBuildDefinitionCommand;
    use crate::cli::azure_devops::build::definition::folder::AzureDevOpsBuildDefinitionFolderArgs;
    use crate::cli::azure_devops::build::definition::folder::AzureDevOpsBuildDefinitionFolderCommand;
    use crate::cli::azure_devops::build::definition::folder::list::AzureDevOpsBuildDefinitionFolderListArgs;
    use crate::cli::azure_devops::project::AzureDevOpsProjectCommand;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_command::CancellationToken;

    #[test]
    fn nested_devops_work_item_preserves_leaf_auth_and_tenant_selection() {
        let cli: Cli = figue::from_slice(&[
            "az",
            "devops",
            "work-item",
            "show",
            "--auth-source",
            "browser",
            "1",
            "--tenant",
            "synthetic-tenant",
            "--org",
            "synthetic",
            "--project",
            "Synthetic project",
        ])
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get();
        let Some(CloudTerrastodonCommand::Azure(AzureArgs {
            command:
                AzureCommand::DevOps(AzureDevOpsArgs {
                    command: AzureDevOpsCommand::WorkItem(args),
                }),
        })) = cli.command
        else {
            panic!("expected nested Azure DevOps work item command");
        };
        let azure_devops::work_item::AzureDevOpsWorkItemCommand::Show(args) = args.command else {
            panic!("expected work item show");
        };
        assert_eq!(args.authentication.auth_source, AuthSource::Browser);
        assert_eq!(
            args.authentication.tenant.unwrap().to_string(),
            "synthetic-tenant"
        );
        assert_eq!(args.org.name(), "synthetic");
        assert_eq!(args.project.to_string(), "Synthetic project");
    }

    #[test]
    fn build_folder_dispatch_reports_auth_error_on_a_small_stack() -> eyre::Result<()> {
        // Windows main normally has a 1 MiB stack. Leave room for application,
        // tracing, REST, and runtime frames by exercising dispatch with half that.
        std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(|| {
                let folder = AzureDevOpsBuildDefinitionFolderArgs {
                    command: AzureDevOpsBuildDefinitionFolderCommand::List(
                        AzureDevOpsBuildDefinitionFolderListArgs {
                            org: "https://dev.azure.com/example".parse().unwrap(),
                            project: "offline-project".parse().unwrap(),
                            authentication:
                                cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs {
                                    auth_source: AuthSource::None,
                                    ..Default::default()
                                },
                            no_cache: false,
                            path: None,
                        },
                    ),
                };
                let definition = AzureDevOpsBuildDefinitionArgs {
                    command: AzureDevOpsBuildDefinitionCommand::Folder(folder),
                };
                let build = AzureDevOpsBuildArgs {
                    command: AzureDevOpsBuildCommand::Definition(definition),
                };
                let devops = AzureDevOpsArgs {
                    command: AzureDevOpsCommand::Build(build),
                };
                let azure = AzureArgs {
                    command: AzureCommand::DevOps(devops),
                };
                let cli = Cli {
                    global_args: GlobalArgs::default(),
                    command: Some(CloudTerrastodonCommand::Azure(azure)),
                };
                // Disabled authentication must fail while configuring REST auth,
                // before credentials, cached output, or HTTP are accessed.
                let cancellation_token = CancellationToken::new();
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                let error = runtime
                    .block_on(Box::pin(cancellation_token.with_async_scope(
                        |cancellation| async move {
                            let root = tempfile::tempdir()?;
                            let executor = CloudTerrastodonExecutor::new(
                                cloud_terrastodon_cache::WorkCache::new(root.path())?,
                                std::sync::Arc::new(
                                    cloud_terrastodon_execution::AnonymousAuthenticationProvider,
                                ),
                                cancellation.clone(),
                            );
                            cli.invoke(&cancellation, &executor).await
                        },
                    )))
                    .expect_err("disabled authentication should fail locally");
                assert!(error.chain().any(|cause| {
                    cause
                        .to_string()
                        .contains("authentication is disabled by auth source none")
                }));
            })?
            .join()
            .expect("command dispatch should fit on a small stack");
        Ok(())
    }

    #[test]
    fn subscription_list_parses_global_output_format_before_and_after_subcommands() {
        for arguments in [
            vec!["--output-format", "json", "az", "sub", "list"],
            vec!["az", "subscription", "list", "--output-format", "json"],
        ] {
            let cli: Cli = figue::from_slice(&arguments)
                .into_result()
                .expect("synthetic CLI arguments should parse")
                .get();
            assert_eq!(cli.global_args.output_format, Some(OutputFormat::Json));
            let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
                panic!("expected Azure command");
            };
            let AzureCommand::Subscription(subscription) = azure.command else {
                panic!("expected subscription command");
            };
            let AzureSubscriptionCommand::List(list) = subscription.command;
            assert!(list.tenant.is_none());
        }
    }

    #[test]
    fn subscription_list_parses_a_tenant_filter_and_explicit_text() {
        let cli: Cli = figue::from_slice(&[
            "az",
            "sub",
            "list",
            "--tenant",
            "agr",
            "--output-format",
            "text",
        ])
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get();
        assert_eq!(cli.global_args.output_format, Some(OutputFormat::Text));
        let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
            panic!("expected Azure command");
        };
        let AzureCommand::Subscription(subscription) = azure.command else {
            panic!("expected subscription command");
        };
        let AzureSubscriptionCommand::List(list) = subscription.command;
        assert_eq!(list.tenant.unwrap().to_string(), "agr");
    }

    #[test]
    fn shared_output_format_parses_for_existing_format_aware_commands() {
        for arguments in [
            vec![
                "rest",
                "--method",
                "GET",
                "--url",
                "https://example.com",
                "--output-format",
                "json",
            ],
            vec![
                "az",
                "entra",
                "group",
                "show",
                "--auth-source",
                "browser",
                "--group-id",
                "11111111-1111-1111-1111-111111111111",
                "--output",
                "text",
            ],
            vec![
                "az",
                "entra",
                "group",
                "show",
                "--auth-source",
                "browser",
                "--group-id",
                "11111111-1111-1111-1111-111111111111",
                "--output-format",
                "auto",
            ],
            vec!["az", "sub", "list", "--output-format", "facet-pretty"],
            vec![
                "rest",
                "--method",
                "GET",
                "--url",
                "https://example.com",
                "--output-format",
                "facet-pretty",
            ],
            vec![
                "az",
                "entra",
                "group",
                "show",
                "--auth-source",
                "browser",
                "--group-id",
                "11111111-1111-1111-1111-111111111111",
                "--output-format",
                "facet-pretty",
            ],
        ] {
            let cli: Cli = figue::from_slice(&arguments)
                .into_result()
                .expect("synthetic CLI arguments should parse")
                .get();
            assert!(cli.global_args.output_format.is_some());
        }
    }

    #[test]
    fn parses_the_linux_project_list_vertical_slice() {
        let cli: Cli =
            figue::from_slice(&["az", "devops", "project", "list", "--org", "synthetic"])
                .into_result()
                .expect("synthetic CLI arguments should parse")
                .get();
        let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
            panic!("expected the az command alias");
        };
        let AzureCommand::DevOps(devops) = azure.command else {
            panic!("expected the devops command alias");
        };
        let AzureDevOpsCommand::Project(project) = devops.command else {
            panic!("expected the project command");
        };
        assert!(matches!(
            project.command,
            AzureDevOpsProjectCommand::List(_)
        ));
    }

    #[test]
    fn parses_the_unauthenticated_pim_client_id_bootstrap() {
        let cli: Cli = figue::from_slice(&[
            "az",
            "pim",
            "setup",
            "--tenant",
            "agr",
            "--client-id",
            "11111111-1111-1111-1111-111111111111",
        ])
        .into_result()
        .expect("synthetic CLI arguments should parse")
        .get();
        let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
            panic!("expected the az command alias");
        };
        let AzureCommand::Pim(pim) = azure.command else {
            panic!("expected the pim command");
        };
        let AzurePimCommand::Setup(setup) = pim.command else {
            panic!("expected the setup command");
        };
        assert_eq!(setup.authentication.tenant.to_string(), "agr");
        assert_eq!(
            setup.client_id.map(|client_id| client_id.to_string()),
            Some("11111111-1111-1111-1111-111111111111".to_string())
        );
    }
}
