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
use std::time::Duration;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct SecurityGroupListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for SecurityGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_security_groups<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> SecurityGroupListRequest<'a> {
    SecurityGroupListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for SecurityGroupListRequest<'a> {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter(["ms", "graph", "GET", "security_groups"]),
            valid_for: Duration::from_secs(2 * 60 * 60),
        }
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching security groups");
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/groups?$filter=securityEnabled eq true",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        let groups: Vec<EntraGroup> = query.fetch_all().await?;
        debug!("Found {} security groups", groups.len());
        Ok(groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(SecurityGroupListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_security_groups;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::EntraGroup;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let auth_context = AuthContext::default();
        let groups: Vec<EntraGroup> =
            fetch_all_security_groups(get_test_tenant_id().await?, &auth_context).await?;
        assert!(groups.len() > 1);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(SecurityGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(SecurityGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(SecurityGroupListRequest<'static> => Vec<EntraGroup>);
