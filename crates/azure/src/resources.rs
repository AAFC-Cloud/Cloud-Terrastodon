// $x = az rest --method GET --url 'https://management.azure.com/subscriptions/{subscription_id}/providers?api-version=2021-04-01&' | ConvertFrom-Json
// $x.value | % { $n = $_.namespace; $_.resourceTypes | % { "$n/$($_.resourceType)" } } | fzf

use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Resource;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct ResourceListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ResourceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_resources<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> ResourceListRequest<'a> {
    ResourceListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for ResourceListRequest<'a> {
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
        let helper = ResourceGraphHelper::new(
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
            self.auth_context.as_ref(),
        );
        let mut helper = helper;
        let resources = helper.collect_all().await?;
        debug!(count = resources.len(), "Retrieved resources");
        Ok(resources)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ResourceListRequest<'a>, 'a);

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
        let auth_context = AuthContext::default();
        let resources = fetch_all_resources(get_test_tenant_id().await?, &auth_context).await?;
        assert!(resources.len() > 10);
        Ok(())
    }

    #[tokio::test]
    async fn resource_groups() -> Result<()> {
        let auth_context = AuthContext::default();
        let resources = fetch_all_resources(get_test_tenant_id().await?, &auth_context)
            .await?
            .into_iter()
            .filter(|res| res.kind.is_resource_group())
            .collect_vec();
        assert!(!resources.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn count() -> Result<()> {
        let auth_context = AuthContext::default();
        let resources = fetch_all_resources(get_test_tenant_id().await?, &auth_context).await?;
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

cloud_terrastodon_registry::register_thing!(ResourceListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ResourceListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ResourceListRequest<'static> => Vec<Resource>);
