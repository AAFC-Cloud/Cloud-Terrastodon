use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ServiceGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct ServiceGroupListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_service_groups<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> ServiceGroupListRequest<'a> {
    ServiceGroupListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for ServiceGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for ServiceGroupListRequest<'a> {
    type Output = Vec<ServiceGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "service_groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching service groups");
        let query = indoc! {r#"
        resourcecontainers
        | where type =~ "microsoft.management/servicegroups"
        | project
            id,
            name,
            properties
    "#}
        .to_owned();

        let service_groups =
            ResourceGraphHelper::new(
                self.tenant_id,
                query,
                Some(self.cache_key()),
                self.auth_context.as_ref(),
            )
                .collect_all::<ServiceGroup>()
                .await?;
        debug!("Found {} service groups", service_groups.len());
        Ok(service_groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ServiceGroupListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let result =
            fetch_all_service_groups(get_test_tenant_id().await?, &AuthContext::default()).await?;
        assert!(!result.is_empty());
        assert!(result.iter().all(|sg| !sg.name.is_empty()));
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ServiceGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ServiceGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ServiceGroupListRequest<'static> => Vec<ServiceGroup>);
