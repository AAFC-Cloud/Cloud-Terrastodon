use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProject;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use tracing::debug;
use tracing::field::debug;

pub struct AzureDevOpsProjectsListRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
}

pub fn fetch_all_azure_devops_projects<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
) -> AzureDevOpsProjectsListRequest<'a> {
    AzureDevOpsProjectsListRequest { org_url }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsProjectsListRequest<'a> {
    type Output = Vec<AzureDevOpsProject>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "project",
            "list",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching Azure DevOps projects");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "devops",
            "project",
            "list",
            "--organization",
            self.org_url.to_string().as_ref(),
            "--output",
            "json",
        ]);
        cmd.cache(self.cache_key());

        #[derive(Serialize, Deserialize)]
        pub struct Response {
            #[serde(rename = "continuationToken")]
            continuation_token: Option<String>,
            value: Vec<AzureDevOpsProject>,
        }

        let mut projects = Vec::new();
        let mut response = cmd.run::<Response>().await?;
        projects.extend(response.value);

        while let Some(continuation) = &response.continuation_token {
            debug("Fetching the next page of projects");
            let mut next_page_cmd = cmd.clone();
            next_page_cmd.args(["--continuation-token", continuation.as_ref()]);

            response = next_page_cmd.run::<Response>().await?;
            projects.extend(response.value);
        }

        debug!("Found {} Azure DevOps projects", projects.len());
        Ok(projects)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsProjectsListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_projects() -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        assert!(!projects.is_empty());
        for project in projects.iter().take(5) {
            println!("Found project: {project:#?}");
        }
        Ok(())
    }
}
