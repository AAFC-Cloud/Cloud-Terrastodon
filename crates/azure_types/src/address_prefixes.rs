use crate::prelude::AddressPrefix;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::{self};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressPrefixes {
    pub address_prefixes: Vec<AddressPrefix>,
}

impl Serialize for AddressPrefixes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        let values: Vec<String> = self
            .address_prefixes
            .iter()
            .map(|p| p.to_string())
            .collect();
        map.serialize_entry("addressPrefixes", &values)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for AddressPrefixes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PrefixesHelper {
            #[serde(rename = "addressPrefix")]
            single: Option<String>,
            #[serde(rename = "addressPrefixes")]
            multiple: Option<Vec<String>>,
        }

        let h = PrefixesHelper::deserialize(deserializer)?;
        let mut result = Vec::new();
        if let Some(s) = h.single {
            result.push(s.parse().map_err(de::Error::custom)?);
        }
        if let Some(m) = h.multiple {
            for prefix in m {
                result.push(prefix.parse().map_err(de::Error::custom)?);
            }
        }
        Ok(AddressPrefixes {
            address_prefixes: result,
        })
    }
}
