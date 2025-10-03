use crate::location::LocationName;
use crate::prelude::ComputeSkuName;
use std::fmt;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeSku {
    #[serde(default)]
    pub capabilities: Vec<ComputeSkuCapability>,
    pub location_info: Vec<ComputeSkuLocationInfo>,
    pub locations: Vec<LocationName>,
    pub name: ComputeSkuName,
    pub resource_type: ComputeSkuResourceType,
    pub restrictions: Vec<ComputeSkuRestriction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputeSkuResourceType {
    VirtualMachines,
    AvailabilitySets,
    HostGroupsSlashHosts,
    Disks,
    Snapshots,
    Other(String),
}

impl serde::Serialize for ComputeSkuResourceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            ComputeSkuResourceType::VirtualMachines => "virtualMachines",
            ComputeSkuResourceType::AvailabilitySets => "availabilitySets",
            ComputeSkuResourceType::HostGroupsSlashHosts => "hostGroups/hosts",
            ComputeSkuResourceType::Disks => "disks",
            ComputeSkuResourceType::Snapshots => "snapshots",
            ComputeSkuResourceType::Other(other) => other.as_str(),
        };
        serializer.serialize_str(value)
    }
}

impl<'de> serde::Deserialize<'de> for ComputeSkuResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResourceTypeVisitor;

        impl<'de> serde::de::Visitor<'de> for ResourceTypeVisitor {
            type Value = ComputeSkuResourceType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a virtual machine SKU resource type")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "virtualMachines" => Ok(ComputeSkuResourceType::VirtualMachines),
                    "availabilitySets" => Ok(ComputeSkuResourceType::AvailabilitySets),
                    "hostGroups/hosts" => Ok(ComputeSkuResourceType::HostGroupsSlashHosts),
                    "disks" => Ok(ComputeSkuResourceType::Disks),
                    "snapshots" => Ok(ComputeSkuResourceType::Snapshots),
                    other => Ok(ComputeSkuResourceType::Other(other.to_owned())),
                }
            }
        }

        deserializer.deserialize_str(ResourceTypeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_known_variants() {
        let cases = vec![
            ("virtualMachines", ComputeSkuResourceType::VirtualMachines),
            ("availabilitySets", ComputeSkuResourceType::AvailabilitySets),
            (
                "hostGroups/hosts",
                ComputeSkuResourceType::HostGroupsSlashHosts,
            ),
            ("disks", ComputeSkuResourceType::Disks),
            ("snapshots", ComputeSkuResourceType::Snapshots),
        ];

        for (expected, variant) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, format!("\"{expected}\""));

            let parsed: ComputeSkuResourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn roundtrip_other_variant() {
        let json = "\"brandNewType\"";
        let parsed: ComputeSkuResourceType = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            ComputeSkuResourceType::Other("brandNewType".to_owned())
        );
        assert_eq!(serde_json::to_string(&parsed).unwrap(), json);
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeSkuCapability {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeSkuLocationInfo {
    pub location: LocationName,
    pub zones: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeSkuRestriction {
    pub reason_code: String,
    pub restriction_info: ComputeSkuRestrictionInfo,
    #[serde(rename = "type")]
    pub kind: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeSkuRestrictionInfo {
    pub locations: Vec<String>,
}
