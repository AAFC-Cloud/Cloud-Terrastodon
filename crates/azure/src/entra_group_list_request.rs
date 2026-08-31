use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
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
pub struct EntraGroupListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_groups<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> EntraGroupListRequest<'a> {
    EntraGroupListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupListRequest<'_> {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, "Fetching Entra groups");
        let rtn: Vec<EntraGroup> = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/groups",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_all()
        .await?;
        debug!(tenant_id = %self.tenant_id, count = rtn.len(), "Found Entra groups");
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let result = fetch_all_groups(tenant_id, &AuthContext::default()).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraGroupListRequest<'static> => Vec<EntraGroup>);
