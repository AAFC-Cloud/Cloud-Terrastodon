use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraServicePrincipalApplicationPermissions;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use http::Method;
use std::path::PathBuf;
use tracing::info;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct ApplicationRoleListRequest {
    pub tenant_id: AzureTenantId,
    pub application_id: EntraApplicationClientId,
}

pub fn fetch_application_roles(
    tenant_id: AzureTenantId,
    application_id: EntraApplicationClientId,
) -> ApplicationRoleListRequest {
    ApplicationRoleListRequest {
        tenant_id,
        application_id,
    }
}

#[async_trait]
impl CacheableCommand for ApplicationRoleListRequest {
    type Output = EntraServicePrincipalApplicationPermissions;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principal_application_permissions",
            self.tenant_id.to_string().as_str(),
            self.application_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!(
            tenant_id = %self.tenant_id,
            application_id = %self.application_id,
            "Fetching service principal app roles and resource-specific application permissions"
        );
        let url = format!(
            "https://graph.microsoft.com/v1.0/servicePrincipals(appId='{application_id}')?$select=appRoles,resourceSpecificApplicationPermissions",
            application_id = self.application_id
        );
        RestRequest::new(Method::GET, url)?
            .tenant(self.tenant_id)
            .cache(self.cache_key())
            .receive()
            .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRoleListRequest);
cloud_terrastodon_registry::register_thing!(ApplicationRoleListRequest);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRoleListRequest);
cloud_terrastodon_registry::register_into_future!(
    ApplicationRoleListRequest => EntraServicePrincipalApplicationPermissions
);
