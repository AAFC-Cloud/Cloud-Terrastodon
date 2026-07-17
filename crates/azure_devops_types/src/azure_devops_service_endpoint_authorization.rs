use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::OptionalNonEmptyStringProxy;
use cloud_terrastodon_azure_types::ScopeImpl;
use compact_str::CompactString;
use eyre::Context;
use facet_json::RawJson;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet, Arbitrary)]
// https://github.com/facet-rs/facet/issues/2342
#[facet(proxy = RawJson<'static>)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAuthorization {
    ServicePrincipal(AzureDevOpsServiceEndpointAuthorizationServicePrincipal),
    UsernamePassword(AzureDevOpsServiceEndpointAuthorizationUsernamePassword),
    WorkloadIdentityFederation(AzureDevOpsServiceEndpointAuthorizationWorkloadIdentityFederation),
    /// For stuff like Azure Container Registry authorization
    ManagedServiceIdentity(AzureDevOpsServiceEndpointAuthorizationManagedServiceIdentity),
    Other(ArbitraryJson),
}
impl AzureDevOpsServiceEndpointAuthorization {
    pub fn service_principal_id(&self) -> Option<&EntraServicePrincipalObjectId> {
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

impl TryFrom<RawJson<'static>> for AzureDevOpsServiceEndpointAuthorization {
    type Error = eyre::Error;

    fn try_from(value: RawJson<'static>) -> Result<Self, Self::Error> {
        let object = facet_json::from_str::<HashMap<String, RawJson<'static>>>(value.as_str())
            .wrap_err("expected Azure DevOps authorization to be a JSON object")?;
        let scheme = object
            .get("scheme")
            .ok_or_else(|| eyre::eyre!("Azure DevOps authorization missing scheme"))?;
        let scheme = facet_json::from_str::<String>(scheme.as_str())
            .wrap_err("expected Azure DevOps authorization scheme to be a string")?;
        let parameters = object
            .get("parameters")
            .ok_or_else(|| eyre::eyre!("Azure DevOps authorization missing parameters"))?;

        match scheme.as_str() {
            "ServicePrincipal" => Ok(Self::ServicePrincipal(facet_json::from_str(
                parameters.as_str(),
            )?)),
            "UsernamePassword" => Ok(Self::UsernamePassword(facet_json::from_str(
                parameters.as_str(),
            )?)),
            "WorkloadIdentityFederation" => Ok(Self::WorkloadIdentityFederation(
                facet_json::from_str(parameters.as_str())?,
            )),
            "ManagedServiceIdentity" => Ok(Self::ManagedServiceIdentity(facet_json::from_str(
                parameters.as_str(),
            )?)),
            _ => Ok(Self::Other(value.into())),
        }
    }
}

impl TryFrom<&AzureDevOpsServiceEndpointAuthorization> for RawJson<'static> {
    type Error = eyre::Error;

