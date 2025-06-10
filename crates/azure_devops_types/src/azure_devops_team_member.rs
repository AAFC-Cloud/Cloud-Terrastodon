use crate::prelude::AzureDevOpsDescriptor;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsTeamMember {
    pub identity: AzureDevOpsTeamMemberIdentity,
    pub is_team_admin: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureDevOpsTeamMemberIdentity {
    pub descriptor: AzureDevOpsDescriptor,
    pub directory_alias: Option<Value>,
    pub display_name: String,
    pub id: Uuid,
    pub image_url: String,
    pub inactive: Option<Value>,
    pub is_aad_identity: Value,
    pub is_container: Option<Value>,
    pub is_deleted_in_origin: Option<Value>,
    pub profile_url: Option<Value>,
    pub unique_name: String,
    pub url: String,
}
