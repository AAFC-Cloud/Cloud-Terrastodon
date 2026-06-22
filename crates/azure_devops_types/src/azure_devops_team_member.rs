use crate::AzureDevOpsDescriptor;
use facet_json::RawJson;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTeamMember {
    pub identity: AzureDevOpsTeamMemberIdentity,
    pub is_team_admin: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTeamMemberIdentity {
    pub descriptor: AzureDevOpsDescriptor,
    pub directory_alias: Option<RawJson<'static>>,
    pub display_name: String,
    pub id: Uuid,
    pub image_url: String,
    pub inactive: Option<RawJson<'static>>,
    pub is_aad_identity: RawJson<'static>,
    pub is_container: Option<RawJson<'static>>,
    pub is_deleted_in_origin: Option<RawJson<'static>>,
    pub profile_url: Option<RawJson<'static>>,
    pub unique_name: String,
    pub url: String,
}
