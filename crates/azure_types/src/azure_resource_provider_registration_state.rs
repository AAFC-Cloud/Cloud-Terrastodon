use arbitrary::Arbitrary;

/// Registration state reported by Azure Resource Manager for a provider.
///
/// The [2021-04-01 provider schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list?view=rest-resources-2021-04-01#provider)
/// describes this property as a string. Known values are documented by the
/// [registration guidance](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/resource-providers-and-types)
/// and used by the [Azure CLI provider implementation](https://github.com/Azure/azure-cli/blob/dev/src/azure-cli/azure/cli/command_modules/resource/custom.py).
/// Subscription registration readiness must be checked for the selected
/// subscription; a tenant catalogue does not establish it. Debug builds expose unmodeled
/// states; release builds retain their original spelling.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureResourceProviderRegistrationState {
    Registered,
    Registering,
    NotRegistered,
    Unregistered,
    Unregistering,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureResourceProviderRegistrationState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Registered" => Self::Registered,
            "Registering" => Self::Registering,
            "NotRegistered" => Self::NotRegistered,
            "Unregistered" => Self::Unregistered,
            "Unregistering" => Self::Unregistering,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureResourceProviderRegistrationState> for String {
    fn from(value: &AzureResourceProviderRegistrationState) -> Self {
        match value {
            AzureResourceProviderRegistrationState::Registered => "Registered",
            AzureResourceProviderRegistrationState::Registering => "Registering",
            AzureResourceProviderRegistrationState::NotRegistered => "NotRegistered",
            AzureResourceProviderRegistrationState::Unregistered => "Unregistered",
            AzureResourceProviderRegistrationState::Unregistering => "Unregistering",
            AzureResourceProviderRegistrationState::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureResourceProviderRegistrationState);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProviderRegistrationState);
