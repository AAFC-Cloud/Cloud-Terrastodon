use arbitrary::Arbitrary;
use facet::Facet;

/// Selective ARM tag update semantics. Delete removes the supplied matching
/// name/value pairs; Replace replaces the entire set, and Merge preserves other keys.
/// [tagsPatchOperation](https://learn.microsoft.com/rest/api/resources/tags/update-at-scope?view=rest-resources-2021-04-01#tagspatchoperation).
#[derive(Debug, Clone, Copy, Facet, Arbitrary)]
#[repr(u8)]
pub enum AzureResourceTagsPatchOperation {
    Replace,
    Merge,
    Delete,
}
cloud_terrastodon_registry::register_thing!(AzureResourceTagsPatchOperation);
cloud_terrastodon_registry::register_arbitrary!(AzureResourceTagsPatchOperation);
