use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipalId;
use cloud_terrastodon_azure_types::ScopeImpl;
use compact_str::CompactString;
use facet_json::RawJson;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[facet(tag = "scheme", content = "parameters")]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAuthorization {
    ServicePrincipal(AzureDevOpsServiceEndpointAuthorizationServicePrincipal),
    UsernamePassword(AzureDevOpsServiceEndpointAuthorizationUsernamePassword),
    WorkloadIdentityFederation(AzureDevOpsServiceEndpointAuthorizationWorkloadIdentityFederation),
    /// For stuff like Azure Container Registry authorization
    ManagedServiceIdentity(AzureDevOpsServiceEndpointAuthorizationManagedServiceIdentity),
    #[facet(untagged)]
    Other(RawJson<'static>),
}
impl AzureDevOpsServiceEndpointAuthorization {
    pub fn service_principal_id(&self) -> Option<&EntraServicePrincipalId> {
        match self {
            AzureDevOpsServiceEndpointAuthorization::ServicePrincipal(data) => {
                Some(&data.service_principal_id)
            }
            AzureDevOpsServiceEndpointAuthorization::WorkloadIdentityFederation(data) => {
                data.service_principal_id.as_ref()
            }
            _ => None,
        }
    }
}

// ============

#[derive(Debug, Eq, PartialEq, Clone, Hash, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointAuthorizationServicePrincipal {
    pub authentication_type:
        AzureDevOpsServiceEndpointAuthorizationServicePrincipalAuthenticationType,
    #[facet(rename = "serviceprincipalid")]
    pub service_principal_id: EntraServicePrincipalId,
    #[facet(rename = "tenantId", alias = "tenantid")]
    pub tenant_id: AzureTenantId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAuthorizationServicePrincipalAuthenticationType {
    #[facet(rename = "spnKey")]
    SpnKey,
}

// ============

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
pub struct AzureDevOpsServiceEndpointAuthorizationUsernamePassword {
    pub username: CompactString,
    #[facet(flatten)]
    pub extra: HashMap<CompactString, CompactString>,
}

// ============

#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointAuthorizationWorkloadIdentityFederation {
    pub scope: Option<ScopeImpl>,
    #[facet(rename = "servicePrincipalId", alias = "serviceprincipalid")]
    pub service_principal_id: Option<EntraServicePrincipalId>,
    #[facet(rename = "tenantId", alias = "tenantid")]
    pub tenant_id: AzureTenantId,
    pub workload_identity_federation_issuer: String,
    pub workload_identity_federation_subject: String,
    pub workload_identity_federation_issuer_type: Option<CompactString>,
}

// ============

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointAuthorizationManagedServiceIdentity {
    #[facet(rename = "tenantId", alias = "tenantid")]
    pub tenant_id: AzureTenantId,
    /// For stuff like Azure Container Registry authorization
    /// "loginServer": "pipelineimage.azurecr.io",
    #[facet(flatten)]
    pub extra: HashMap<CompactString, CompactString>,
}
