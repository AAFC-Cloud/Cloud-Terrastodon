use crate::azure_devops_agent_pool_entitlements_for_project::fetch_azure_devops_agent_pool_entitlements_for_project;
use crate::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;

pub struct AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub pool: AzureDevOpsAgentPoolArgument<'a>,
    pub invalidate_cache: bool,
}

pub fn fetch_azure_devops_agent_pool_entitlements_for_pool<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    pool: impl Into<AzureDevOpsAgentPoolArgument<'a>>,
) -> AzureDevOpsAgentPoolEntitlementListForPoolRequest<'a> {
    AzureDevOpsAgentPoolEntitlementListForPoolRequest {
        org_url,
        pool: pool.into(),
        invalidate_cache: false,
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
    type Output = eyre::Result<Vec<crate::prelude::AzureDevOpsAgentPoolEntitlement>>;

    type IntoFuture = impl std::future::Future<Output = Self::Output> + 'a;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            let projects = fetch_all_azure_devops_projects(&self.org_url)
                .with_invalidation(self.invalidate_cache)
                .await?;
            let mut all_entitlements = Vec::new();
            for project in projects {
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_project(self.org_url, &project)
                        .with_invalidation(self.invalidate_cache)
                        .await?;
                for entitlement in entitlements {
                    if self.pool.matches(&entitlement.pool.id) || self.pool.matches(&entitlement.pool.name) {
                        all_entitlements.push(entitlement);
                    }
                }
            }
            Ok(all_entitlements)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::fetch_azure_devops_agent_pools;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let agent_pools = fetch_azure_devops_agent_pools(&org_url).await?;
        println!("Found {} agent pools", agent_pools.len());
        assert!(!agent_pools.is_empty(), "Expected at least one agent pool");
        for pool in agent_pools {
            let entitlements =
                fetch_azure_devops_agent_pool_entitlements_for_pool(&org_url, &pool).await?;
            println!("Found {} entitlements for pool", entitlements.len());
            if !entitlements.is_empty() {
                return Ok(());
            }
        }
        eyre::bail!("Expected at least one agent pool entitlement across all pools");
    }
}
