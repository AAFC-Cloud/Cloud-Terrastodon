use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountDeployment;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountDeploymentListResult;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResourceId;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::SerializableRestResponse;
use eyre::bail;
use eyre::Result;
use eyre::WrapErr;
use std::path::PathBuf;
use tracing::info;

const COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENTS_API_VERSION: &str = "2025-06-01";

#[must_use = "This is a future request, you must .await it"]
pub struct CognitiveServicesAccountDeploymentListRequest {
    pub tenant_id: AzureTenantId,
    pub account_id: AzureCognitiveServicesAccountResourceId,
}

pub fn fetch_cognitive_services_account_deployments(
    tenant_id: AzureTenantId,
    account_id: AzureCognitiveServicesAccountResourceId,
) -> CognitiveServicesAccountDeploymentListRequest {
    CognitiveServicesAccountDeploymentListRequest {
        tenant_id,
        account_id,
    }
}

#[async_trait]
impl CacheableCommand for CognitiveServicesAccountDeploymentListRequest {
    type Output = Vec<AzureCognitiveServicesAccountDeployment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "cognitive_services",
            "deployments",
            self.tenant_id.to_string().as_str(),
            self.account_id
                .resource_group_id
                .subscription_id
                .to_string()
                .as_str(),
            self.account_id
                .resource_group_id
                .resource_group_name
                .as_ref(),
            self.account_id
                .azure_cognitive_services_account_resource_name
                .as_ref(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(
            tenant_id = %self.tenant_id,
            account_id = %self.account_id.expanded_form(),
            "Fetching Cognitive Services account deployments"
        );

        let url = format!(
            "https://management.azure.com{}/deployments?api-version={}",
            self.account_id.expanded_form(),
            COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENTS_API_VERSION
        );
        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.args([
            "rest",
            "--method",
            "GET",
            "--url",
            &url,
            "--output-format",
            "json",
            "--tenant",
            self.tenant_id.to_string().as_str(),
        ]);
        cmd.cache(self.cache_key());

        let response = cmd.run::<SerializableRestResponse>().await?;
        if !response.ok {
            bail!(
                "Cognitive Services account deployment list request failed with status {} ({})",
                response.status,
                response.reason_phrase.as_deref().unwrap_or("Unknown error")
            );
        }

        let body = response
            .into_json_body()
            .wrap_err("Expected deployment list response body to contain JSON")?;
        let deployments: AzureCognitiveServicesAccountDeploymentListResult =
            serde_json::from_value(body).wrap_err(
                "Failed to deserialize Cognitive Services account deployment list response",
            )?;
        info!(
            count = deployments.value.len(),
            account_id = %self.account_id.expanded_form(),
            "Fetched Cognitive Services account deployments"
        );
        Ok(deployments.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(
    CognitiveServicesAccountDeploymentListRequest
);

#[cfg(test)]
mod tests {
    use super::COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENTS_API_VERSION;
    use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResourceId;
    use cloud_terrastodon_azure_types::Scope;

    #[test]
    fn builds_deployment_list_url() -> eyre::Result<()> {
        let id = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.CognitiveServices/accounts/my-openai"
            .parse::<AzureCognitiveServicesAccountResourceId>()?;
        assert_eq!(
            format!(
                "https://management.azure.com{}/deployments?api-version={}",
                id.expanded_form(),
                COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENTS_API_VERSION
            ),
            "https://management.azure.com/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.CognitiveServices/accounts/my-openai/deployments?api-version=2025-06-01"
        );
        assert_eq!(
            id.expanded_form(),
            "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/my-rg/providers/Microsoft.CognitiveServices/accounts/my-openai"
        );
        Ok(())
    }
}
