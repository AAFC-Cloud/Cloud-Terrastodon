use arbitrary::Arbitrary;
use facet::Facet;

/// Approval status of a storage private endpoint connection.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#privateendpointserviceconnectionstatus).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountPrivateEndpointServiceConnectionStatus {
    Pending,
    Approved,
    Rejected,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountPrivateEndpointServiceConnectionStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Pending" => Self::Pending,
            "Approved" => Self::Approved,
            "Rejected" => Self::Rejected,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountPrivateEndpointServiceConnectionStatus> for String {
    fn from(value: &AzureStorageAccountPrivateEndpointServiceConnectionStatus) -> Self {
        match value {
            AzureStorageAccountPrivateEndpointServiceConnectionStatus::Pending => "Pending",
            AzureStorageAccountPrivateEndpointServiceConnectionStatus::Approved => "Approved",
            AzureStorageAccountPrivateEndpointServiceConnectionStatus::Rejected => "Rejected",
            AzureStorageAccountPrivateEndpointServiceConnectionStatus::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(
    AzureStorageAccountPrivateEndpointServiceConnectionStatus
);
cloud_terrastodon_registry::register_arbitrary!(
    AzureStorageAccountPrivateEndpointServiceConnectionStatus
);
