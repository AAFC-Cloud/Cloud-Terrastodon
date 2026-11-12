use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::KeyVaultId;
use cloud_terrastodon_azure_types::KeyVaultSecret;
use cloud_terrastodon_azure_types::KeyVaultSecretListResponse;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use facet::Facet;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// List base secret metadata using the selected tenant's Key Vault credential.
/// [Get Secrets, API 2025-07-01](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01).
#[derive(Facet)]
pub struct AzureKeyVaultSecretsListRequest<'a> {
    pub key_vault_id: Cow<'a, KeyVaultId>,
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> Arbitrary<'a> for AzureKeyVaultSecretsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            key_vault_id: Cow::Owned(Arbitrary::arbitrary(u)?),
            authentication: Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureKeyVaultSecretsListRequest<'_> {}

impl HasCacheKey for AzureKeyVaultSecretsListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(
            PathBuf::from("rest/key-vault/tenants")
                .join(self.authentication.tenant_id.to_string())
                .join("subscriptions")
                .join(
                    self.key_vault_id
                        .resource_group_id
                        .subscription_id
                        .to_string(),
                )
                .join("resource-groups")
                .join(sanitize_windows_path_component(
                    self.key_vault_id
                        .resource_group_id
                        .resource_group_name
                        .as_str(),
                ))
                .join("vaults")
                .join(self.key_vault_id.key_vault_name.as_str())
                .join("secrets"),
        )
    }
}

impl ExecutableRequest for AzureKeyVaultSecretsListRequest<'_> {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<KeyVaultSecret>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let origin = Url::parse(&format!(
                "https://{}.vault.azure.net",
                self.key_vault_id.key_vault_name
            ))?;
            let mut url = origin.join("/secrets")?;
            url.query_pairs_mut()
                .append_pair("api-version", "2025-07-01")
                .append_pair("maxresults", "25");
            let cache = self.cache_key()?;
            let mut visited = HashSet::new();
            let mut secrets = Vec::new();
            loop {
                executor.cancellation().bail_if_cancelled()?;
                ensure!(
                    url.origin() == origin.origin()
                        && url.path() == "/secrets"
                        && url.username().is_empty()
                        && url.password().is_none()
                        && url.fragment().is_none(),
                    "Key Vault pagination must remain on the selected vault's metadata-list operation"
                );
                ensure!(
                    visited.insert(url.clone()),
                    "Key Vault returned a repeated continuation URL"
                );
                let request = RestRequest::from_method_and_url(Method::GET, url.clone())?
                    .authentication(self.authentication.into())
                    .cache(CacheKey {
                        scope: cache
                            .scope
                            .join("pages")?
                            .join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str())?,
                        valid_for: cache.valid_for,
                    });
                let page = request
                    .receive_cached::<KeyVaultSecretListResponse>(
                        &executor,
                        CacheOperationOwner::of::<AzureKeyVaultSecretsListRequest<'static>>(),
                    )
                    .await?;
                secrets.extend(page.value.iter().cloned());
                match page.next_link.clone() {
                    Some(next) => url = next,
                    None => return Ok(secrets),
                }
            }
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureKeyVaultSecretsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureKeyVaultSecretsListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(AzureKeyVaultSecretsListRequest<'static> => Vec<Arc<KeyVaultSecret>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureKeyVaultSecretsListRequest<'static>,
    name = "key-vault-secrets-list",
    cached
);
