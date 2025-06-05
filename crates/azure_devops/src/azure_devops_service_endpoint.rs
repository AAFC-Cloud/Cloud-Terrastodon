use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

pub async fn fetch_all_azure_devops_service_endpoints(
    organization: &AzureDevOpsOrganizationUrl,
    project: &AzureDevOpsProjectName,
) -> eyre::Result<Vec<serde_json::Value>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "service-endpoint",
        "list",
        "--organization",
        &organization.to_string(),
        "--project",
        &project.to_string(),
        "--output",
        "json",
    ]);

    let response = cmd.run_raw().await?;
    let endpoints: Vec<serde_json::Value> = serde_json::from_slice(&response.stdout)?;

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

        println!("{:?}", service_endpoints);

        Ok(())
    }
}
