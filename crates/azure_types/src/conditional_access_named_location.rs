use crate::prelude::ConditionalAccessNamedLocationId;
use chrono::DateTime;
use chrono::Utc;
use compact_str::CompactString;
use ipnetwork::Ipv4Network;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "@odata.type")]
pub enum ConditionalAccessNamedLocation {
    #[serde(rename = "#microsoft.graph.ipNamedLocation")]
    IpNamedLocation(ConditionalAccessIpNamedLocation),
    #[serde(rename = "#microsoft.graph.countryNamedLocation")]
    CountryNamedLocation(ConditionalAccessCountryNamedLocation),
}
impl ConditionalAccessNamedLocation {
    pub fn id(&self) -> &ConditionalAccessNamedLocationId {
        match self {
            ConditionalAccessNamedLocation::IpNamedLocation(location) => &location.id,
            ConditionalAccessNamedLocation::CountryNamedLocation(location) => &location.id,
        }
    }
    pub fn ips(&self) -> Vec<&Ipv4Network> {
        match self {
            ConditionalAccessNamedLocation::IpNamedLocation(location) => location
                .ip_ranges
                .iter()
                .map(|ip| &ip.cidr_address)
                .collect(),
            ConditionalAccessNamedLocation::CountryNamedLocation(_) => Vec::new(),
        }
    }
    pub fn display_name(&self) -> &CompactString {
        match self {
            ConditionalAccessNamedLocation::IpNamedLocation(location) => &location.display_name,
            ConditionalAccessNamedLocation::CountryNamedLocation(location) => &location.display_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessIpNamedLocation {
    pub id: ConditionalAccessNamedLocationId,
    pub display_name: CompactString,
    pub modified_date_time: Option<DateTime<Utc>>,
    pub created_date_time: Option<DateTime<Utc>>,
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub is_trusted: bool,
    pub ip_ranges: Vec<CidrHolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CidrHolder {
    pub cidr_address: Ipv4Network,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalAccessCountryNamedLocation {
    pub id: ConditionalAccessNamedLocationId,
    pub display_name: CompactString,
    pub modified_date_time: Option<DateTime<Utc>>,
    pub created_date_time: Option<DateTime<Utc>>,
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub countries_and_regions: Vec<CompactString>,
    pub include_unknown_countries_and_regions: bool,
    pub country_lookup_method: CompactString,
}
