use crate::AzureDevOpsDescriptor;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::ArbitraryJson;
use compact_str::CompactString;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsServiceEndpointCreatedBy {
    pub descriptor: AzureDevOpsDescriptor,
    pub directory_alias: ArbitraryJson,
    pub display_name: CompactString,
    pub id: Uuid,
    pub image_url: String,
    pub inactive: Option<ArbitraryJson>,
    pub is_aad_identity: Option<ArbitraryJson>,
    pub is_container: Option<ArbitraryJson>,
    pub is_deleted_in_origin: Option<ArbitraryJson>,
    pub profile_url: Option<ArbitraryJson>,
    pub unique_name: String,
    pub url: String,
}
