use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::ConditionalAccessNamedLocation;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct ConditionalAccessNamedLocationListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ConditionalAccessNamedLocationListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_conditional_access_named_locations<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> ConditionalAccessNamedLocationListRequest<'a> {
    ConditionalAccessNamedLocationListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for ConditionalAccessNamedLocationListRequest<'_> {
    type Output = Vec<ConditionalAccessNamedLocation>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "conditional_access_named_locations",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let query = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/beta/identity/conditionalAccess/namedLocations",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );

        let found = query.fetch_all().await?;
        Ok(found)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ConditionalAccessNamedLocationListRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::fetch_all_conditional_access_named_locations;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let auth_context =
            AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let found = fetch_all_conditional_access_named_locations(&auth_context).await?;
        assert!(!found.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ConditionalAccessNamedLocationListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ConditionalAccessNamedLocationListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ConditionalAccessNamedLocationListRequest<'static> => Vec<ConditionalAccessNamedLocation>);
