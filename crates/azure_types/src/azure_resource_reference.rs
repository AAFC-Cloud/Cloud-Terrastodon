use arbitrary::Arbitrary;
use facet::Facet;

/// An ARM [subresource reference] identified by its resource ID.
///
/// [subresource reference]: https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#subresource
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureResourceReference<Id> {
    pub id: Id,
}
