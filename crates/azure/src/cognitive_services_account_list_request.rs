use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResource;
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
pub struct CognitiveServicesAccountListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

pub fn fetch_all_cognitive_services_accounts<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> CognitiveServicesAccountListRequest<'a> {
    CognitiveServicesAccountListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for CognitiveServicesAccountListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
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
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(%self.tenant_id, "Fetching Cognitive Services accounts");
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

        let accounts = ResourceGraphHelper::new(
            self.tenant_id,
            query,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
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

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        let result = fetch_all_cognitive_services_accounts(
            get_test_tenant_id().await?,
            &AuthContext::default(),
        )
        .await?;
        for account in &result {
            assert!(!account.name.is_empty());
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(CognitiveServicesAccountListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(CognitiveServicesAccountListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(CognitiveServicesAccountListRequest<'static> => Vec<AzureCognitiveServicesAccountResource>);
