use cloud_terrastodon_azure::GovernanceRoleAssignment;
use cloud_terrastodon_azure::PimEntraRoleDefinition;
use std::sync::Arc;

/// A signed-in user's Entra PIM assignment alongside its available role metadata.
#[derive(Debug)]
pub(crate) struct PimEntraRoleAssignment {
    pub assignment: Arc<GovernanceRoleAssignment>,
    pub definition: Option<Arc<PimEntraRoleDefinition>>,
}
