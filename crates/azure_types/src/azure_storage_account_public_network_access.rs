use arbitrary::Arbitrary;
use facet::Facet;

/// Public endpoint admission policy.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#publicnetworkaccess).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountPublicNetworkAccess {
    Enabled,
    Disabled,
    SecuredByPerimeter,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountPublicNetworkAccess {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Enabled" => Self::Enabled,
            "Disabled" => Self::Disabled,
            "SecuredByPerimeter" => Self::SecuredByPerimeter,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountPublicNetworkAccess> for String {
    fn from(value: &AzureStorageAccountPublicNetworkAccess) -> Self {
        match value {
            AzureStorageAccountPublicNetworkAccess::Enabled => "Enabled",
            AzureStorageAccountPublicNetworkAccess::Disabled => "Disabled",
            AzureStorageAccountPublicNetworkAccess::SecuredByPerimeter => "SecuredByPerimeter",
            AzureStorageAccountPublicNetworkAccess::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountPublicNetworkAccess);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountPublicNetworkAccess);
