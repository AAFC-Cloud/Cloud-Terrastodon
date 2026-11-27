use crate::icons::DEVOPS_PROJECT_ICON;
use crate::portal_address::format_shell_command;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure::PercentEncodeExt;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectWorkItemListRequest;
use cloud_terrastodon_azure_devops_cli::agent::pool::entitlement::list::AzureDevOpsAgentPoolEntitlementListArgs;
use cloud_terrastodon_azure_devops_cli::build::definition::list::AzureDevOpsBuildDefinitionListArgs;
use cloud_terrastodon_azure_devops_cli::group::list::AzureDevOpsGroupListArgs;
use cloud_terrastodon_azure_devops_cli::project::show::AzureDevOpsProjectShowArgs;
use cloud_terrastodon_azure_devops_cli::repo::list::AzureDevOpsRepoListArgs;
use cloud_terrastodon_azure_devops_cli::service_endpoint::list::AzureDevOpsServiceEndpointListArgs;
use cloud_terrastodon_azure_devops_cli::team::list::AzureDevOpsTeamListArgs;
use cloud_terrastodon_azure_devops_cli::test::plan::list::AzureDevOpsTestPlanListArgs;
use cloud_terrastodon_azure_devops_cli::work_item::recent::AzureDevOpsWorkItemRecentArgs;
use eframe::egui::ImageSource;
use eframe::egui::include_image as i;
use eyre::Result;
use facet::Facet;

/// A section of an Azure DevOps project's resource page.
#[derive(Clone, Copy, Debug, Eq, Facet, Hash, PartialEq)]
#[repr(u8)]
pub enum PortalAzureDevOpsProjectView {
    Overview,
    Repositories,
    WorkItems,
    Groups,
    Teams,
    ServiceConnections,
    AgentPools,
    Pipelines,
    TestPlans,
}

impl PortalAzureDevOpsProjectView {
    pub(crate) const ALL: [Self; 9] = [
        Self::Overview,
        Self::Repositories,
        Self::WorkItems,
        Self::Groups,
        Self::Teams,
        Self::ServiceConnections,
        Self::AgentPools,
        Self::Pipelines,
        Self::TestPlans,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Repositories => "Repos",
            Self::WorkItems => "Work items",
            Self::Groups => "Groups",
            Self::Teams => "Teams",
            Self::ServiceConnections => "Service connections",
            Self::AgentPools => "Agent pools",
            Self::Pipelines => "Pipelines",
            Self::TestPlans => "Test plans",
        }
    }

    pub(crate) fn icon(self) -> ImageSource<'static> {
        match self {
            Self::Overview => DEVOPS_PROJECT_ICON,
            Self::Repositories => i!("../assets/devops-repos.svg"),
            Self::WorkItems => i!("../assets/devops-work-items.svg"),
            Self::Groups => i!("../assets/access-control.svg"),
            Self::Teams => i!("../assets/entra-group.svg"),
            Self::ServiceConnections => i!("../assets/devops-service-connections.svg"),
            Self::AgentPools => i!("../assets/devops-agent-pools.svg"),
            Self::Pipelines => i!("../assets/devops-pipelines.svg"),
            Self::TestPlans => i!("../assets/devops-test-plans.svg"),
        }
    }

    pub(crate) fn target(
        self,
        organization: &AzureDevOpsOrganizationUrl,
        project: AzureDevOpsProjectId,
    ) -> PortalTabTarget {
        PortalTabTarget::AzureDevOpsProject {
            organization: Box::new(organization.clone()),
            project,
            view: self,
        }
    }

    pub(crate) fn cli_command(
        self,
        organization: &AzureDevOpsOrganizationUrl,
        project: AzureDevOpsProjectId,
        authentication: AzureDevOpsAuthenticationArgs,
    ) -> Result<Option<String>> {
        let project = project.into();
        let org = organization.clone();
        let command = match self {
            Self::Overview => format_shell_command(
                "ct az devops project show".to_owned(),
                &AzureDevOpsProjectShowArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                },
            ),
            Self::Repositories => format_shell_command(
                "ct az devops repo list".to_owned(),
                &AzureDevOpsRepoListArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                },
            ),
            Self::WorkItems => format_shell_command(
                "ct az devops work-item recent".to_owned(),
                &AzureDevOpsWorkItemRecentArgs {
                    org,
                    project,
                    authentication,
                    limit: AzureDevOpsProjectWorkItemListRequest::DEFAULT_LIMIT,
                    no_cache: false,
                },
            ),
            Self::Groups => format_shell_command(
                "ct az devops group list".to_owned(),
                &AzureDevOpsGroupListArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                },
            ),
            Self::Teams => format_shell_command(
                "ct az devops team list".to_owned(),
                &AzureDevOpsTeamListArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                },
            ),
            Self::ServiceConnections => format_shell_command(
                "ct az devops service-endpoint list".to_owned(),
                &AzureDevOpsServiceEndpointListArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                },
            ),
            Self::AgentPools => format_shell_command(
                "ct az devops agent pool entitlement list".to_owned(),
                &AzureDevOpsAgentPoolEntitlementListArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                    pool: None,
                },
            ),
            Self::Pipelines => format_shell_command(
                "ct az devops build definition list".to_owned(),
                &AzureDevOpsBuildDefinitionListArgs {
                    org,
                    project,
                    authentication,
                    no_cache: false,
                    name: None,
                    path: None,
                },
            ),
            Self::TestPlans => format_shell_command(
                "ct az devops test plan list".to_owned(),
                &AzureDevOpsTestPlanListArgs {
                    no_cache: false,
                    org,
                    project,
                    authentication,
                },
            ),
        }?;
        Ok(Some(command))
    }

    /// Browse the [project in Azure DevOps](https://learn.microsoft.com/en-us/azure/devops/project/navigation/).
    pub(crate) fn portal_url(
        self,
        organization: &AzureDevOpsOrganizationUrl,
        project_name: &str,
    ) -> String {
        let section = match self {
            Self::Overview => "",
            Self::Repositories => "/_git",
            Self::WorkItems => "/_workitems",
            Self::Groups => "/_settings/security",
            Self::Teams => "/_settings/teams",
            Self::ServiceConnections => "/_settings/adminservices",
            Self::AgentPools => "/_settings/agentqueues",
            Self::Pipelines => "/_build",
            Self::TestPlans => "/_testPlans",
        };
        format!(
            "{}/{}{section}",
            organization.expanded_form().trim_end_matches('/'),
            project_name.percent_encode()
        )
    }
}
