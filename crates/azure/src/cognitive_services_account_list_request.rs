use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResource;
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
pub struct CognitiveServicesAccountListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_cognitive_services_accounts<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> CognitiveServicesAccountListRequest<'a> {
    CognitiveServicesAccountListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for CognitiveServicesAccountListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for CognitiveServicesAccountListRequest<'a> {
    type Output = Vec<AzureCognitiveServicesAccountResource>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "cognitive_services_accounts",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.auth_context.tenant_id, "Fetching Cognitive Services accounts");
        let query = indoc! {r#"
        Resources
        | where type == "microsoft.cognitiveservices/accounts"
        | project
            id,
            tenantId,
            name,
            kind,
            location,
            tags,
            sku,
            properties
        "#}
        .to_owned();

        let accounts =
            ResourceGraphHelper::new(query, Some(self.cache_key()), self.auth_context.as_ref())
                .collect_all::<AzureCognitiveServicesAccountResource>()
                .await?;
        info!(
            count = accounts.len(),
            "Fetched Cognitive Services accounts"
        );
        Ok(accounts)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(CognitiveServicesAccountListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let auth_context =
            AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let result = fetch_all_cognitive_services_accounts(&auth_context).await?;
        for account in &result {
            assert!(!account.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(CognitiveServicesAccountListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(CognitiveServicesAccountListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(CognitiveServicesAccountListRequest<'static> => Vec<AzureCognitiveServicesAccountResource>);
