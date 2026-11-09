use cloud_terrastodon_azure::EntraDirectoryObject;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::RoleDefinitionsAndAssignments;
use std::sync::Arc;

/// A completed group-report inventory, retaining its association to the group.
pub(super) enum AzureEntraGroupShowLookupResult {
    Members {
        group_id: EntraGroupId,
        members: Vec<Arc<EntraDirectoryObject>>,
    },
    Owners {
        group_id: EntraGroupId,
        owners: Vec<Arc<EntraDirectoryObject>>,
    },
    RoleAssignments(Arc<RoleDefinitionsAndAssignments>),
}
