use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct EntraGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_groups<'a>(auth_context: &'a AzureTenantAuthContext) -> EntraGroupListRequest<'a> {
    EntraGroupListRequest {
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
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.auth_context.tenant_id, "Fetching Entra groups");
        let rtn: Vec<EntraGroup> = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/v1.0/groups",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_all()
        .await?;
        debug!(tenant_id = %self.auth_context.tenant_id, count = rtn.len(), "Found Entra groups");
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
        let result = fetch_all_groups(&auth_context).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraGroupListRequest<'static> => Vec<EntraGroup>);
