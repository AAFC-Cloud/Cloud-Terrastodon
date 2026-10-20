use crate::domain::DesiredPrincipal;
use crate::domain::ReconciliationMode;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraGroupDisplayName;
use cloud_terrastodon::azure_types::EntraGroupId;

/// Desired relationships for an existing group, identified within its tenant.
///
/// `displayName` is an expected annotation, not a rename instruction. Both
/// relationships require an explicit mode and list. `ignore` with an empty list
/// leaves a relationship unmanaged.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesiredGroup {
    pub tenant_id: AzureTenantId,
    pub group_id: EntraGroupId,
    pub display_name: EntraGroupDisplayName,
    pub members_mode: ReconciliationMode,
    pub members: Vec<DesiredPrincipal>,
    pub owners_mode: ReconciliationMode,
    pub owners: Vec<DesiredPrincipal>,
}
