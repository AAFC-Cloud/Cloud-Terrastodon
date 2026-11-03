use arbitrary::Arbitrary;

/// ARM classification of the identity responsible for resource creation or modification.
///
/// Shared by [Storage RP 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#createdbytype)
/// and [Policy Insights 2024-10-01](https://learn.microsoft.com/en-us/rest/api/policyinsights/remediations/create-or-update-at-resource?view=rest-policyinsights-2024-10-01#createdbytype).
/// Release builds preserve new service values; debug builds expose unmodeled values.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceCreatedByType {
    User,
    Application,
    ManagedIdentity,
    Key,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceCreatedByType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "User" => Self::User,
            "Application" => Self::Application,
            "ManagedIdentity" => Self::ManagedIdentity,
            "Key" => Self::Key,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceCreatedByType> for String {
    fn from(value: &AzureResourceCreatedByType) -> Self {
        match value {
            AzureResourceCreatedByType::User => "User",
            AzureResourceCreatedByType::Application => "Application",
            AzureResourceCreatedByType::ManagedIdentity => "ManagedIdentity",
            AzureResourceCreatedByType::Key => "Key",
            AzureResourceCreatedByType::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceCreatedByType);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceCreatedByType);
