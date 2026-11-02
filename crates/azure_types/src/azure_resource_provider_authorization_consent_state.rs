use arbitrary::Arbitrary;

/// The provider's authorization-consent state.
///
/// [Resource Management 2021-04-01 vocabulary](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#providerauthorizationconsentstate).
/// Debug builds expose unmodeled values; release builds retain their wire spelling.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceProviderAuthorizationConsentState {
    NotSpecified,
    Required,
    NotRequired,
    Consented,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceProviderAuthorizationConsentState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "NotSpecified" => Self::NotSpecified,
            "Required" => Self::Required,
            "NotRequired" => Self::NotRequired,
            "Consented" => Self::Consented,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceProviderAuthorizationConsentState> for String {
    fn from(value: &AzureResourceProviderAuthorizationConsentState) -> Self {
        match value {
            AzureResourceProviderAuthorizationConsentState::NotSpecified => "NotSpecified",
            AzureResourceProviderAuthorizationConsentState::Required => "Required",
            AzureResourceProviderAuthorizationConsentState::NotRequired => "NotRequired",
            AzureResourceProviderAuthorizationConsentState::Consented => "Consented",
            AzureResourceProviderAuthorizationConsentState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderAuthorizationConsentState);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderAuthorizationConsentState);
