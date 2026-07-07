use crate::AzureDevOpsDescriptor;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTeamMember {
    pub identity: AzureDevOpsTeamMemberIdentity,
    pub is_team_admin: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsTeamMemberIdentity {
    pub descriptor: AzureDevOpsDescriptor,
    pub directory_alias: Option<ArbitraryJson>,
    pub display_name: String,
    pub id: Uuid,
    pub image_url: String,
    pub inactive: Option<ArbitraryJson>,
    pub is_aad_identity: ArbitraryJson,
    pub is_container: Option<ArbitraryJson>,
    pub is_deleted_in_origin: Option<ArbitraryJson>,
    pub profile_url: Option<ArbitraryJson>,
    pub unique_name: String,
    pub url: String,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsTeamMember);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsTeamMember);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsTeamMember>);
