use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureNetworkInterfaceResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct NetworkInterfaceListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_network_interfaces<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> NetworkInterfaceListRequest<'a> {
    NetworkInterfaceListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for NetworkInterfaceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for NetworkInterfaceListRequest<'a> {
    type Output = Vec<AzureNetworkInterfaceResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "network_interfaces",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching network interfaces");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/networkinterfaces"
        | project
            id,
            tenantId,
            name,
            location,
            managedBy,
            tags,
            properties
        "#}
        .to_owned();

        let network_interfaces = ResourceGraphHelper::new(
            self.tenant_id,
            query,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .collect_all::<AzureNetworkInterfaceResource>()
        .await?;
        info!(
            count = network_interfaces.len(),
            "Fetched network interfaces"
        );
        Ok(network_interfaces)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(NetworkInterfaceListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result =
            fetch_all_network_interfaces(get_test_tenant_id().await?, &AuthContext::default())
                .await?;
        for network_interface in &result {
            assert!(!network_interface.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(NetworkInterfaceListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(NetworkInterfaceListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(NetworkInterfaceListRequest<'static> => Vec<AzureNetworkInterfaceResource>);
