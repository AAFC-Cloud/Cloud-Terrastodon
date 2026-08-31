use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureApplicationGatewayResource;
use cloud_terrastodon_azure_types::AzureTenantId;
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
pub struct ApplicationGatewayListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_application_gateways<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> ApplicationGatewayListRequest<'a> {
    ApplicationGatewayListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for ApplicationGatewayListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for ApplicationGatewayListRequest<'a> {
    type Output = Vec<AzureApplicationGatewayResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "application_gateways",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(%self.tenant_id, "Fetching application gateways");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/applicationgateways"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            identity,
            properties
        "#}
        .to_owned();

        let application_gateways =
            ResourceGraphHelper::new(
                self.tenant_id,
                query,
                Some(self.cache_key()),
                self.auth_context.as_ref(),
            )
                .collect_all::<AzureApplicationGatewayResource>()
                .await?;
        debug!(
            count = application_gateways.len(),
            "Fetched application gateways"
        );
        Ok(application_gateways)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationGatewayListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_application_gateways(
            get_test_tenant_id().await?,
            &AuthContext::default(),
        )
        .await?;
        for application_gateway in &result {
            assert!(!application_gateway.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ApplicationGatewayListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationGatewayListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ApplicationGatewayListRequest<'static> => Vec<AzureApplicationGatewayResource>);
