use crate::AuthContext;
use crate::AuthSource;
use crate::AzureBearerToken;
use crate::AzureRestResource;
use cloud_terrastodon_azure_types::AzureAccessToken;
use cloud_terrastodon_azure_types::AzureTenantId;
use eyre::Result;
use eyre::ensure;

/// Invocation authentication state bound to one Azure tenant.
///
/// Cloning this value preserves the invocation's shared token caches because
/// the cache-bearing fields in [`AuthContext`] are reference counted.
#[derive(Debug, Clone, facet::Facet)]
pub struct AzureTenantAuthContext {
    pub auth_context: AuthContext,
    pub tenant_id: AzureTenantId,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureTenantAuthContext {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: AuthContext::default(),
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl AzureTenantAuthContext {
    pub fn new(auth_context: &AuthContext, tenant_id: AzureTenantId) -> Result<Self> {
        let source = auth_context.source().ok_or_else(|| {
            eyre::eyre!("the placeholder authentication context cannot be bound to an Azure tenant")
        })?;
        ensure!(
            !matches!(source, AuthSource::Auto),
            "authentication source must be resolved before binding an Azure tenant"
        );
        ensure!(
            !matches!(source, AuthSource::PersonalAccessToken),
            "a personal access token cannot be bound to an Azure tenant"
        );
        if let Some(workload_identity) = auth_context.workload_identity() {
            ensure!(
                workload_identity.tenant_id == tenant_id,
                "requested tenant does not match the workload identity tenant"
            );
        }
        Ok(Self {
            auth_context: auth_context.clone(),
            tenant_id,
        })
    }

    pub async fn fetch_bearer_access_token(
        &self,
        resource: AzureRestResource,
    ) -> Result<AzureAccessToken<AzureBearerToken>> {
        crate::fetch_azure_bearer_access_token(&self.auth_context, Some(self.tenant_id), resource)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorkloadIdentityConfig;

    #[test]
    fn rejects_a_workload_identity_tenant_mismatch() -> Result<()> {
        let configured_tenant = "11111111-1111-1111-1111-111111111111".parse()?;
        let requested_tenant = "22222222-2222-2222-2222-222222222222".parse()?;
        let auth_context = AuthContext::explicit(AuthSource::WorkloadIdentity)
            .with_workload_identity(WorkloadIdentityConfig {
                client_id: "33333333-3333-3333-3333-333333333333".parse()?,
                tenant_id: configured_tenant,
                federated_token: "test-token".to_owned(),
            })?;

        let error = AzureTenantAuthContext::new(&auth_context, requested_tenant)
            .expect_err("mismatched workload identity tenant should be rejected");

        assert!(error.to_string().contains("does not match"));
        Ok(())
    }

    #[test]
    fn accepts_an_azure_cli_tenant() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let auth_context = AuthContext::explicit(AuthSource::AzureCli);

        let tenant_auth_context = AzureTenantAuthContext::new(&auth_context, tenant_id)?;

        assert_eq!(tenant_auth_context.tenant_id, tenant_id);
        assert_eq!(
            tenant_auth_context.auth_context.source(),
            Some(AuthSource::AzureCli)
        );
        Ok(())
    }
}
