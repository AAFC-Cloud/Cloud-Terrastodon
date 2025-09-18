use compact_str::CompactString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct GovernanceRoleDefinition {
    pub display_name: CompactString,
    pub external_id: String,
    pub resource_id: String,
    pub template_id: String,
}