use crate::azure_devops_agent_pool_entitlements_for_project::fetch_azure_devops_agent_pool_entitlements_for_project;
use crate::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops_types::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use std::borrow::Cow;
use std::pin::Pin;

pub struct AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub pool: AzureDevOpsAgentPoolArgument<'a>,
    pub invalidate_cache: bool,
    pub auth_context: Cow<'a, AzureDevOpsAuthContext>,
}

pub fn fetch_azure_devops_agent_pool_entitlements_for_pool<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    pool: impl Into<AzureDevOpsAgentPoolArgument<'a>>,
    auth_context: &'a AzureDevOpsAuthContext,
) -> AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    AzureDevOpsAgentPoolEntitlementListForPoolRequest {
        org_url,
        pool: pool.into(),
        invalidate_cache: false,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> CacheInvalidatableIntoFuture for AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    type WithInvalidation = Self;
    fn with_invalidation(mut self, invalidate_cache: bool) -> Self {
        self.invalidate_cache = invalidate_cache;
        self
    }
}

impl<'a> IntoFuture for AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    type Output = eyre::Result<Vec<crate::AzureDevOpsAgentPoolEntitlement>>;

    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let projects =
                fetch_all_azure_devops_projects(self.org_url, self.auth_context.as_ref())
                    .with_invalidation(self.invalidate_cache)
                    .await?;
            let mut all_entitlements = Vec::new();
            for project in projects {
                let entitlements = fetch_azure_devops_agent_pool_entitlements_for_project(
                    self.org_url,
                    &project,
                    self.auth_context.as_ref(),
                )
                .with_invalidation(self.invalidate_cache)
                .await?;
                for entitlement in entitlements {
                    if self.pool.matches_entitlement(&entitlement) {
                        all_entitlements.push(entitlement);
                    }
                }
            }
            Ok(all_entitlements)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fetch_azure_devops_agent_pools;
    use crate::get_default_organization_url;
    use cloud_terrastodon_credentials::AuthContext;
    use itertools::Itertools;

    #[tokio::test]
    #[ignore = "This takes a long time because it iterates all projects"]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let auth_context = AuthContext::default();
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(&auth_context)?;
        let agent_pools = fetch_azure_devops_agent_pools(&org_url).await?;
        let our_agent_pools = agent_pools
            .iter()
            .filter(|pool| !pool.is_hosted)
            .collect_vec();
        assert!(
            !our_agent_pools.is_empty(),
            "Expected at least one of our agent pools"
        );
        for pool in our_agent_pools {
            let entitlements = fetch_azure_devops_agent_pool_entitlements_for_pool(
                &org_url,
                pool,
                &azure_devops_auth_context,
            )
            .await?;
            if !entitlements.is_empty() {
                assert!(
                    entitlements.iter().all(|entitlement| {
                        !entitlement.name.is_empty()
                            && entitlement.pool.id == pool.id
                            && entitlement.pool.name == pool.name
                    }),
                    "Expected agent pool entitlements to match the requested pool"
                );
                return Ok(());
            }
        }
        eyre::bail!("Expected at least one agent pool entitlement across all pools");
    }
}
