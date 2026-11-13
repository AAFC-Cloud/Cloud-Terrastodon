use arbitrary::Arbitrary;
use facet::Facet;

/// DNS endpoint allocation scheme for a storage account.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#dnsendpointtype).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountDnsEndpointType {
    Standard,
    AzureDnsZone,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountDnsEndpointType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Standard" => Self::Standard,
            "AzureDnsZone" => Self::AzureDnsZone,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountDnsEndpointType> for String {
    fn from(value: &AzureStorageAccountDnsEndpointType) -> Self {
        match value {
            AzureStorageAccountDnsEndpointType::Standard => "Standard",
            AzureStorageAccountDnsEndpointType::AzureDnsZone => "AzureDnsZone",
            AzureStorageAccountDnsEndpointType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountDnsEndpointType);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountDnsEndpointType);
