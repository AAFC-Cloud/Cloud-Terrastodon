use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RoleDefinitionKind;
use facet::Facet;
use std::sync::Arc;

#[derive(Facet)]
pub(crate) struct ScopeRoleDefinitionRow {
    pub definition: Arc<RoleDefinition>,
    pub role: String,
    pub kind: String,
    pub description: String,
}

impl From<Arc<RoleDefinition>> for ScopeRoleDefinitionRow {
    fn from(definition: Arc<RoleDefinition>) -> Self {
        Self {
            role: definition.display_name.clone(),
            kind: match definition.kind {
                RoleDefinitionKind::BuiltInRole => "Built-in role".into(),
                RoleDefinitionKind::CustomRole => "Custom role".into(),
            },
            description: definition.description.clone(),
            definition,
        }
    }
}