    fn try_from(value: &AzureDevOpsServiceEndpointAuthorization) -> Result<Self, Self::Error> {
        let (scheme, parameters) = match value {
            AzureDevOpsServiceEndpointAuthorization::ServicePrincipal(parameters) => (
                "ServicePrincipal",
                RawJson::from_owned(facet_json::to_string(parameters)?),
            ),
            AzureDevOpsServiceEndpointAuthorization::UsernamePassword(parameters) => (
                "UsernamePassword",
                RawJson::from_owned(facet_json::to_string(parameters)?),
            ),
            AzureDevOpsServiceEndpointAuthorization::WorkloadIdentityFederation(parameters) => (
                "WorkloadIdentityFederation",
                RawJson::from_owned(facet_json::to_string(parameters)?),
            ),
            AzureDevOpsServiceEndpointAuthorization::ManagedServiceIdentity(parameters) => (
                "ManagedServiceIdentity",
                RawJson::from_owned(facet_json::to_string(parameters)?),
            ),
            AzureDevOpsServiceEndpointAuthorization::Other(raw) => return Ok(raw.clone().into()),
        };

        let mut object = HashMap::new();
        object.insert(
            "scheme".to_string(),
            RawJson::from_owned(facet_json::to_string(scheme)?),
        );
        object.insert("parameters".to_string(), parameters);
        Ok(RawJson::from_owned(facet_json::to_string(&object)?))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, arbitrary::Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointAuthorizationServicePrincipal {
    pub authentication_type:
        AzureDevOpsServiceEndpointAuthorizationServicePrincipalAuthenticationType,
    #[facet(rename = "serviceprincipalid")]
    pub service_principal_id: EntraServicePrincipalObjectId,
    #[facet(rename = "tenantId", alias = "tenantid")]
    pub tenant_id: AzureTenantId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, arbitrary::Arbitrary, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsServiceEndpointAuthorizationServicePrincipalAuthenticationType {
    #[facet(rename = "spnKey")]
    SpnKey,
}

#[derive(Debug, Clone, PartialEq, Eq, arbitrary::Arbitrary, facet::Facet)]
pub struct AzureDevOpsServiceEndpointAuthorizationUsernamePassword {
    pub username: CompactString,
    #[facet(flatten)]
    pub extra: HashMap<CompactString, CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, arbitrary::Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointAuthorizationWorkloadIdentityFederation {
    pub scope: Option<ScopeImpl>,
    #[facet(rename = "servicePrincipalId", alias = "serviceprincipalid")]
    #[facet(proxy = OptionalNonEmptyStringProxy)]
    pub service_principal_id: Option<EntraServicePrincipalObjectId>,
    #[facet(rename = "tenantId", alias = "tenantid")]
    pub tenant_id: AzureTenantId,
    pub workload_identity_federation_issuer: String,
    pub workload_identity_federation_subject: String,
    pub workload_identity_federation_issuer_type: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, arbitrary::Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointAuthorizationManagedServiceIdentity {
    #[facet(rename = "tenantId", alias = "tenantid")]
    pub tenant_id: AzureTenantId,
    /// For stuff like Azure Container Registry authorization
    /// "loginServer": "pipelineimage.azurecr.io",
    #[facet(flatten)]
    pub extra: HashMap<CompactString, CompactString>,
}

#[cfg(test)]
mod test {
    use super::AzureDevOpsServiceEndpointAuthorization;
    use facet_json::RawJson;

    #[test]
    fn unknown_scheme_deserializes_to_other() -> eyre::Result<()> {
        let json = r#"{"scheme":"Token","parameters":{"token":"redacted"}}"#;

        let authorization: AzureDevOpsServiceEndpointAuthorization = facet_json::from_str(json)?;

        assert_eq!(
            authorization,
            AzureDevOpsServiceEndpointAuthorization::Other(
                RawJson::from_owned(json.to_string()).into()
            )
        );
        Ok(())
    }

    #[test]
    fn unknown_scheme_with_null_parameters_deserializes_to_other() -> eyre::Result<()> {
        let json = r#"{"scheme":"Token","parameters":null}"#;

        let authorization: AzureDevOpsServiceEndpointAuthorization = facet_json::from_str(json)?;

        assert_eq!(
            authorization,
            AzureDevOpsServiceEndpointAuthorization::Other(
                RawJson::from_owned(json.to_string()).into()
            )
        );
        Ok(())
    }

    #[test]
    fn workload_identity_empty_service_principal_id_deserializes_to_none() -> eyre::Result<()> {
        let json = r#"{
            "scheme": "WorkloadIdentityFederation",
            "parameters": {
                "serviceprincipalid": "",
                "tenantid": "9da98bb1-1857-4cc3-8751-9a49e35d24cd",
                "workloadIdentityFederationIssuer": "https://vstoken.dev.azure.com/example",
                "workloadIdentityFederationSubject": "sc://org/project/service-connection"
            }
        }"#;

        let authorization: AzureDevOpsServiceEndpointAuthorization = facet_json::from_str(json)?;

        let AzureDevOpsServiceEndpointAuthorization::WorkloadIdentityFederation(parameters) =
            authorization
        else {
            eyre::bail!("expected WorkloadIdentityFederation authorization");
        };
        assert_eq!(parameters.service_principal_id, None);
        Ok(())
    }
}
