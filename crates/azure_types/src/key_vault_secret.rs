use crate::KeyVaultSecretAttributes;
use crate::KeyVaultSecretId;
use arbitrary::Arbitrary;
use facet::Facet;
use std::collections::BTreeMap;

/// Base secret metadata; listing never retrieves a secret value or its versions.
/// [SecretItem, API 2025-07-01](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#secretitem).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct KeyVaultSecret {
    pub attributes: KeyVaultSecretAttributes,
    pub content_type: Option<String>,
    pub id: KeyVaultSecretId,
    pub managed: Option<bool>,
    pub tags: Option<BTreeMap<String, String>>,
}

cloud_terrastodon_registry::register_thing!(KeyVaultSecret);
cloud_terrastodon_registry::register_arbitrary!(KeyVaultSecret);
