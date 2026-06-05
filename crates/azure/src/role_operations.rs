use cloud_terrastodon_azure_types::AzureProviderOperationsMetadata;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::debug;

const PROVIDER_OPERATIONS_API_VERSION: &str = "2022-04-01";

#[must_use = "This is a future request, you must .await it"]
pub struct RoleOperationListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_role_operation_metadata(tenant_id: AzureTenantId) -> RoleOperationListRequest {
    RoleOperationListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for RoleOperationListRequest {
    type Output = Vec<AzureProviderOperationsMetadata>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "providerOperations",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        #[derive(Deserialize)]
        #[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
        #[serde(rename_all = "camelCase")]
        struct Response {
            next_link: Option<String>,
            value: Vec<AzureProviderOperationsMetadata>,
        }

        let tenant_id = self.tenant_id.to_string();
        let mut next_url = Some(format!(
            "https://management.azure.com/providers/Microsoft.Authorization/providerOperations?$expand=resourceTypes&api-version={PROVIDER_OPERATIONS_API_VERSION}"
        ));
        let mut page_index = 0usize;
        let mut operations = Vec::new();

        while let Some(url) = next_url.take() {
            debug!(page_index, %url, %tenant_id, "Fetching Azure provider operations metadata");
            let mut response: Response = RestRequest::new(http::Method::GET, &url)?
                .tenant(self.tenant_id)
                .cache(CacheKey {
                    path: self.cache_key().path.join(page_index.to_string()),
                    valid_for: self.cache_key().valid_for,
                })
                .receive()
                .await?;
            operations.append(&mut response.value);
            next_url = response.next_link;
            page_index += 1;
        }

        debug!(count = operations.len(), %tenant_id, "Fetched Azure provider operations metadata");
        Ok(operations)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RoleOperationListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let results = fetch_all_role_operation_metadata(get_test_tenant_id().await?).await?;
        assert!(!results.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn includes_microsoft_authorization_provider() -> Result<()> {
        let results = fetch_all_role_operation_metadata(get_test_tenant_id().await?).await?;
        assert!(
            results
                .iter()
                .any(|provider| provider.name == "Microsoft.Authorization")
        );
        Ok(())
    }
}
