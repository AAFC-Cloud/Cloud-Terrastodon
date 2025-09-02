use compact_str::CompactString;
use ipnetwork::Ipv4Network;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum AddressPrefix {
    Ipv4(Ipv4Network),
    Other(CompactString),
}
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
impl Serialize for AddressPrefix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AddressPrefix::Ipv4(value) => value.serialize(serializer),
            AddressPrefix::Other(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AddressPrefix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = AddressPrefix::from_str(&expanded)
            .map_err(|e| serde::de::Error::custom(format!("{}", e)))?;
        Ok(id)
    }
}
