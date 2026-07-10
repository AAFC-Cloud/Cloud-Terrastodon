use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPackage;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use facet_json::RawJson;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsAgentPackagesRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
}

pub fn fetch_azure_devops_agent_packages<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
) -> AzureDevOpsAgentPackagesRequest<'a> {
    AzureDevOpsAgentPackagesRequest {
        org_url: Cow::Borrowed(org_url),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPackagesRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsAgentPackagesRequest<'a> {
    type Output = Vec<AzureDevOpsAgentPackage>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "packages",
            "list",
            "agent",
        ]))
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

        #[derive(facet::Facet)]
        struct InvokeResponse {
            continuation_token: Option<RawJson<'static>>,
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
cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPackagesRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPackagesRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsAgentPackagesRequest<'static> => Vec<AzureDevOpsAgentPackage>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let pkgs = fetch_azure_devops_agent_packages(&org_url).await?;
        assert!(
            !pkgs.is_empty(),
            "Expected at least one Azure DevOps agent package"
        );
        assert!(
            pkgs.iter()
                .all(|p| !p.filename.is_empty() && !p.platform.is_empty() && p.version.major > 0),
            "Expected sampled Azure DevOps agent packages to include filename, platform, and version"
        );

        Ok(())
    }
}
use arbitrary::Arbitrary;
