use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct SecurityGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for SecurityGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_security_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> SecurityGroupListRequest<'a> {
    SecurityGroupListRequest {
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
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let groups: Vec<EntraGroup> = fetch_all_security_groups(&auth_context).await?;
        assert!(groups.len() > 1);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(SecurityGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(SecurityGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(SecurityGroupListRequest<'static> => Vec<EntraGroup>);
