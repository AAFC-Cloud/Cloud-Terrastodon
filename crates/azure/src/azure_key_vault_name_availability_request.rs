use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureResourceNameAvailability;
use cloud_terrastodon_azure_types::AzureResourceNameAvailabilityBody;
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
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Check name availability in an explicitly selected subscription.
/// Always queries the service: an available name is not a reservation.
/// Returns the typed availability result, including any reason and message.
/// [Check Name Availability](https://learn.microsoft.com/rest/api/keyvault/keyvault/vaults/check-name-availability?view=rest-keyvault-keyvault-2024-11-01).
#[must_use = "Execute this request with an executor"]
#[derive(Debug, Facet)]
pub struct AzureKeyVaultNameAvailabilityRequest<'a> {
    pub authentication: AzureBearerAuthenticationSelection,
    pub subscription_id: SubscriptionId,
    pub name: Cow<'a, KeyVaultName>,
}
impl<'a> Arbitrary<'a> for AzureKeyVaultNameAvailabilityRequest<'static> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: Arbitrary::arbitrary(u)?,
            subscription_id: Arbitrary::arbitrary(u)?,
            name: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

impl HasCacheKey for AzureKeyVaultNameAvailabilityRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        Ok(CacheKey {
            scope: CacheScope::try_from_relative_path(
                PathBuf::from("rest/arm/tenants")
                    .join(self.authentication.tenant_id.to_string())
                    .join("subscriptions")
                    .join(self.subscription_id.to_string())
                    .join("key-vault-name-availability")
                    .join(self.name.as_str()),
            )?,
            valid_for: Duration::ZERO,
        })
    }
}
impl InvalidatesOwnCacheKey for AzureKeyVaultNameAvailabilityRequest<'_> {}
impl ExecutableRequest for AzureKeyVaultNameAvailabilityRequest<'_> {
    type Output = AzureResourceNameAvailability;
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
    {
        Box::pin(async move {
            let url = Url::parse(&format!(
                "https://management.azure.com/subscriptions/{}/providers/Microsoft.KeyVault/checkNameAvailability?api-version=2024-11-01",
                self.subscription_id
            ))?;
            let body = AzureResourceNameAvailabilityBody {
                name: &*self.name,
                resource_type: ResourceType::MICROSOFT_DOT_KEYVAULT_SLASH_VAULTS,
            };
            RestRequest::from_method_and_url(Method::POST, url)?
                .authentication(self.authentication.into())
                .cancellation_safe()
                .body(facet_json::to_string(&body)?)
                .cache(self.cache_key()?)
                .receive(&executor)
                .await
        })
    }
}
cloud_terrastodon_registry::register_thing!(AzureKeyVaultNameAvailabilityRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultNameAvailabilityRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureKeyVaultNameAvailabilityRequest<'static> => AzureResourceNameAvailability, effects = [Read]);
