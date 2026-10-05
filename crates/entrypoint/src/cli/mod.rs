use arbitrary::Arbitrary;
pub mod command;
pub mod global_args;
pub mod output {
    pub use cloud_terrastodon_app::CliOutput;
    pub use cloud_terrastodon_app::CliOutputValue;
    pub use cloud_terrastodon_app::OutputFormat;
}
pub(crate) mod scalar_args;

use crate::menu::menu_loop;
use cloud_terrastodon_credentials::AuthContext;
pub use command::*;
pub use global_args::GlobalArgs;
use teamy_cancellation::CancellationToken;

/// The primary entrypoint for command-line parsing.
#[derive(facet::Facet, Debug)]
#[facet(rename = "cloud_terrastodon")]
pub struct Cli {
    #[facet(flatten)]
    pub global_args: GlobalArgs,

    #[facet(flatten)]
    pub builtins: figue::FigueBuiltins,

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
            builtins: figue::FigueBuiltins::default(),
            command: Option::<CloudTerrastodonCommand>::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_thing!(Cli);
cloud_terrastodon_registry::register_arbitrary!(Cli);
impl Cli {
    pub async fn invoke(
        self,
        cancellation_token: &CancellationToken,
        auth_context: &AuthContext,
    ) -> eyre::Result<()> {
        let requested_format = self.global_args.output_format;
        // Keep command and interactive-menu futures out of this outer future's
        // inline state; nested debug poll frames otherwise exhaust small stacks.
        let output = match self.command {
            Some(cmd) => Box::pin(cmd.invoke(cancellation_token, auth_context)).await?,
            None => {
                Box::pin(menu_loop(auth_context)).await?;
                output::CliOutput::none()
            }
        };
        output.emit(requested_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::azure::AzureArgs;
    use crate::cli::azure::azure_command_cli::AzureCommand;
    use crate::cli::azure::pim::AzurePimCommand;
    use crate::cli::azure::subscription::AzureSubscriptionCommand;
    use crate::cli::azure::tenant::AzureTenantCommand;
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
    use cloud_terrastodon_credentials::AuthSource;

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
                            org: Some("https://dev.azure.com/example".parse().unwrap()),
                            project: Some("offline-project".parse().unwrap()),
                            tenant: None,
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
                    builtins: figue::FigueBuiltins::default(),
                    command: Some(CloudTerrastodonCommand::Azure(azure)),
                };
                // The placeholder context must fail while configuring REST auth,
                // before credentials, cached output, or HTTP are accessed.
                let auth_context = AuthContext::None;
                let cancellation_token = CancellationToken::new();
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                let error = runtime
                    .block_on(Box::pin(cli.invoke(&cancellation_token, &auth_context)))
                    .expect_err("placeholder authentication should fail locally");
                assert!(error.chain().any(|cause| {
                    cause
                        .to_string()
                        .contains("Azure DevOps authentication is not configured for this request")
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
            let cli: Cli = figue::from_slice(&arguments).unwrap();
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
        .unwrap();
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
                "--group-id",
                "11111111-1111-1111-1111-111111111111",
                "--output-format",
                "facet-pretty",
            ],
        ] {
            let cli: Cli = figue::from_slice(&arguments).unwrap();
            assert!(cli.global_args.output_format.is_some());
        }
    }

    #[test]
    fn parses_the_linux_project_list_vertical_slice() {
        let cli: Cli = figue::from_slice(&["az", "devops", "project", "list"]).unwrap();
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
    fn parses_the_browser_tenant_login_vertical_slice() {
        let cli: Cli = figue::from_slice(&["az", "tenant", "login", "agr"]).unwrap();
        let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
            panic!("expected the az command alias");
        };
        let AzureCommand::Tenant(tenant) = azure.command else {
            panic!("expected the tenant command");
        };
        let AzureTenantCommand::Login(login) = tenant.command else {
            panic!("expected the login command");
        };
        assert_eq!(login.tenant.to_string(), "agr");
    }

    #[test]
    fn parses_the_tenant_auth_source_setting_vertical_slice() {
        let cli: Cli =
            figue::from_slice(&["az", "tenant", "set-auth-source", "agr", "browser"]).unwrap();
        let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
            panic!("expected the az command alias");
        };
        let AzureCommand::Tenant(tenant) = azure.command else {
            panic!("expected the tenant command");
        };
        let AzureTenantCommand::SetAuthSource(set_auth_source) = tenant.command else {
            panic!("expected the set-auth-source command");
        };
        assert_eq!(set_auth_source.tenant.to_string(), "agr");
        assert_eq!(set_auth_source.auth_source, AuthSource::Browser);
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
        .unwrap();
        let Some(CloudTerrastodonCommand::Azure(azure)) = cli.command else {
            panic!("expected the az command alias");
        };
        let AzureCommand::Pim(pim) = azure.command else {
            panic!("expected the pim command");
        };
        let AzurePimCommand::Setup(setup) = pim.command else {
            panic!("expected the setup command");
        };
        assert_eq!(setup.tenant.to_string(), "agr");
        assert_eq!(
            setup.client_id.map(|client_id| client_id.to_string()),
            Some("11111111-1111-1111-1111-111111111111".to_string())
        );
    }
}
