use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzurePrivateEndpointResource;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct PrivateEndpointListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_private_endpoints<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> PrivateEndpointListRequest<'a> {
    PrivateEndpointListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PrivateEndpointListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for PrivateEndpointListRequest<'a> {
    type Output = Vec<AzurePrivateEndpointResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "private_endpoints",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.auth_context.tenant_id, "Fetching private endpoints");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/privateendpoints"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            properties
        "#}
        .to_owned();

        let private_endpoints =
            ResourceGraphHelper::new(query, Some(self.cache_key()), self.auth_context.as_ref())
                .collect_all::<AzurePrivateEndpointResource>()
                .await?;
        info!(count = private_endpoints.len(), "Fetched private endpoints");
        Ok(private_endpoints)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PrivateEndpointListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_private_endpoints(
            &AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?,
        )
        .await?;
        for private_endpoint in &result {
            assert!(!private_endpoint.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PrivateEndpointListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PrivateEndpointListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PrivateEndpointListRequest<'static> => Vec<AzurePrivateEndpointResource>);
