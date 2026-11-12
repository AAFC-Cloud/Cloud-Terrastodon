use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureResourceNameAvailability;
use cloud_terrastodon_azure_types::AzureResourceNameAvailabilityBody;
use cloud_terrastodon_azure_types::KeyVault;
use cloud_terrastodon_azure_types::KeyVaultName;
use cloud_terrastodon_azure_types::ResourceType;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_rest::RestRequest;
use facet::Facet;
use http::Method;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureKeyVaultListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureKeyVaultListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureKeyVaultListRequest {}

impl HasCacheKey for AzureKeyVaultListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "key_vaults",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureKeyVaultListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<KeyVault>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut query = ResourceGraphHelper::new(
                r#"
resources
| where type =~ "microsoft.keyvault/vaults"
| project id,name,location,properties,tags
        "#,
                Some(self.cache_key()?),
                self.authentication,
            );
            query.collect_all(&executor).await
        })
    }
}

/// Check vault-name availability with the selected subscription and tenant.
/// Always checks the service; availability can change outside this application.
///
/// [Key Vault 2024-11-01](https://learn.microsoft.com/rest/api/keyvault/keyvault/vaults/check-name-availability?view=rest-keyvault-keyvault-2024-11-01).
pub async fn is_key_vault_name_available(
    name: &KeyVaultName,
    subscription_id: SubscriptionId,
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<bool> {
    let url = Url::parse(&format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.KeyVault/checkNameAvailability?api-version=2024-11-01"
    ))?;
    let body = AzureResourceNameAvailabilityBody {
        name: name.clone(),
        resource_type: ResourceType::MICROSOFT_DOT_KEYVAULT_SLASH_VAULTS,
    };
    let response: AzureResourceNameAvailability =
        RestRequest::from_method_and_url(Method::POST, url)?
            .authentication((*authentication).into())
            .cancellation_safe()
            .body(facet_json::to_string(&body)?)
            .cache(CacheKey {
                scope: CacheScope::try_from_relative_path(
                    PathBuf::from("rest/arm/tenants")
                        .join(authentication.tenant_id.to_string())
                        .join("subscriptions")
                        .join(subscription_id.to_string())
                        .join("key-vault-name-availability")
                        .join(name.as_str()),
                )?,
                valid_for: Duration::ZERO,
            })
            .receive(executor)
            .await?;
    Ok(response.name_available)
}

cloud_terrastodon_registry::register_thing!(AzureKeyVaultListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureKeyVaultListRequest => Vec<Arc<KeyVault>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureKeyVaultListRequest,
    name = "key-vault-list",
    cached
);
