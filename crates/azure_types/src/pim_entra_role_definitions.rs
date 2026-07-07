use arbitrary::Arbitrary;
use uuid::Uuid;

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum PimEntraRoleDefinitionKind {
    BuiltInRole,
    CustomRole,
}

#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct PimEntraRoleDefinition {
    #[facet(rename = "displayName")]
    pub display_name: String,
    pub id: Uuid,
    #[facet(rename = "type")]
    pub kind: PimEntraRoleDefinitionKind,
}

cloud_terrastodon_registry::register_thing!(PimEntraRoleDefinition);
cloud_terrastodon_registry::register_arbitrary!(PimEntraRoleDefinition);
cloud_terrastodon_registry::register_arbitrary!(Vec<PimEntraRoleDefinition>);
