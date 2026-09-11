use arbitrary::Arbitrary;
pub mod command;
pub mod global_args;
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
        match self.command {
            Some(cmd) => cmd.invoke(cancellation_token, auth_context).await,
            None => {
                menu_loop(auth_context).await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::azure::azure_command_cli::AzureCommand;
    use crate::cli::azure::pim::AzurePimCommand;
    use crate::cli::azure::tenant::AzureTenantCommand;
    use crate::cli::azure_devops::azure_devops_command_cli::AzureDevOpsCommand;
    use crate::cli::azure_devops::project::AzureDevOpsProjectCommand;
    use cloud_terrastodon_credentials::AuthSource;

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
