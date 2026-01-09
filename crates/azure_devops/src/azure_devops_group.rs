use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsGroupsListRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_azure_devops_groups_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsGroupsListRequest<'a> {
    AzureDevOpsGroupsListRequest {
        org_url,
        project: project.into(),
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsGroupsListRequest<'a> {
    type Output = Vec<AzureDevOpsGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "security",
            "group",
            "list",
            "--project",
            &self.project.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let project = &self.project;
        debug!("Fetching Azure DevOps groups for project {project}");

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        let org = self.org_url.to_string();
        cmd.args([
            "devops",
            "security",
            "group",
            "list",
            "--organization",
            org.as_str(),
            "--project",
            &project.to_string(),
            "--output",
            "json",
        ]);
        cmd.cache(self.cache_key());

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            continuation_token: Option<String>,
            graph_groups: Vec<AzureDevOpsGroup>,
        }

        let response = cmd.run::<Response>().await?;
        assert!(
            response.continuation_token.is_none(),
            "Continuation token found in Azure DevOps group list response"
        );

        debug!(
            "Found {} Azure DevOps groups for project {}",
            response.graph_groups.len(),
            project
        );
        Ok(response.graph_groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsGroupsListRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_azure_devops_groups_for_project;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let project = fetch_all_azure_devops_projects(&org_url)
            .await?
            .into_iter()
            .next()
            .expect("No Azure DevOps projects found");
        let groups = fetch_azure_devops_groups_for_project(&org_url, &project.name).await?;
        assert!(
            !groups.is_empty(),
            "Expected at least one Azure DevOps group"
        );
        for group in &groups {
            println!("Group: {:#?}", group);
        }
        Ok(())
    }
}
