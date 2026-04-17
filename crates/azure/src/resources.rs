// $x = az rest --method GET --url 'https://management.azure.com/subscriptions/{subscription_id}/providers?api-version=2021-04-01&' | ConvertFrom-Json
// $x.value | % { $n = $_.namespace; $_.resourceTypes | % { "$n/$($_.resourceType)" } } | fzf

use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Resource;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct ResourceListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_resources(tenant_id: AzureTenantId) -> ResourceListRequest {
    ResourceListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ResourceListRequest {
    type Output = Vec<Resource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resources",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(fetching = "resources");
        let resources = ResourceGraphHelper::new(
            self.tenant_id,
            r#"
resources 
| union resourcecontainers
| project
    id,
    ['kind'] = type,
    name,
    tags,
    properties
"#,
            Some(self.cache_key()),
        )
        .collect_all()
        .await?;
        debug!(count = resources.len(), "Retrieved resources");
        Ok(resources)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ResourceListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::ResourceType;
    use cloud_terrastodon_azure_types::Scope;
    use cloud_terrastodon_azure_types::ScopeImplKind;
    use itertools::Itertools;
    use std::collections::HashMap;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let resources = fetch_all_resources(get_test_tenant_id().await?).await?;
        assert!(resources.len() > 10);
        Ok(())
    }

    #[tokio::test]
    async fn resource_groups() -> Result<()> {
        let resources = fetch_all_resources(get_test_tenant_id().await?)
            .await?
            .into_iter()
            .filter(|res| res.kind.is_resource_group())
            .collect_vec();
        assert!(!resources.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn count() -> Result<()> {
        let resources = fetch_all_resources(get_test_tenant_id().await?).await?;
        let ids: HashMap<ScopeImplKind, i32> =
            resources
                .iter()
                .map(|res| res.id.kind())
                .fold(HashMap::default(), |mut acc, kind| {
                    *acc.entry(kind).or_insert(0) += 1;
                    acc
                });

        let known_count: i32 = ids
            .iter()
            .filter(|x| *x.0 != ScopeImplKind::Unknown)
            .map(|(_, v)| *v)
            .sum();
        let unknown_count = ids
            .get(&ScopeImplKind::Unknown)
            .cloned()
            .unwrap_or_default();

        let unknown_kinds: HashMap<ResourceType, i32> = resources
            .iter()
            .filter(|res| res.id.kind() == ScopeImplKind::Unknown)
            .map(|res| res.kind.clone())
            .fold(HashMap::new(), |mut acc, kind| {
                *acc.entry(kind).or_insert(0) += 1;
                acc
            });

        assert_eq!(known_count + unknown_count, resources.len() as i32);
        assert_eq!(unknown_kinds.values().sum::<i32>(), unknown_count);

        Ok(())
    }
}
