use crate::AzureDevOpsProjectArgument;
use crate::azure_devops_rest::authenticate_azure_devops_request;
use crate::azure_devops_rest::azure_devops_api_url;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_rest::RestRequest;
use reqwest::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    pub org_url: Cow<'a, AzureDevOpsOrganizationUrl>,
    pub project: AzureDevOpsProjectArgument<'a>,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

pub fn fetch_azure_devops_agent_pool_entitlements_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
) -> AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a> {
    AzureDevOpsAgentPoolEntitlementListForProjectRequest {
        org_url: Cow::Borrowed(org_url),
        project: project.into(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            org_url: Cow::Owned(AzureDevOpsOrganizationUrl::arbitrary(u)?),
            project: AzureDevOpsProjectArgument::arbitrary(u)?.into_owned(),
            auth_context: Cow::Owned(AzureDevOpsAuthContext::None),
        })
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand
    for AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a>
{
    type Output = Vec<crate::AzureDevOpsAgentPoolEntitlement>;

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

        #[derive(facet::Facet)]
        struct Response {
            count: u32,
            value: Vec<crate::AzureDevOpsAgentPoolEntitlement>,
        }

        let project = project.to_string();
        let url = azure_devops_api_url(
            &self.org_url,
            "dev.azure.com",
            &format!("{project}/_apis/distributedtask/queues"),
            &[("api-version", "7.1")],
        )?;
        let request = RestRequest::new(Method::GET, url)?.cache(self.cache_key());
        let response = authenticate_azure_devops_request(request, self.auth_context.as_ref())?
            .receive::<Response>()
            .await?;

        debug!(
            "Found {} Azure DevOps agent queue entitlements for project {}",
            response.count, project
        );

        Ok(response.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureDevOpsAgentPoolEntitlementListForProjectRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(
    AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static>
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static>
);
cloud_terrastodon_registry::register_into_future!(AzureDevOpsAgentPoolEntitlementListForProjectRequest<'static> => Vec<crate::AzureDevOpsAgentPoolEntitlement>, effects = [Read]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_all_azure_devops_projects;
    use crate::get_default_organization_url;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let auth_context = AuthContext::default();
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(&auth_context)?;
        let projects =
            fetch_all_azure_devops_projects(&org_url, &azure_devops_auth_context).await?;

        // Iterate projects, and stop when we find the first project with entitlements.
        let mut found = false;
        for project in projects {
            let entitlements = fetch_azure_devops_agent_pool_entitlements_for_project(
                &org_url,
                &project.name,
                &azure_devops_auth_context,
            )
            .await?;
            if !entitlements.is_empty() {
                assert!(
                    entitlements
                        .iter()
                        .all(|entitlement| !entitlement.name.is_empty()
                            && !entitlement.pool.name.as_ref().is_empty()),
                    "Expected Azure DevOps queue/pool entitlements to include names"
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
