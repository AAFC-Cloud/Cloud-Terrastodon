use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolEntitlement;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolEntitlementListForProjectRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsBuildDefinitionReference;
use cloud_terrastodon_azure_devops::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectWorkItemListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsRepo;
use cloud_terrastodon_azure_devops::AzureDevOpsRepositoriesListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpoint;
use cloud_terrastodon_azure_devops::AzureDevOpsServiceEndpointsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsTeam;
use cloud_terrastodon_azure_devops::AzureDevOpsTeamsForProjectRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsTestPlan;
use cloud_terrastodon_azure_devops::AzureDevOpsTestPlanListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItem;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use std::borrow::Cow;
use std::num::NonZeroU16;
use std::sync::Arc;

pub(crate) const WORK_ITEM_LIMIT: NonZeroU16 = NonZeroU16::MIN.saturating_add(200 - 1);

/// Inventory for a selected Azure DevOps project section.
#[derive(Debug)]
pub(crate) enum AzureDevOpsProjectData {
    Repositories(Arc<Vec<AzureDevOpsRepo>>),
    WorkItems(Arc<Vec<Arc<AzureDevOpsWorkItem>>>),
    Groups(Vec<Arc<AzureDevOpsGroup>>),
    Teams(Vec<Arc<AzureDevOpsTeam>>),
    ServiceConnections(Arc<Vec<AzureDevOpsServiceEndpoint>>),
    AgentPools(Vec<Arc<AzureDevOpsAgentPoolEntitlement>>),
    Pipelines(Vec<Arc<AzureDevOpsBuildDefinitionReference>>),
    TestPlans(Vec<Arc<AzureDevOpsTestPlan>>),
}

impl AzureDevOpsProjectData {
    pub async fn load(
        view: PortalAzureDevOpsProjectView,
        organization: &AzureDevOpsOrganizationUrl,
        project_id: AzureDevOpsProjectId,
        auth_selection: AzureDevOpsAuthSelection,
        refresh: bool,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Self> {
        let org_url = Cow::Borrowed(organization);
        match view {
            PortalAzureDevOpsProjectView::Overview => {
                bail!("The overview uses the organization project inventory");
            }
            PortalAzureDevOpsProjectView::Repositories => AzureDevOpsRepositoriesListRequest {
                org_url,
                project_id,
                auth_selection,
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await
            .map(Self::Repositories),
            PortalAzureDevOpsProjectView::WorkItems => AzureDevOpsProjectWorkItemListRequest {
                org_url,
                project_id,
                auth_selection,
                limit: WORK_ITEM_LIMIT,
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await
            .map(Self::WorkItems),
            PortalAzureDevOpsProjectView::Groups => AzureDevOpsGroupsListRequest {
                org_url,
                project: project_id.into(),
                auth_selection,
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await
            .map(Self::Groups),
            PortalAzureDevOpsProjectView::Teams => AzureDevOpsTeamsForProjectRequest {
                org_url,
                project: project_id.into(),
                auth_selection,
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await
            .map(Self::Teams),
            PortalAzureDevOpsProjectView::ServiceConnections => {
                AzureDevOpsServiceEndpointsListRequest {
                    org_url,
                    project: project_id.into(),
                    auth_selection,
                }
                .execute(executor)
                .with_invalidation(refresh)
                .await
                .map(Self::ServiceConnections)
            }
            PortalAzureDevOpsProjectView::AgentPools => {
                AzureDevOpsAgentPoolEntitlementListForProjectRequest {
                    org_url,
                    project: project_id.into(),
                    auth_selection,
                }
                .execute(executor)
                .with_invalidation(refresh)
                .await
                .map(Self::AgentPools)
            }
            PortalAzureDevOpsProjectView::Pipelines => AzureDevOpsBuildDefinitionListRequest {
                org_url,
                project: project_id.into(),
                auth_selection,
                name: None,
                path: None,
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await
            .map(Self::Pipelines),
            PortalAzureDevOpsProjectView::TestPlans => AzureDevOpsTestPlanListRequest {
                org_url,
                project: project_id.into(),
                auth_selection,
            }
            .execute(executor)
            .with_invalidation(refresh)
            .await
            .map(Self::TestPlans),
        }
    }
}
