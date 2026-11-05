use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureResourceNameAvailability;
use cloud_terrastodon_azure_types::AzureResourceNameAvailabilityBody;
use cloud_terrastodon_azure_types::ResourceType;
use cloud_terrastodon_azure_types::StorageAccountName;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use http::Method;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

/// Check storage-account name availability in an explicitly selected subscription.
/// Always checks the service; availability can change outside this application.
///
/// [Storage Resource Provider 2025-06-01](https://learn.microsoft.com/rest/api/storagerp/storage-accounts/check-name-availability?view=rest-storagerp-2025-06-01).
pub async fn is_storage_account_name_available(
    name: &StorageAccountName,
    subscription_id: SubscriptionId,
    auth_context: &AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> Result<bool> {
    let url = Url::parse(&format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Storage/checkNameAvailability?api-version=2025-06-01"
    ))?;
    let body = AzureResourceNameAvailabilityBody {
        name: name.clone(),
        resource_type: ResourceType::MICROSOFT_DOT_STORAGE_SLASH_STORAGEACCOUNTS,
    };
    let response: AzureResourceNameAvailability =
        RestRequest::from_method_and_url(Method::POST, url, cancellation)?
            .azure_auth_context(auth_context)
            .cancellation_safe()
            .body(facet_json::to_string(&body)?)
            .cache(CacheKey {
                path: PathBuf::from("rest/arm/tenants")
                    .join(auth_context.tenant_id.to_string())
                    .join("subscriptions")
                    .join(subscription_id.to_string())
                    .join("storage-account-name-availability")
                    .join(name.as_str()),
                valid_for: Duration::ZERO,
            })
            .receive()
            .await?;
    Ok(response.name_available)
}
