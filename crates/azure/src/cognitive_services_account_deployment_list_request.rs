use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountDeployment;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountDeploymentListResult;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResourceId;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

const COGNITIVE_SERVICES_ACCOUNT_DEPLOYMENTS_API_VERSION: &str = "2025-06-01";

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct CognitiveServicesAccountDeploymentListRequest {
    pub auth_source: AzureBearerAuthSource,

    pub tenant_id: AzureTenantId,
    pub account_id: AzureCognitiveServicesAccountResourceId,
}

#[track_caller]
pub fn fetch_cognitive_services_account_deployments<'exec>(
    tenant_id: AzureTenantId,
    account_id: AzureCognitiveServicesAccountResourceId,
    auth_source: AzureBearerAuthSource,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, CognitiveServicesAccountDeploymentListRequest> {
    executor.execute(CognitiveServicesAccountDeploymentListRequest {
        auth_source,

        tenant_id,
        account_id,
    })
}

impl InvalidatesOwnCacheKey for CognitiveServicesAccountDeploymentListRequest {}

impl HasCacheKey for CognitiveServicesAccountDeploymentListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
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
}

impl ExecutableRequest for CognitiveServicesAccountDeploymentListRequest {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureCognitiveServicesAccountDeployment>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
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
            let deployments = RestRequest::new(http::Method::GET, &url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<AzureCognitiveServicesAccountDeploymentListResult>(
                    &executor,
                    CacheOperationOwner::of::<Self>(),
                )
                .await?;
            info!(
                count = deployments.value.len(),
                account_id = %self.account_id.expanded_form(),
                "Fetched Cognitive Services account deployments"
            );
            Ok(deployments.value.clone())
        })
    }
}

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

cloud_terrastodon_registry::register_thing!(CognitiveServicesAccountDeploymentListRequest);
cloud_terrastodon_registry::register_arbitrary!(CognitiveServicesAccountDeploymentListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(CognitiveServicesAccountDeploymentListRequest => Vec<Arc<AzureCognitiveServicesAccountDeployment>>, effects = [Read]);
