use crate::domain::DesiredPrincipal;
use crate::domain::ObservedPrincipal;
use cloud_terrastodon::azure_types::AzureTenantId;

/// Completed lookups for one desired reference in its group's tenant.
///
/// For every supplied selector, the adapter must perform a fresh lookup. `None`
/// then means not found; transport/authentication failures must be returned as
/// errors rather than converted into this record. Both selectors are queried
/// independently so a reassigned UPN cannot silently select a different person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrincipalResolution {
    pub tenant_id: AzureTenantId,
    pub reference: DesiredPrincipal,
    pub by_id: Option<ObservedPrincipal>,
    pub by_upn: Option<ObservedPrincipal>,
}
