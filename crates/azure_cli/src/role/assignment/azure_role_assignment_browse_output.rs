use cloud_terrastodon_azure::EntraPrincipal;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleDefinition;
use facet::Facet;
use std::sync::Arc;

/// A selected Azure assignment with its shared definition and optional principal.
#[derive(Facet)]
pub struct AzureRoleAssignmentBrowseOutput {
    pub role_assignment: Arc<RoleAssignment>,
    pub role_definition: Arc<RoleDefinition>,
    pub principal: Option<EntraPrincipal>,
}
