use arbitrary::Arbitrary;
use facet::Facet;

/// Network used to route storage service traffic.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#routingchoice).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountRoutingChoice {
    MicrosoftRouting,
    InternetRouting,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountRoutingChoice {
    fn from(value: String) -> Self {
        match value.as_str() {
            "MicrosoftRouting" => Self::MicrosoftRouting,
            "InternetRouting" => Self::InternetRouting,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountRoutingChoice> for String {
    fn from(value: &AzureStorageAccountRoutingChoice) -> Self {
        match value {
            AzureStorageAccountRoutingChoice::MicrosoftRouting => "MicrosoftRouting",
            AzureStorageAccountRoutingChoice::InternetRouting => "InternetRouting",
            AzureStorageAccountRoutingChoice::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountRoutingChoice);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountRoutingChoice);
