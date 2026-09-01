use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzurePublicIpResource;
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
pub struct PublicIpListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_public_ips<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> PublicIpListRequest<'a> {
    PublicIpListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PublicIpListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for PublicIpListRequest<'a> {
    type Output = Vec<AzurePublicIpResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "public_ips",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.auth_context.tenant_id, "Fetching public IP addresses");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/publicipaddresses"
        | project
            id,
            tenantId,
            name,
            sku,
            location,
            tags,
            properties
        "#}
        .to_owned();

        let public_ips =
            ResourceGraphHelper::new(query, Some(self.cache_key()), self.auth_context.as_ref())
                .collect_all::<AzurePublicIpResource>()
                .await?;
        info!(count = public_ips.len(), "Fetched public IP addresses");
        Ok(public_ips)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PublicIpListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_public_ips(
            &AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?,
        )
        .await?;
        for public_ip in &result {
            assert!(!public_ip.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PublicIpListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PublicIpListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PublicIpListRequest<'static> => Vec<AzurePublicIpResource>);
