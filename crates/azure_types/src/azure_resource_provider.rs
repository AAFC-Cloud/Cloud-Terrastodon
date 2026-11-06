use crate::AzureResourceProviderId;
use crate::AzureResourceProviderNamespace;
use crate::AzureResourceProviderRegistrationState;
use crate::azure_resource_provider_authorization_consent_state::AzureResourceProviderAuthorizationConsentState;
use crate::azure_resource_provider_resource_type::AzureResourceProviderResourceType;
use arbitrary::Arbitrary;
use std::sync::Arc;

/// Provider identity and advertised resource types.
///
/// [Resource Management 2021-04-01 schema](https://learn.microsoft.com/en-us/rest/api/resources/providers/list-at-tenant-scope?view=rest-resources-2021-04-01#provider).
/// The API schema does not require these fields; missing values remain explicit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceProvider {
    pub id: Option<AzureResourceProviderId>,
    pub namespace: Option<AzureResourceProviderNamespace>,
    pub registration_state: Option<AzureResourceProviderRegistrationState>,
    pub registration_policy: Option<String>,
    pub resource_types: Option<Vec<AzureResourceProviderResourceType>>,
    pub provider_authorization_consent_state:
        Option<AzureResourceProviderAuthorizationConsentState>,
}

cloud_terrastodon_registry::register_thing!(AzureResourceProvider);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceProvider);
cloud_terrastodon_registry::register_thing!(Vec<AzureResourceProvider>);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureResourceProvider>);
cloud_terrastodon_registry::register_arbitrary!(Vec<Arc<AzureResourceProvider>>);
