use arbitrary::Arbitrary;
use facet::Facet;
use uuid::Uuid;

#[derive(Debug, Clone, Arbitrary, Facet)]
#[repr(C)]
pub enum PimEntraRoleDefinitionKind {
    BuiltInRole,
    CustomRole,
}

#[derive(Debug, Clone, Arbitrary, Facet)]
pub struct PimEntraRoleDefinition {
    #[facet(rename = "displayName")]
    pub display_name: String,
    pub id: Uuid,
    #[facet(rename = "type")]
    pub kind: PimEntraRoleDefinitionKind,
}

cloud_terrastodon_registry::register_thing!(PimEntraRoleDefinition);
cloud_terrastodon_registry::register_arbitrary!(PimEntraRoleDefinition);
