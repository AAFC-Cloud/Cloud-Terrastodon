use arbitrary::Arbitrary;
use compact_str::CompactString;
use ipnetwork::Ipv4Network;

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
impl std::str::FromStr for AddressPrefix {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.parse::<Ipv4Network>().map(AddressPrefix::Ipv4) {
            Ok(address_prefix) => address_prefix,
            Err(_) => AddressPrefix::Other(CompactString::new(s)),
        })
    }
}

// There is no feature in `ipnetwork` or `arbitrary` crates to provide this, so we have to do it our selves for `Ipv4Network`
// This means we cannot use the arbitrary derive macro on `AddressPrefix` until we create an `Ipv4Network` newtype
impl<'a> Arbitrary<'a> for AddressPrefix {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        if bool::arbitrary(u)? {
            let third = u.int_in_range(0..=255)?;
            let prefix = u.int_in_range(16..=30)?;
            let network = Ipv4Network::new(std::net::Ipv4Addr::new(10, 0, third, 0), prefix)
                .map_err(|_| arbitrary::Error::IncorrectFormat)?;
            Ok(Self::Ipv4(network))
        } else {
            let mut value = CompactString::arbitrary(u)?;
            if value.is_empty() {
                value.push('x');
            }
            Ok(Self::Other(value))
        }
    }
}

cloud_terrastodon_registry::register_thing!(AddressPrefix);
cloud_terrastodon_registry::register_arbitrary!(AddressPrefix);

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
