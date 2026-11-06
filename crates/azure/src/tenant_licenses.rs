use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::TenantLicense;
use cloud_terrastodon_azure_types::TenantLicenseCollection;
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

#[must_use = "This is a future request, you must .execute it"]
#[derive(Arbitrary, Facet)]
pub struct TenantLicenseListRequest {
    pub auth_source: AzureBearerAuthSource,
    pub tenant_id: AzureTenantId,
}

#[track_caller]
pub fn fetch_all_tenant_licenses<'exec>(
    tenant_id: AzureTenantId,
    auth_source: AzureBearerAuthSource,
    executor: &'exec CloudTerrastodonExecutor,
) -> RequestExecution<'exec, TenantLicenseListRequest> {
    executor.execute(TenantLicenseListRequest {
        auth_source,
        tenant_id,
    })
}

impl InvalidatesOwnCacheKey for TenantLicenseListRequest {}

impl HasCacheKey for TenantLicenseListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "subscribedSkus",
            &self.tenant_id.to_string(),
        ]))
    }
}

impl ExecutableRequest for TenantLicenseListRequest {
    type Error = eyre::Report;
    type Output = TenantLicenseCollection;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let url = "https://graph.microsoft.com/v1.0/subscribedSkus";
            let resp = RestRequest::new(http::Method::GET, url)?
                .authentication(AuthenticationSelection::AzureBearer {
                    auth_source: self.auth_source,
                    tenant_id: self.tenant_id,
                })
                .cache(self.cache_key()?)
                .receive_cached::<MicrosoftGraphResponse<Arc<TenantLicense>>>(
                    &executor,
                    CacheOperationOwner::of::<TenantLicenseListRequest>(),
                )
                .await?;
            Ok(TenantLicenseCollection(resp.value.clone()))
        })
    }
}

#[cfg(test)]
pub mod test_helpers {
    use crate::fetch_all_tenant_licenses;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure_types::AzureTenantId;
    use cloud_terrastodon_command::CommandOutput;
    use cloud_terrastodon_command::bstr::ByteSlice;
    use cloud_terrastodon_execution::CloudTerrastodonExecutor;

    /// If the given result is a failure, it MUST contain an AAD Premium P2 license error.
    /// If it is an error ,we also validate that the tenant licenses do not contain the AAD Premium P2 license.
    /// If it is a success, we validate that the tenant licenses DO contain the AAD Premium P2 license.
    pub async fn expect_aad_premium_p2_license<T>(
        result: eyre::Result<T>,
        auth_source: AzureBearerAuthSource,
        tenant_id: AzureTenantId,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<Option<T>> {
        let tenant_licenses = fetch_all_tenant_licenses(tenant_id, auth_source, executor).await?;
        let has_aad_premium_p2_license = tenant_licenses.has_aad_premium_p2();
        match result {
            Ok(x) => {
                eyre::ensure!(
                    has_aad_premium_p2_license,
                    "Expected AAD Premium P2 license, but it was not found in tenant licenses: {tenant_licenses:#?}"
                );
                Ok(Some(x))
            }
            Err(e) => {
                if has_aad_premium_p2_license {
                    return Err(e.wrap_err(
                            "A result failed while wrapped in a guard that expected AAD Premium P2 license to be missing, but it was found in tenant licenses, this means the inner command truly failed?"
                        ));
                }
                let expected = "The tenant needs to have Microsoft Entra ID P2 or Microsoft Entra ID Governance license.";
                let has_expected_message = e
                    .downcast_ref::<CommandOutput>()
                    .map(|command_output| command_output.stderr.contains_str(expected))
                    .unwrap_or(false)
                    || format!("{e:?}").contains(expected);
                if !has_expected_message {
                    return Err(e.wrap_err("A result failed while wrapped in a guard that expected AAD Premium P2 license to be missing, but the error did not contain expected AAD Premium P2 license error text"));
                }
                eprintln!(
                    "Result failed with expected error due to missing AAD_PREMIUM_P2 licenses."
                );
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_tenant_licenses;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = crate::get_test_tenant_id(&cancellation).await?;
                let tenant_licenses = fetch_all_tenant_licenses(
                    tenant_id,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    &executor,
                )
                .await?;
                assert!(!tenant_licenses.0.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(TenantLicenseListRequest);
cloud_terrastodon_registry::register_arbitrary!(TenantLicenseListRequest);
cloud_terrastodon_registry::register_arbitrary!(TenantLicenseCollection);
cloud_terrastodon_registry_execution::register_executable_request!(TenantLicenseListRequest => TenantLicenseCollection, effects = [Read]);
