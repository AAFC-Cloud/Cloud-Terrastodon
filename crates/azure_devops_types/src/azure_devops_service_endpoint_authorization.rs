use cloud_terrastodon_azure_types::prelude::ScopeImpl;
use cloud_terrastodon_azure_types::prelude::ServicePrincipalId;
use cloud_terrastodon_azure_types::prelude::TenantId;
use cloud_terrastodon_azure_types::serde_helpers::deserialize_none_if_empty;
use compact_str::CompactString;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(tag = "scheme", content = "parameters")]
pub enum AzureDevOpsServiceEndpointAuthorization {
    ServicePrincipal(AzureDevOpsServiceEndpointAuthorizationServicePrincipal),
    UsernamePassword(AzureDevOpsServiceEndpointAuthorizationUsernamePassword),
    Token(()),
    WorkloadIdentityFederation(AzureDevOpsServiceEndpointAuthorizationWorkloadIdentityFederation),
    ManagedServiceIdentity(AzureDevOpsServiceEndpointAuthorizationManagedServiceIdentity),
}

// ============

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct AzureDevOpsServiceEndpointAuthorizationServicePrincipal {
    #[serde(rename = "authenticationType")]
    pub authentication_type:
        AzureDevOpsServiceEndpointAuthorizationServicePrincipalAuthenticationType,
    #[serde(rename = "serviceprincipalid")]
    pub service_principal_id: ServicePrincipalId,
    #[serde(rename = "tenantId")]
    #[serde(alias = "tenantid")]
    pub tenant_id: TenantId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AzureDevOpsServiceEndpointAuthorizationServicePrincipalAuthenticationType {
    #[serde(rename = "spnKey")]
    SpnKey,
}

// ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AzureDevOpsServiceEndpointAuthorizationUsernamePassword {
    pub username: CompactString,
    #[serde(flatten)]
    pub extra: HashMap<CompactString, CompactString>,
}

// ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct AzureDevOpsServiceEndpointAuthorizationWorkloadIdentityFederation {
    pub scope: Option<ScopeImpl>,
    #[serde(rename = "servicePrincipalId")]
    #[serde(alias = "serviceprincipalid")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_none_if_empty")]
    pub service_principal_id: Option<ServicePrincipalId>,
    #[serde(rename = "tenantId")]
    #[serde(alias = "tenantid")]
    pub tenant_id: TenantId,
    #[serde(rename = "workloadIdentityFederationIssuer")]
    pub workload_identity_federation_issuer: String,
    #[serde(rename = "workloadIdentityFederationSubject")]
    pub workload_identity_federation_subject: String,
    #[serde(rename = "workloadIdentityFederationIssuerType")]
    pub workload_identity_federation_issuer_type: Option<CompactString>,
}

// ============

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AzureDevOpsServiceEndpointAuthorizationManagedServiceIdentity {
    #[serde(rename = "tenantId")]
    #[serde(alias = "tenantid")]
    pub tenant_id: TenantId,
    #[serde(flatten)]
    pub extra: HashMap<CompactString, CompactString>,
}
