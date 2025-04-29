use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum PimEntraRoleDefinitionKind {
    BuiltInRole,
    CustomRole,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PimEntraRoleDefinition {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: Uuid,
    #[serde(rename = "type")]
    pub kind: PimEntraRoleDefinitionKind,
}
