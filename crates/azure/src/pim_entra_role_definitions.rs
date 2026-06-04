use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::PimEntraRoleDefinition;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct PimEntraRoleDefinitionListRequest {
    tenant_id: AzureTenantId,
}

pub fn fetch_all_entra_pim_role_definitions(
    tenant_id: AzureTenantId,
) -> PimEntraRoleDefinitionListRequest {
    PimEntraRoleDefinitionListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for PimEntraRoleDefinitionListRequest {
    type Output = Vec<PimEntraRoleDefinition>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "pim_roleDefinitions",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let url = format!(
            "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleDefinitions?$select=id,displayName,type,isbuiltIn&$orderby=displayName",
            tenant_id = self.tenant_id,
        );
        #[derive(Deserialize)]
        struct Response {
            value: Vec<PimEntraRoleDefinition>,
        }

        let request = RestRequest::new(http::Method::GET, &url)?.cache(self.cache_key());
        let mut result: Result<Response, _> = request.clone().send_json().await;
        if result.is_err() {
            // single retry - sometimes this returns a gateway error
            result = request.send_json().await;
        }
        Ok(result?.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PimEntraRoleDefinitionListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use crate::test_helpers::expect_aad_premium_p2_license;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(result) = expect_aad_premium_p2_license(
            fetch_all_entra_pim_role_definitions(get_test_tenant_id().await?).await,
        )
        .await?
        else {
            return Ok(());
        };
        assert!(!result.is_empty());
        Ok(())
    }
}
