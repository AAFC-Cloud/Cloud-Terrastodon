use crate::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_azure_devops_agent_pool_entitlements_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    AzureDevOpsAgentPoolEntitlementListForProjectRequest {
        org_url,
        project: project.into(),
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand
    for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a>
{
    type Output = Vec<crate::prelude::AzureDevOpsAgentPoolEntitlement>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "distributedtask",
            "queue",
            "list",
            "--project",
            &self.project.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let project = &self.project;
        debug!("Fetching Azure DevOps agent queues (pools) for project {project}");

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "distributedtask"]);
        cmd.args(["--resource", "queues"]);
        let route = format!("project={}", project.to_string());
        cmd.args(["--route-parameters", route.as_str()]);
        cmd.args(["--api-version", "7.1"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.cache(self.cache_key());

        #[derive(Deserialize)]
        struct Response {
            continuation_token: Option<Value>,
            count: u32,
            value: Vec<crate::prelude::AzureDevOpsAgentPoolEntitlement>,
        }

        let resp = cmd.run::<Response>().await?;
        let entitlements = resp.value;

        debug!(
            "Found {} Azure DevOps agent queue entitlements for project {}",
            resp.count, project
        );

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

        Ok(entitlements)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;

        // Iterate projects, and stop when we find the first project with entitlements.
        let mut found = false;
        for project in projects {
            let entitlements =
                fetch_azure_devops_agent_pool_entitlements_for_project(&org_url, &project.name).await?;
            if !entitlements.is_empty() {
                println!(
                    "Found {} Azure DevOps queue/pool entitlements",
                    entitlements.len()
                );
                found = true;
                break;
            }
        }

        assert!(
            found,
            "Expected at least one Azure DevOps queue/pool entitlement across projects"
        );
        Ok(())
    }
}
