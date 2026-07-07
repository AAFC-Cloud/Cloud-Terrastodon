use crate::location::AzureLocationName;
use arbitrary::Arbitrary;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureLocation {
    #[facet(default)]
    pub availability_zone_mappings: Vec<AzureLocationAvailabilityZoneMapping>,
    pub metadata: AzureLocationMetadata,
    pub name: AzureLocationName,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureLocationAvailabilityZoneMapping {
    pub logical_zone: String,
    pub physical_zone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
#[repr(C)]
#[facet(tag = "regionType")]
pub enum AzureLocationMetadata {
    #[facet(rename_all = "camelCase")]
    Physical {
        geography: String,
        geography_group: String,
        latitude: String,
        longitude: String,
        paired_region: Vec<AzureLocationPairedRegion>,
        physical_location: String,
        region_category: String,
    },
    #[facet(rename_all = "camelCase")]
    Logical {
        geography: Option<String>,
        geography_group: Option<String>,
        region_category: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, facet::Facet)]
pub struct AzureLocationPairedRegion {
    pub id: String,
    pub name: AzureLocationName,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_json_round_trips_through_facet() -> eyre::Result<()> {
        let json = r#"
        {
            "metadata": {
                "regionType": "Physical",
                "geography": "United States",
                "geographyGroup": "US",
                "latitude": "37.3719",
                "longitude": "-79.8164",
                "pairedRegion": [],
                "physicalLocation": "Virginia",
                "regionCategory": "Recommended"
            },
            "name": "eastus",
            "displayName": "East US"
        }
        "#;

        let location = facet_json::from_str::<AzureLocation>(json)?;
        assert!(location.availability_zone_mappings.is_empty());
        let reparsed = facet_json::from_str::<AzureLocation>(&facet_json::to_string(&location)?)?;
        assert_eq!(location, reparsed);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureLocationMetadata);
cloud_terrastodon_registry::register_arbitrary!(AzureLocationMetadata);
cloud_terrastodon_registry::register_thing!(AzureLocation);
cloud_terrastodon_registry::register_arbitrary!(AzureLocation);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureLocation>);
