use crate::location::LocationName;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    #[serde(default)]
    pub availability_zone_mappings: Vec<LocationAvailabilityZoneMapping>,
    pub metadata: LocationMetadata,
    pub name: LocationName,
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LocationAvailabilityZoneMapping {
    pub logical_zone: String,
    pub physical_zone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "regionType")]
pub enum LocationMetadata {
    #[serde(rename_all = "camelCase")]
    Physical {
        geography: String,
        geography_group: String,
        latitude: String,
        longitude: String,
        paired_region: Vec<LocationPairedRegion>,
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
pub struct LocationPairedRegion {
    pub id: String,
    pub name: LocationName,
}
