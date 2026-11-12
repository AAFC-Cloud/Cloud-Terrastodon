use cloud_terrastodon_azure::EntraPrincipal;
use cloud_terrastodon_azure::UnifiedRoleAssignment;
use cloud_terrastodon_azure::UnifiedRoleDefinition;
use facet::Facet;
use std::sync::Arc;

/// A selected Entra assignment with its shared definition and optional principal.
#[derive(Facet)]
pub struct AzureEntraRoleAssignmentBrowseOutput {
    pub role_assignment: Arc<UnifiedRoleAssignment>,
    pub role_definition: Arc<UnifiedRoleDefinition>,
    pub principal: Option<EntraPrincipal>,
}
