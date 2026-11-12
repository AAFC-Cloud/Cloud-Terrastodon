use crate::AzureDevOpsProjectsListRequest;
use crate::azure_devops_agent_pool_entitlement_list_for_project_request::AzureDevOpsAgentPoolEntitlementListForProjectRequest;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::borrow::Cow;
use std::sync::Arc;

/// Lists an agent pool's entitlements across the selected organization's projects.
/// See [Agent Queues](https://learn.microsoft.com/en-us/rest/api/azure/devops/distributedtask/queues/get-agent-queues?view=azure-devops-rest-7.1).
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub pool: AzureDevOpsAgentPoolArgument<'a>,
    pub auth_selection: AzureDevOpsAuthSelection,
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolEntitlementListForPoolRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            pool: AzureDevOpsAgentPoolArgument::arbitrary(u)?.into_owned(),
            auth_selection: AzureDevOpsAuthSelection::arbitrary(u)?,
        })
    }
}

impl CacheInvalidatableRequest for AzureDevOpsAgentPoolEntitlementListForPoolRequest<'_> {
    fn invalidate<'a>(&'a self, executor: &'a CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()> {
        Box::pin(async move {
            let projects = AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(executor)
            .with_invalidation(true)
            .await?;
            for project in projects.iter() {
                executor.cancellation().bail_if_cancelled()?;
                AzureDevOpsAgentPoolEntitlementListForProjectRequest {
                    org_url: Cow::Borrowed(self.org_url.as_ref()),
                    project: (project.as_ref()).into(),
                    auth_selection: self.auth_selection,
                }
                .execute(executor)
                .invalidate()
                .await?;
            }
            Ok(())
        })
    }
}

impl<'a> ExecutableRequest for AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> Result<CloudTerrastodonExecutor, Self::Error> {
        executor.validate_authentication_target(
            &self.auth_selection.into(),
            &self.org_url.api_url("dev.azure.com", "_apis", &[])?,
        )?;
        executor.with_authentication_lease(&self.auth_selection.into())
    }

    type Error = eyre::Report;
    type Output = Vec<Arc<crate::AzureDevOpsAgentPoolEntitlement>>;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let projects = AzureDevOpsProjectsListRequest {
                org_url: Cow::Borrowed(self.org_url.as_ref()),
                auth_selection: self.auth_selection,
            }
            .execute(&executor)
            .await?;
            let mut all_entitlements = Vec::new();
            for project in projects.iter() {
                executor.cancellation().bail_if_cancelled()?;
                let entitlements = AzureDevOpsAgentPoolEntitlementListForProjectRequest {
                    org_url: Cow::Borrowed(self.org_url.as_ref()),
                    project: (project.as_ref()).into(),
                    auth_selection: self.auth_selection,
                }
                .execute(&executor)
                .await?;
                for entitlement in entitlements {
                    if self.pool.matches_entitlement(&entitlement) {
                        all_entitlements.push(entitlement);
                    }
                }
            }
            Ok(all_entitlements)
        })
    }
}

cloud_terrastodon_registry::register_thing!(
    AzureDevOpsAgentPoolEntitlementListForPoolRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsAgentPoolEntitlementListForPoolRequest<'static>
);
cloud_terrastodon_registry_execution::register_executable_request!(
    AzureDevOpsAgentPoolEntitlementListForPoolRequest<'static> => Vec<Arc<crate::AzureDevOpsAgentPoolEntitlement>>,
    effects = [Read]
);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureDevOpsAgentPoolEntitlementListForPoolRequest<'static>,
    name = "azure-devops-agent-pool-entitlement-list-for-pool",
    cached
);
