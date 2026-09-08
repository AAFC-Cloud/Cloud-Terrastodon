use crate::ArbitraryJson;
use crate::AzureContainerInstanceResourceId;
use crate::AzureContainerInstanceResourceName;
use crate::AzureTenantId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceResource {
    #[facet(default)]
    pub api_version: Option<String>,
    pub id: AzureContainerInstanceResourceId,
    pub name: AzureContainerInstanceResourceName,
    #[facet(default, rename = "type")]
    pub resource_type: Option<String>,
    pub location: String,
    #[facet(default)]
    pub tenant_id: Option<AzureTenantId>,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    #[facet(default)]
    pub identity: Option<AzureContainerInstanceIdentity>,
    pub properties: AzureContainerInstanceResourceProperties,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceIdentity {
    #[facet(rename = "type")]
    pub identity_type: String,
    #[facet(default)]
    pub user_assigned_identities: Option<ArbitraryJson>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceResourceProperties {
    #[facet(default)]
    pub sku: Option<String>,
    #[facet(default)]
    pub is_created_from_standby_pool: Option<bool>,
    #[facet(default)]
    pub provisioning_state: Option<String>,
    #[facet(default)]
    pub provisioning_timeout_in_seconds: Option<u32>,
    #[facet(default)]
    pub is_custom_provisioning_timeout: Option<bool>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceContainer>)]
    pub containers: Vec<AzureContainerInstanceContainer>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceContainer>)]
    pub init_containers: Vec<AzureContainerInstanceContainer>,
    #[facet(
        default,
        proxy = crate::VecDefaultNullProxy<AzureContainerInstanceImageRegistryCredential>
    )]
    pub image_registry_credentials: Vec<AzureContainerInstanceImageRegistryCredential>,
    #[facet(default)]
    pub restart_policy: Option<String>,
    #[facet(default)]
    pub ip_address: Option<AzureContainerInstanceIpAddress>,
    #[facet(default)]
    pub os_type: Option<String>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceVolume>)]
    pub volumes: Vec<AzureContainerInstanceVolume>,
    #[facet(default)]
    pub instance_view: Option<AzureContainerInstanceInstanceView>,
    #[facet(default)]
    pub diagnostics: Option<AzureContainerInstanceDiagnostics>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceSubnetReference>)]
    pub subnet_ids: Vec<AzureContainerInstanceSubnetReference>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceContainer {
    pub name: String,
    pub properties: AzureContainerInstanceContainerProperties,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceContainerProperties {
    pub image: String,
    #[facet(default, proxy = crate::VecDefaultNullProxy<String>)]
    pub command: Vec<String>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstancePort>)]
    pub ports: Vec<AzureContainerInstancePort>,
    #[facet(
        default,
        proxy = crate::VecDefaultNullProxy<AzureContainerInstanceEnvironmentVariable>
    )]
    pub environment_variables: Vec<AzureContainerInstanceEnvironmentVariable>,
    #[facet(default)]
    pub config_map: Option<AzureContainerInstanceConfigMap>,
    #[facet(default)]
    pub instance_view: Option<AzureContainerInstanceContainerInstanceView>,
    #[facet(default)]
    pub resources: Option<AzureContainerInstanceResources>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceVolumeMount>)]
    pub volume_mounts: Vec<AzureContainerInstanceVolumeMount>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstancePort {
    pub protocol: String,
    pub port: u16,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceEnvironmentVariable {
    pub name: String,
    #[facet(default)]
    pub value: Option<String>,
    #[facet(default)]
    pub secure_value: Option<String>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceConfigMap {
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub key_value_pairs: HashMap<String, String>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceContainerInstanceView {
    #[facet(default)]
    pub restart_count: Option<u32>,
    #[facet(default)]
    pub current_state: Option<AzureContainerInstanceCurrentState>,
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceEvent>)]
    pub events: Vec<AzureContainerInstanceEvent>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceCurrentState {
    #[facet(default)]
    pub state: Option<String>,
    #[facet(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[facet(default)]
    pub detail_status: Option<String>,
    #[facet(default)]
    pub exit_code: Option<isize>,
    #[facet(default)]
    pub finish_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceEvent {
    #[facet(default)]
    pub count: Option<u32>,
    #[facet(default)]
    pub first_timestamp: Option<DateTime<Utc>>,
    #[facet(default)]
    pub last_timestamp: Option<DateTime<Utc>>,
    #[facet(default)]
    pub name: Option<String>,
    #[facet(default)]
    pub message: Option<String>,
    #[facet(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceResources {
    #[facet(default)]
    pub requests: Option<AzureContainerInstanceResourceRequests>,
    #[facet(default)]
    pub limits: Option<AzureContainerInstanceResourceRequests>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceResourceRequests {
    #[facet(default, rename = "memoryInGB")]
    pub memory_in_gb: Option<f64>,
    #[facet(default)]
    pub cpu: Option<f64>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceVolumeMount {
    pub name: String,
    pub mount_path: String,
    #[facet(default)]
    pub read_only: Option<bool>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceImageRegistryCredential {
    pub server: String,
    #[facet(default)]
    pub username: Option<String>,
    #[facet(default)]
    pub password: Option<String>,
    #[facet(default)]
    pub is_delegated_identity: Option<bool>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceIpAddress {
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstancePort>)]
    pub ports: Vec<AzureContainerInstancePort>,
    #[facet(default, proxy = crate::OptionalIpAddrProxy)]
    pub ip: Option<IpAddr>,
    #[facet(default)]
    pub r#type: Option<String>,
    #[facet(default)]
    pub fqdn: Option<String>,
    #[facet(default)]
    pub dns_name_label: Option<String>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceVolume {
    pub name: String,
    #[facet(default)]
    pub azure_file: Option<AzureContainerInstanceAzureFileVolume>,
    #[facet(default)]
    pub empty_dir: Option<ArbitraryJson>,
    #[facet(default)]
    pub git_repo: Option<ArbitraryJson>,
    #[facet(default)]
    pub secret: Option<ArbitraryJson>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceAzureFileVolume {
    pub share_name: String,
    #[facet(default)]
    pub read_only: Option<bool>,
    pub storage_account_name: String,
    #[facet(default)]
    pub storage_account_key: Option<String>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceInstanceView {
    #[facet(default, proxy = crate::VecDefaultNullProxy<AzureContainerInstanceEvent>)]
    pub events: Vec<AzureContainerInstanceEvent>,
    #[facet(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceDiagnostics {
    #[facet(default)]
    pub log_analytics: Option<AzureContainerInstanceLogAnalytics>,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceLogAnalytics {
    pub workspace_id: String,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
#[facet(rename_all = "camelCase")]
pub struct AzureContainerInstanceSubnetReference {
    pub id: String,
    #[facet(default)]
    pub name: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureContainerInstanceResource);
cloud_terrastodon_registry::register_arbitrary!(AzureContainerInstanceResource);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureContainerInstanceResource>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scopes::Scope;
    use facet::Facet;

    #[test]
    fn registered_container_generator_can_produce_nonempty_json() -> eyre::Result<()> {
        let constructor = cloud_terrastodon_registry::functions_from_to(
            cloud_terrastodon_registry::ArbitraryBytes::SHAPE,
            Vec::<AzureContainerInstanceResource>::SHAPE,
        )
        .into_iter()
        .find(|function| {
            function
                .output_shape
                .is_shape(Vec::<AzureContainerInstanceResource>::SHAPE)
        })
        .expect("an exact container vector generator must be registered");
        let mut bytes = vec![0; 4096];
        bytes[0] = 1;
        let mut input = cloud_terrastodon_registry::ArbitraryBytes::new(bytes);
        let resources = constructor
            .invoke_mut_boxed(&mut input)?
            .downcast::<Vec<AzureContainerInstanceResource>>()
            .expect("constructor output has the registered vector type");
        assert_eq!(
            resources.len(),
            1,
            "the generator must not be an empty placeholder"
        );
        let json = facet_json::to_string(resources.as_ref())?;
        let decoded = facet_json::from_str::<Vec<AzureContainerInstanceResource>>(&json)?;
        assert_eq!(decoded.len(), resources.len());
        for (original, decoded) in resources.iter().zip(&decoded) {
            assert_eq!(decoded.id, original.id);
            assert_eq!(
                original
                    .id
                    .expanded_form()
                    .parse::<AzureContainerInstanceResourceId>()?,
                original.id
            );
            assert_eq!(decoded.name, original.name);
        }
        Ok(())
    }
}
