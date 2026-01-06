use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsAgentPackage;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

pub struct AzureDevOpsAgentPackagesRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
}

pub fn fetch_azure_devops_agent_packages<'a>(org_url: &'a AzureDevOpsOrganizationUrl) -> AzureDevOpsAgentPackagesRequest<'a> {
    AzureDevOpsAgentPackagesRequest { org_url }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsAgentPackagesRequest<'a> {
    type Output = Vec<AzureDevOpsAgentPackage>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter(["az", "devops", "packages", "list", "agent"]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps agent packages");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "distributedtask"]);
        cmd.args(["--resource", "packages"]);
        cmd.args(["--route-parameters", "packageType=agent"]);
        cmd.args(["--api-version", "7.2-preview"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.cache(self.cache_key());

        #[derive(Deserialize)]
        struct InvokeResponse {
            continuation_token: Option<Value>,
            count: u32,
            value: Vec<AzureDevOpsAgentPackage>,
        }

        let resp = cmd.run::<InvokeResponse>().await?;
        let packages = resp.value;

        debug!("Found {} Azure DevOps agent packages", resp.count);

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

        Ok(packages)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsAgentPackagesRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let pkgs = fetch_azure_devops_agent_packages(&org_url).await?;
        println!("Found {} packages", pkgs.len());
        for p in pkgs.iter().take(5) {
            println!("{} - {} ({})", p.filename, p.platform, p.version.major);
        }

        Ok(())
    }
}
