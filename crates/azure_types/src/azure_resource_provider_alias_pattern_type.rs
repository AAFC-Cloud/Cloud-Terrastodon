use arbitrary::Arbitrary;

/// The policy-alias pattern operation.
///
/// [Resource Management 2021-04-01 vocabulary](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliaspatterntype).
/// Debug builds expose unmodeled values; release builds retain their wire spelling.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceProviderAliasPatternType {
    NotSpecified,
    Extract,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceProviderAliasPatternType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "NotSpecified" => Self::NotSpecified,
            "Extract" => Self::Extract,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceProviderAliasPatternType> for String {
    fn from(value: &AzureResourceProviderAliasPatternType) -> Self {
        match value {
            AzureResourceProviderAliasPatternType::NotSpecified => "NotSpecified",
            AzureResourceProviderAliasPatternType::Extract => "Extract",
            AzureResourceProviderAliasPatternType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasPatternType);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasPatternType);
