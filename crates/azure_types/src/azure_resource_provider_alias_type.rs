use arbitrary::Arbitrary;
use facet::Facet;

/// The policy-alias value's sensitivity classification.
///
/// [Resource Management 2021-04-01 vocabulary](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliastype).
/// Debug builds expose unmodeled values; release builds retain their wire spelling.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceProviderAliasType {
    NotSpecified,
    PlainText,
    Mask,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceProviderAliasType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "NotSpecified" => Self::NotSpecified,
            "PlainText" => Self::PlainText,
            "Mask" => Self::Mask,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceProviderAliasType> for String {
    fn from(value: &AzureResourceProviderAliasType) -> Self {
        match value {
            AzureResourceProviderAliasType::NotSpecified => "NotSpecified",
            AzureResourceProviderAliasType::PlainText => "PlainText",
            AzureResourceProviderAliasType::Mask => "Mask",
            AzureResourceProviderAliasType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasType);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasType);
