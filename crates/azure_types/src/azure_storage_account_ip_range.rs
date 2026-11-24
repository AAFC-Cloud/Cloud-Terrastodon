use arbitrary::Arbitrary;
use facet::Facet;
use ipnetwork::IpNetwork;
use std::fmt::Display;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

/// An individual IP address or CIDR range admitted by a storage firewall rule.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#iprule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Facet)]
#[facet(opaque, proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountIpRange {
    Address(IpAddr),
    Network(IpNetwork),
}

impl AzureStorageAccountIpRange {
    /// Check whether an address is included in the configured rule.
    pub fn contains(self, address: IpAddr) -> bool {
        match self {
            Self::Address(rule) => rule == address,
            Self::Network(rule) => match (rule.ip(), address) {
                (IpAddr::V4(network), IpAddr::V4(address)) => {
                    let mask = !u32::MAX.checked_shr(u32::from(rule.prefix())).unwrap_or(0);
                    u32::from(network) & mask == u32::from(address) & mask
                }
                (IpAddr::V6(network), IpAddr::V6(address)) => {
                    let mask = !u128::MAX.checked_shr(u32::from(rule.prefix())).unwrap_or(0);
                    u128::from(network) & mask == u128::from(address) & mask
                }
                _ => false,
            },
        }
    }
}

impl FromStr for AzureStorageAccountIpRange {
    type Err = eyre::Report;

    fn from_str(value: &str) -> eyre::Result<Self> {
        if value.contains('/') {
            Ok(Self::Network(value.parse()?))
        } else {
            Ok(Self::Address(value.parse()?))
        }
    }
}

impl Display for AzureStorageAccountIpRange {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address(address) => address.fmt(formatter),
            Self::Network(network) => network.fmt(formatter),
        }
    }
}

impl<'a> Arbitrary<'a> for AzureStorageAccountIpRange {
    fn arbitrary(input: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let address = if bool::arbitrary(input)? {
            IpAddr::V4(Ipv4Addr::from(<[u8; 4]>::arbitrary(input)?))
        } else {
            IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::arbitrary(input)?))
        };
        if bool::arbitrary(input)? {
            let prefix = input.int_in_range(0..=if address.is_ipv4() { 32 } else { 128 })?;
            Ok(Self::Network(
                IpNetwork::new(address, prefix).map_err(|_| arbitrary::Error::IncorrectFormat)?,
            ))
        } else {
            Ok(Self::Address(address))
        }
    }
}

crate::impl_facet_string_proxy!(AzureStorageAccountIpRange, value => value.to_string());
cloud_terrastodon_registry::register_thing!(AzureStorageAccountIpRange);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountIpRange);

#[cfg(test)]
mod tests {
    use super::AzureStorageAccountIpRange;

    #[test]
    fn distinguishes_addresses_and_networks_without_accepting_arbitrary_strings() -> eyre::Result<()>
    {
        assert!(matches!(
            "192.0.2.1".parse::<AzureStorageAccountIpRange>()?,
            AzureStorageAccountIpRange::Address(_)
        ));
        assert!(matches!(
            "2001:db8::/64".parse::<AzureStorageAccountIpRange>()?,
            AzureStorageAccountIpRange::Network(_)
        ));
        for invalid in ["AzureCloud", "192.0.2.0/33", "2001:db8::/129", ""] {
            assert!(invalid.parse::<AzureStorageAccountIpRange>().is_err());
        }
        Ok(())
    }

    #[test]
    fn matches_only_addresses_within_the_same_ip_rule_family() -> eyre::Result<()> {
        for (rule, matching, outside) in [
            ("192.0.2.8", "192.0.2.8", "192.0.2.9"),
            ("192.0.2.8/28", "192.0.2.15", "192.0.2.16"),
            ("192.0.2.8/32", "192.0.2.8", "192.0.2.9"),
            ("0.0.0.0/0", "192.0.2.8", "::ffff:192.0.2.8"),
            ("2001:db8::1/64", "2001:db8::f", "2001:db8:0:1::1"),
            ("2001:db8::1/128", "2001:db8::1", "2001:db8::2"),
            ("::/0", "2001:db8::1", "192.0.2.8"),
        ] {
            let rule: AzureStorageAccountIpRange = rule.parse()?;
            assert!(rule.contains(matching.parse()?));
            assert!(!rule.contains(outside.parse()?));
        }
        Ok(())
    }
}
