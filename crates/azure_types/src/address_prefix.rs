use compact_str::CompactString;
use ipnetwork::Ipv4Network;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Eq, facet::Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AddressPrefix {
    Ipv4(Ipv4Network),
    Other(CompactString),
}
crate::impl_facet_string_proxy!(AddressPrefix, value => value.to_string());
impl std::fmt::Display for AddressPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddressPrefix::Ipv4(ipv4_network) => ipv4_network.fmt(f),
            AddressPrefix::Other(value) => value.fmt(f),
        }
    }
}
impl FromStr for AddressPrefix {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match Ipv4Network::from_str(s).map(AddressPrefix::Ipv4) {
            Ok(address_prefix) => address_prefix,
            Err(_) => AddressPrefix::Other(CompactString::new(s)),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::AddressPrefix;

    #[test]
    fn json_round_trips_through_facet() -> eyre::Result<()> {
        let prefix = facet_json::from_str::<AddressPrefix>("\"10.0.0.0/24\"")?;
        assert_eq!(facet_json::to_string(&prefix)?, "\"10.0.0.0/24\"");
        let other = facet_json::from_str::<AddressPrefix>("\"AzureCloud\"")?;
        assert_eq!(facet_json::to_string(&other)?, "\"AzureCloud\"");
        Ok(())
    }
}
