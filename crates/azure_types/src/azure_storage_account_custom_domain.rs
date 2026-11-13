use arbitrary::Arbitrary;
use facet::Facet;

/// A user-assigned CNAME and the optional indirect-validation setting.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#customdomain).
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureStorageAccountCustomDomain {
    pub name: String,
    pub use_sub_domain_name: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountCustomDomain);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountCustomDomain);
