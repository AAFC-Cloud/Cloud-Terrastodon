use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsServiceEndpoint;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_azure_devops_service_endpoints(
    org_url: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectName,
) -> eyre::Result<Vec<AzureDevOpsServiceEndpoint>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "service-endpoint",
        "list",
        "--organization",
        &org_url.to_string(),
        "--project",
        project.as_ref(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(cloud_terrastodon_command::CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "service-endpoint",
            "list",
            &org_url.organization_name,
            project,
        ]),
        valid_for: Duration::from_hours(8),
    });

    let response = cmd.run_raw().await?;
    let endpoints: Vec<AzureDevOpsServiceEndpoint> = serde_json::from_slice(&response.stdout)?;

    Ok(endpoints)
}

#[cfg(test)]
mod test {
    use crate::prelude::get_default_organization_url;
    use crate::prelude::get_default_project_name;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org = get_default_organization_url().await?;
        let proj = get_default_project_name().await?;
        let service_endpoints =
            super::fetch_all_azure_devops_service_endpoints(&org, &proj).await?;

        println!("{:#?}", service_endpoints);

        Ok(())
    }
}
