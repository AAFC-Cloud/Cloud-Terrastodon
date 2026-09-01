use crate::AuthContext;
use crate::AuthSource;
use crate::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureTenantId;
use eyre::Result;
use eyre::bail;

/// Authentication state normalized for an Azure DevOps request.
///
/// Entra bearer authentication is tenant-bound. Azure CLI's active-tenant
/// compatibility behavior and tenant-independent PAT authentication are
/// represented explicitly instead of overloading `None` as a tenant value.
#[derive(Debug, Clone, facet::Facet)]
#[repr(u8)]
pub enum AzureDevOpsAuthContext {
    /// Synthetic or otherwise incomplete request state. Production context
    /// resolution never returns this variant.
    None,
    Bearer(AzureTenantAuthContext),
    AzureCli(AuthContext),
    PersonalAccessToken(AuthContext),
}

impl AzureDevOpsAuthContext {
    /// Normalize an already tenant-bound Azure authentication context for
    /// Azure DevOps without splitting the tenant from its credentials.
    pub fn for_azure_tenant(auth_context: &AzureTenantAuthContext) -> Self {
        if matches!(
            auth_context.auth_context.source(),
            Some(AuthSource::PersonalAccessToken)
        ) {
            return Self::PersonalAccessToken(auth_context.auth_context.clone());
        }
        if auth_context.auth_context.source().is_none() {
            return Self::None;
        }
        Self::Bearer(auth_context.clone())
    }

    /// Normalize the selected source without an explicit tenant override.
    pub fn new(auth_context: &AuthContext) -> Result<Self> {
        if let Some(tenant_id) = auth_context.tenant_id() {
            return Self::for_tenant(auth_context, tenant_id);
        }
        match auth_context.source() {
            Some(AuthSource::AzureCli) => Ok(Self::AzureCli(auth_context.clone())),
            Some(AuthSource::PersonalAccessToken) => {
                Ok(Self::PersonalAccessToken(auth_context.clone()))
            }
            Some(AuthSource::WorkloadIdentity) => bail!(
                "workload identity authentication requires a configured tenant before making Azure DevOps requests"
            ),
            Some(AuthSource::Browser) => bail!(
                "browser authentication requires an explicit tenant before making Azure DevOps requests"
            ),
            Some(AuthSource::Auto) => {
                bail!("authentication source must be resolved before making Azure DevOps requests")
            }
            None => Ok(Self::None),
        }
    }

    /// Normalize the selected source with an explicit tenant.
    pub fn for_tenant(auth_context: &AuthContext, tenant_id: AzureTenantId) -> Result<Self> {
        if matches!(auth_context.source(), Some(AuthSource::PersonalAccessToken)) {
            return Ok(Self::PersonalAccessToken(auth_context.clone()));
        }
        if auth_context.source().is_none() {
            return Ok(Self::None);
        }
        Ok(Self::Bearer(AzureTenantAuthContext::new(
            auth_context,
            tenant_id,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personal_access_token_does_not_require_a_tenant() -> Result<()> {
        let auth_context = AuthContext::explicit(AuthSource::PersonalAccessToken);

        let context = AzureDevOpsAuthContext::new(&auth_context)?;

        assert!(matches!(
            context,
            AzureDevOpsAuthContext::PersonalAccessToken(_)
        ));
        Ok(())
    }

    #[test]
    fn explicit_tenant_produces_a_bearer_context() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let auth_context = AuthContext::explicit(AuthSource::Browser);

        let context = AzureDevOpsAuthContext::for_tenant(&auth_context, tenant_id)?;

        let AzureDevOpsAuthContext::Bearer(context) = context else {
            panic!("expected tenant-bound bearer authentication");
        };
        assert_eq!(context.tenant_id, tenant_id);
        Ok(())
    }

    #[test]
    fn tenant_context_remains_tenant_bound() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let auth_context = AuthContext::explicit(AuthSource::Browser);
        let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;

        let AzureDevOpsAuthContext::Bearer(context) =
            AzureDevOpsAuthContext::for_azure_tenant(&auth_context)
        else {
            panic!("expected tenant-bound bearer authentication");
        };
        assert_eq!(context.tenant_id, tenant_id);
        Ok(())
    }

    #[test]
    fn azure_cli_active_tenant_is_an_explicit_variant() -> Result<()> {
        let auth_context = AuthContext::explicit(AuthSource::AzureCli);

        let context = AzureDevOpsAuthContext::new(&auth_context)?;

        assert!(matches!(context, AzureDevOpsAuthContext::AzureCli(_)));
        Ok(())
    }
}
