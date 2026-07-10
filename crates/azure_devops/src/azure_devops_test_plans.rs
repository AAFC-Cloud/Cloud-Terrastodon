use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsTestPlan;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use facet_json::RawJson;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsTestPlanListRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_azure_devops_test_plans<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> AzureDevOpsTestPlanListRequest<'a> {
    AzureDevOpsTestPlanListRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsTestPlanListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for AzureDevOpsTestPlanListRequest<'a> {
    type Output = Vec<AzureDevOpsTestPlan>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "test",
            "plan",
            "list",
            self.project.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching Azure DevOps user entitlements");
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        let org = self.org_url.to_string();
        cmd.args(["--organization", org.as_str()]);
        cmd.args(["--area", "test"]);
        cmd.args(["--resource", "plans"]);
        cmd.args(["--api-version", "5.0"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.args([
            "--route-parameters",
            format!("project={}", self.project).as_str(),
        ]);
        cmd.cache(self.cache_key());

        #[derive(facet::Facet)]
        struct InvokeResponse {
            continuation_token: Option<RawJson<'static>>,
            count: u32,
            value: Vec<AzureDevOpsTestPlan>,
        }

        let resp = cmd.run::<InvokeResponse>().await?;
        let entitlements = resp.value;

        debug!("Found {} Azure DevOps user entitlements", resp.count);

        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }

        Ok(entitlements)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsTestPlanListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(AzureDevOpsTestPlanListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTestPlanListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsTestPlanListRequest<'static> => Vec<AzureDevOpsTestPlan>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use eyre::bail;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        for project in projects {
            let test_plans = fetch_azure_devops_test_plans(&org_url, &project).await?;
            if test_plans.is_empty() {
                continue;
            }
            assert!(
                test_plans
                    .iter()
                    .all(|test_plan| !test_plan.name.is_empty() && test_plan.id > 0),
                "Expected sampled Azure DevOps test plans to include names and ids"
            );
            return Ok(());
        }

        bail!("Failed to find any test plans in any project");
    }
}
use arbitrary::Arbitrary;
