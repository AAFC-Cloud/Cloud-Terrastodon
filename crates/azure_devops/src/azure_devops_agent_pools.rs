use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPool;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use facet_json::RawJson;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsAgentPoolListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
}

pub fn fetch_azure_devops_agent_pools<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    cancellation: &CancellationScope,
) -> AzureDevOpsAgentPoolListRequest<'a> {
    AzureDevOpsAgentPoolListRequest {
        cancellation: cancellation.clone(),
        org_url: Cow::Borrowed(org_url),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: CancellationScope::arbitrary(u)?,
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsAgentPoolListRequest<'a> {
    type Output = Vec<AzureDevOpsAgentPool>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.name(),
            "agent-pool",
            "list",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps agent pools");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.cancellation_safe();
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "distributedtask"]);
        cmd.args(["--resource", "pools"]);
        cmd.args(["--api-version", "7.2-preview"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.cache(self.cache_key());
        #[derive(facet::Facet)]
        struct InvokeResponse {
            continuation_token: Option<RawJson<'static>>,
            count: u32,
            value: Vec<AzureDevOpsAgentPool>,
        }

        let resp = cmd.run::<InvokeResponse>(&self.cancellation).await?;
        let pools = resp.value;

        debug!("Found {} Azure DevOps agent pools", resp.count);

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

        Ok(pools)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsAgentPoolListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsAgentPoolListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsAgentPoolListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsAgentPoolListRequest<'static> => Vec<AzureDevOpsAgentPool>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_default_organization_url;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let org_url = get_default_organization_url(&cancellation).await?;
                let pools = fetch_azure_devops_agent_pools(&org_url, &cancellation).await?;
                assert!(
                    !pools.is_empty(),
                    "Expected at least one Azure DevOps agent pool"
                );
                assert!(
                    pools.iter().all(|pool| !pool.name.as_ref().is_empty()),
                    "Expected sampled Azure DevOps agent pools to have names"
                );

                Ok(())
            })
            .await
    }
}
