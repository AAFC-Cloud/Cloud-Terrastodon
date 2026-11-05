use arbitrary::Arbitrary;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::KeyVaultId;
use cloud_terrastodon_azure_types::KeyVaultSecret;
use cloud_terrastodon_azure_types::KeyVaultSecretListResponse;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_pathing::sanitize_windows_path_component;
use cloud_terrastodon_rest::RestRequest;
use eyre::ensure;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use url::Url;

/// List base secret metadata using the selected tenant's Key Vault credential.
/// [Get Secrets, API 2025-07-01](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01).
#[derive(facet::Facet)]
pub struct KeyVaultSecretsListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    pub key_vault_id: Cow<'a, KeyVaultId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_key_vault_secrets<'a>(
    key_vault_id: &'a KeyVaultId,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> KeyVaultSecretsListRequest<'a> {
    KeyVaultSecretsListRequest {
        cancellation: cancellation.clone(),
        key_vault_id: Cow::Borrowed(key_vault_id),
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> Arbitrary<'a> for KeyVaultSecretsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: Arbitrary::arbitrary(u)?,
            key_vault_id: Cow::Owned(Arbitrary::arbitrary(u)?),
            auth_context: Cow::Owned(Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl CacheableCommand for KeyVaultSecretsListRequest<'_> {
    type Output = Vec<KeyVaultSecret>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            PathBuf::from("rest/key-vault/tenants")
                .join(self.auth_context.tenant_id.to_string())
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

    async fn run(self) -> eyre::Result<Self::Output> {
        let origin = Url::parse(&format!(
            "https://{}.vault.azure.net",
            self.key_vault_id.key_vault_name
        ))?;
        let mut url = origin.join("/secrets")?;
        url.query_pairs_mut()
            .append_pair("api-version", "2025-07-01")
            .append_pair("maxresults", "25");
        let cache = self.cache_key();
        let mut visited = HashSet::new();
        let mut secrets = Vec::new();
        loop {
            self.cancellation.bail_if_cancelled()?;
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
            let request =
                RestRequest::from_method_and_url(Method::GET, url.clone(), &self.cancellation)?
                    .azure_auth_context(&self.auth_context)
                    .cache(CacheKey {
                        path: cache
                            .path
                            .join("pages")
                            .join(blake3::hash(url.as_str().as_bytes()).to_hex().as_str()),
                        valid_for: cache.valid_for,
                    });
            let page = request.receive::<KeyVaultSecretListResponse>().await?;
            secrets.extend(page.value);
            match page.next_link {
                Some(next) => url = next,
                None => return Ok(secrets),
            }
        }
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(KeyVaultSecretsListRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(KeyVaultSecretsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultSecretsListRequest<'static>);
cloud_terrastodon_registry_execution::register_into_future!(KeyVaultSecretsListRequest<'static> => Vec<KeyVaultSecret>, effects = [Read]);
