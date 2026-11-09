use arbitrary::Arbitrary;
use facet::Facet;

/// Policy-modification capabilities of a policy-alias path.
///
/// [Resource Management 2021-04-01 vocabulary](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliaspathattributes).
/// Debug builds expose unmodeled values; release builds retain their wire spelling.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceProviderAliasPathAttributes {
    None,
    Modifiable,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceProviderAliasPathAttributes {
    fn from(value: String) -> Self {
        match value.as_str() {
            "None" => Self::None,
            "Modifiable" => Self::Modifiable,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceProviderAliasPathAttributes> for String {
    fn from(value: &AzureResourceProviderAliasPathAttributes) -> Self {
        match value {
            AzureResourceProviderAliasPathAttributes::None => "None",
            AzureResourceProviderAliasPathAttributes::Modifiable => "Modifiable",
            AzureResourceProviderAliasPathAttributes::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasPathAttributes);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasPathAttributes);
