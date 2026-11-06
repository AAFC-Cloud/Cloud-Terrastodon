use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
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
use cloud_terrastodon_execution::RequestExecution;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct KeyVaultListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_key_vaults<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, KeyVaultListRequest<'a>> {
    executor.execute(KeyVaultListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for KeyVaultListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for KeyVaultListRequest<'a> {}

impl<'a> HasCacheKey for KeyVaultListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "key_vaults",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for KeyVaultListRequest<'a> {
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
                self.auth_context.as_ref(),
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
    auth_context: &AzureTenantAuthContext,
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
            .azure_auth_context(auth_context)?
            .cancellation_safe()
            .body(facet_json::to_string(&body)?)
            .cache(CacheKey {
                scope: CacheScope::try_from_relative_path(
                    PathBuf::from("rest/arm/tenants")
                        .join(auth_context.tenant_id.to_string())
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
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let key_vaults = fetch_all_key_vaults(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
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
cloud_terrastodon_registry_execution::register_executable_request!(KeyVaultListRequest<'static> => Vec<Arc<KeyVault>>, effects = [Read]);
