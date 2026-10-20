use crate::domain::ObservedPrincipal;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraGroupDisplayName;
use cloud_terrastodon::azure_types::EntraGroupId;

/// Fresh direct relationships for an existing group in a particular tenant.
///
/// In this milestone, a false completeness flag records the documented Graph
/// v1.0 omission of service principals. User/group observations can support
/// additive proposals, but exact reconciliation and absent service principals
/// cannot be inferred safely. Failed pages, inaccessible relationships, or other
/// unknown omissions must fail acquisition rather than produce this snapshot.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct GroupSnapshot {
    pub tenant_id: AzureTenantId,
    pub group_id: EntraGroupId,
    pub display_name: EntraGroupDisplayName,
    pub members: Vec<ObservedPrincipal>,
    pub owners: Vec<ObservedPrincipal>,
    pub members_complete: bool,
    pub owners_complete: bool,
}
