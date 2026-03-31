use crate::location::AzureLocationName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AzureLocation {
    #[serde(default)]
    pub availability_zone_mappings: Vec<AzureLocationAvailabilityZoneMapping>,
    pub metadata: AzureLocationMetadata,
    pub name: AzureLocationName,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AzureLocationAvailabilityZoneMapping {
    pub logical_zone: String,
    pub physical_zone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "regionType")]
pub enum AzureLocationMetadata {
    #[serde(rename_all = "camelCase")]
    Physical {
        geography: String,
        geography_group: String,
        latitude: String,
        longitude: String,
        paired_region: Vec<AzureLocationPairedRegion>,
        physical_location: String,
        region_category: String,
    },
    #[serde(rename_all = "camelCase")]
    Logical {
        geography: Option<String>,
        geography_group: Option<String>,
        region_category: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct AzureLocationPairedRegion {
    pub id: String,
    pub name: AzureLocationName,
}
