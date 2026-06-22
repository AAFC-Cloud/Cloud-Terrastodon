use crate::ConditionalAccessNamedLocationId;
use chrono::DateTime;
use chrono::Utc;
use compact_str::CompactString;
use ipnetwork::Ipv4Network;

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[repr(C)]
#[facet(tag = "@odata.type")]
pub enum ConditionalAccessNamedLocation {
    #[facet(rename = "#microsoft.graph.ipNamedLocation")]
    IpNamedLocation(ConditionalAccessIpNamedLocation),
    #[facet(rename = "#microsoft.graph.countryNamedLocation")]
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
            ConditionalAccessNamedLocation::CountryNamedLocation(location) => {
                &location.display_name
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessIpNamedLocation {
    pub id: ConditionalAccessNamedLocationId,
    pub display_name: CompactString,
    #[facet(default)]
    pub modified_date_time: Option<DateTime<Utc>>,
    #[facet(default)]
    pub created_date_time: Option<DateTime<Utc>>,
    #[facet(default)]
    pub deleted_date_time: Option<DateTime<Utc>>,
    pub is_trusted: bool,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<CidrHolder>)]
    pub ip_ranges: Vec<CidrHolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct CidrHolder {
    #[facet(opaque, proxy = crate::Ipv4NetworkProxy)]
    pub cidr_address: Ipv4Network,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ConditionalAccessCountryNamedLocation {
    pub id: ConditionalAccessNamedLocationId,
    pub display_name: CompactString,
    #[facet(default)]
    pub modified_date_time: Option<DateTime<Utc>>,
    #[facet(default)]
    pub created_date_time: Option<DateTime<Utc>>,
    #[facet(default)]
    pub deleted_date_time: Option<DateTime<Utc>>,
    #[facet(default, opaque, proxy = crate::VecDefaultNullProxy<CompactString>)]
    pub countries_and_regions: Vec<CompactString>,
    pub include_unknown_countries_and_regions: bool,
    pub country_lookup_method: CompactString,
}
