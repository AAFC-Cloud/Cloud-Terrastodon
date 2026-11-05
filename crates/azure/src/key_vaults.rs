use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureResourceNameAvailability;
use cloud_terrastodon_azure_types::AzureResourceNameAvailabilityBody;
use cloud_terrastodon_azure_types::KeyVault;
use cloud_terrastodon_azure_types::KeyVaultName;
use cloud_terrastodon_azure_types::ResourceType;
use cloud_terrastodon_azure_types::SubscriptionId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct KeyVaultListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_key_vaults<'a>(
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> KeyVaultListRequest<'a> {
    KeyVaultListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for KeyVaultListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for KeyVaultListRequest<'a> {
    type Output = Vec<KeyVault>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "key_vaults",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut query = ResourceGraphHelper::new(
            r#"
resources
| where type =~ "microsoft.keyvault/vaults"
| project id,name,location,properties,tags
        "#,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
            &self.cancellation,
        );
        query.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(KeyVaultListRequest<'a>, 'a);

/// Check vault-name availability with the selected subscription and tenant.
/// Always checks the service; availability can change outside this application.
///
/// [Key Vault 2024-11-01](https://learn.microsoft.com/rest/api/keyvault/keyvault/vaults/check-name-availability?view=rest-keyvault-keyvault-2024-11-01).
pub async fn is_key_vault_name_available(
    name: &KeyVaultName,
    subscription_id: SubscriptionId,
    auth_context: &AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> eyre::Result<bool> {
    let url = Url::parse(&format!(
        "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.KeyVault/checkNameAvailability?api-version=2024-11-01"
    ))?;
    let body = AzureResourceNameAvailabilityBody {
        name: name.clone(),
        resource_type: ResourceType::MICROSOFT_DOT_KEYVAULT_SLASH_VAULTS,
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
                    .join("key-vault-name-availability")
                    .join(name.as_str()),
                valid_for: Duration::ZERO,
            })
            .receive()
            .await?;
    Ok(response.name_available)
}

#[cfg(test)]
mod test {
    use crate::fetch_all_key_vaults;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let key_vaults = fetch_all_key_vaults(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &cancellation,
                )
                .await?;
                assert!(!key_vaults.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(KeyVaultListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultListRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(KeyVaultListRequest<'static> => Vec<KeyVault>, effects = [Read]);
