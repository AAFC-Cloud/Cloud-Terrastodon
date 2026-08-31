use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RouteTable;
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
pub struct RouteTableListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_route_tables<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> RouteTableListRequest<'a> {
    RouteTableListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for RouteTableListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for RouteTableListRequest<'a> {
    type Output = Vec<RouteTable>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "route_tables",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!("Fetching route tables");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/routetables"
        | project
            id,
            name,
            location,
            resource_group_name=resourceGroup,
            subscription_id=subscriptionId,
            tags,
            properties
    "#}
        .to_owned();

        let route_tables = ResourceGraphHelper::new(
            self.tenant_id,
            query,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .collect_all::<RouteTable>()
        .await?;
        info!("Found {} route tables", route_tables.len());
        Ok(route_tables)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RouteTableListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result =
            fetch_all_route_tables(get_test_tenant_id().await?, &AuthContext::default()).await?;
        assert!(!result.is_empty());
        for route_table in result {
            assert!(!route_table.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(RouteTableListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RouteTableListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(RouteTableListRequest<'static> => Vec<RouteTable>);
