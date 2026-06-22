use uuid::Uuid;

#[derive(Debug, Clone, facet::Facet)]
#[repr(C)]
pub enum PimEntraRoleDefinitionKind {
    BuiltInRole,
    CustomRole,
}

#[derive(Debug, Clone, facet::Facet)]
pub struct PimEntraRoleDefinition {
    #[facet(rename = "displayName")]
    pub display_name: String,
    pub id: Uuid,
    #[facet(rename = "type")]
    pub kind: PimEntraRoleDefinitionKind,
}
