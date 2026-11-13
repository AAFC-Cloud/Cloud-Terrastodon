use arbitrary::Arbitrary;
use facet::Facet;

/// Reported minimum transport-security protocol version.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#minimumtlsversion).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountMinimumTlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountMinimumTlsVersion {
    fn from(value: String) -> Self {
        match value.as_str() {
            "TLS1_0" => Self::Tls10,
            "TLS1_1" => Self::Tls11,
            "TLS1_2" => Self::Tls12,
            "TLS1_3" => Self::Tls13,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountMinimumTlsVersion> for String {
    fn from(value: &AzureStorageAccountMinimumTlsVersion) -> Self {
        match value {
            AzureStorageAccountMinimumTlsVersion::Tls10 => "TLS1_0",
            AzureStorageAccountMinimumTlsVersion::Tls11 => "TLS1_1",
            AzureStorageAccountMinimumTlsVersion::Tls12 => "TLS1_2",
            AzureStorageAccountMinimumTlsVersion::Tls13 => "TLS1_3",
            AzureStorageAccountMinimumTlsVersion::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountMinimumTlsVersion);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountMinimumTlsVersion);
