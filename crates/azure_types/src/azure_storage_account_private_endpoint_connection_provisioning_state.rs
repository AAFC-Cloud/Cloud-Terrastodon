use arbitrary::Arbitrary;
use facet::Facet;

/// Provisioning status of a storage private endpoint connection.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#privateendpointconnectionprovisioningstate).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountPrivateEndpointConnectionProvisioningState {
    Succeeded,
    Creating,
    Deleting,
    Failed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountPrivateEndpointConnectionProvisioningState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Succeeded" => Self::Succeeded,
            "Creating" => Self::Creating,
            "Deleting" => Self::Deleting,
            "Failed" => Self::Failed,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountPrivateEndpointConnectionProvisioningState> for String {
    fn from(value: &AzureStorageAccountPrivateEndpointConnectionProvisioningState) -> Self {
        match value {
            AzureStorageAccountPrivateEndpointConnectionProvisioningState::Succeeded => "Succeeded",
            AzureStorageAccountPrivateEndpointConnectionProvisioningState::Creating => "Creating",
            AzureStorageAccountPrivateEndpointConnectionProvisioningState::Deleting => "Deleting",
            AzureStorageAccountPrivateEndpointConnectionProvisioningState::Failed => "Failed",
            AzureStorageAccountPrivateEndpointConnectionProvisioningState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountPrivateEndpointConnectionProvisioningState
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountPrivateEndpointConnectionProvisioningState
);
