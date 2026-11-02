use arbitrary::Arbitrary;

/// The value vocabulary represented by a policy-alias path.
///
/// [Resource Management 2021-04-01 vocabulary](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#aliaspathtokentype).
/// Debug builds expose unmodeled values; release builds retain their wire spelling.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceProviderAliasPathTokenType {
    NotSpecified,
    Any,
    String,
    Object,
    Array,
    Integer,
    Number,
    Boolean,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceProviderAliasPathTokenType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "NotSpecified" => Self::NotSpecified,
            "Any" => Self::Any,
            "String" => Self::String,
            "Object" => Self::Object,
            "Array" => Self::Array,
            "Integer" => Self::Integer,
            "Number" => Self::Number,
            "Boolean" => Self::Boolean,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceProviderAliasPathTokenType> for String {
    fn from(value: &AzureResourceProviderAliasPathTokenType) -> Self {
        match value {
            AzureResourceProviderAliasPathTokenType::NotSpecified => "NotSpecified",
            AzureResourceProviderAliasPathTokenType::Any => "Any",
            AzureResourceProviderAliasPathTokenType::String => "String",
            AzureResourceProviderAliasPathTokenType::Object => "Object",
            AzureResourceProviderAliasPathTokenType::Array => "Array",
            AzureResourceProviderAliasPathTokenType::Integer => "Integer",
            AzureResourceProviderAliasPathTokenType::Number => "Number",
            AzureResourceProviderAliasPathTokenType::Boolean => "Boolean",
            AzureResourceProviderAliasPathTokenType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAliasPathTokenType);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAliasPathTokenType);
