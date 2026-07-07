use crate::ComputeSkuName;
use crate::location::AzureLocationName;
use arbitrary::Arbitrary;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputeSku {
    #[facet(default)]
    pub capabilities: Vec<ComputeSkuCapability>,
    pub location_info: Vec<ComputeSkuLocationInfo>,
    pub locations: Vec<AzureLocationName>,
    pub name: ComputeSkuName,
    pub resource_type: ComputeSkuResourceType,
    pub restrictions: Vec<ComputeSkuRestriction>,
}

#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum ComputeSkuResourceType {
    VirtualMachines,
    AvailabilitySets,
    HostGroupsSlashHosts,
    Disks,
    Snapshots,
    Other(String),
}
crate::impl_facet_string_proxy!(ComputeSkuResourceType, value => value.to_string());

impl core::fmt::Display for ComputeSkuResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ComputeSkuResourceType::VirtualMachines => "virtualMachines",
            ComputeSkuResourceType::AvailabilitySets => "availabilitySets",
            ComputeSkuResourceType::HostGroupsSlashHosts => "hostGroups/hosts",
            ComputeSkuResourceType::Disks => "disks",
            ComputeSkuResourceType::Snapshots => "snapshots",
            ComputeSkuResourceType::Other(other) => other.as_str(),
        };
        write!(f, "{value}")
    }
}

impl std::str::FromStr for ComputeSkuResourceType {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "virtualMachines" => ComputeSkuResourceType::VirtualMachines,
            "availabilitySets" => ComputeSkuResourceType::AvailabilitySets,
            "hostGroups/hosts" => ComputeSkuResourceType::HostGroupsSlashHosts,
            "disks" => ComputeSkuResourceType::Disks,
            "snapshots" => ComputeSkuResourceType::Snapshots,
            other => ComputeSkuResourceType::Other(other.to_owned()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_known_variants() -> eyre::Result<()> {
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
            let json = facet_json::to_string(&variant)?;
            assert_eq!(json, format!("\"{expected}\""));

            let parsed: ComputeSkuResourceType = facet_json::from_str(&json)?;
            assert_eq!(parsed, variant);
        }
        Ok(())
    }

    #[test]
    fn roundtrip_other_variant() -> eyre::Result<()> {
        let json = "\"brandNewType\"";
        let parsed: ComputeSkuResourceType = facet_json::from_str(json)?;
        assert_eq!(
            parsed,
            ComputeSkuResourceType::Other("brandNewType".to_owned())
        );
        assert_eq!(facet_json::to_string(&parsed)?, json);
        Ok(())
    }

    #[test]
    fn sku_json_defaults_missing_capabilities() -> eyre::Result<()> {
        let json = r#"
        {
            "locationInfo": [],
            "locations": ["eastus"],
            "name": "Standard_D2s_v5",
            "resourceType": "virtualMachines",
            "restrictions": []
        }
        "#;

        let sku = facet_json::from_str::<ComputeSku>(json)?;
        assert!(sku.capabilities.is_empty());
        assert_eq!(sku.resource_type, ComputeSkuResourceType::VirtualMachines);
        let reparsed = facet_json::from_str::<ComputeSku>(&facet_json::to_string(&sku)?)?;
        assert_eq!(sku, reparsed);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputeSkuCapability {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputeSkuLocationInfo {
    pub location: AzureLocationName,
    pub zones: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputeSkuRestriction {
    pub reason_code: String,
    pub restriction_info: ComputeSkuRestrictionInfo,
    #[facet(rename = "type")]
    pub kind: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct ComputeSkuRestrictionInfo {
    pub locations: Vec<String>,
}

cloud_terrastodon_registry::register_thing!(ComputeSkuResourceType);
cloud_terrastodon_registry::register_arbitrary!(ComputeSkuResourceType);
cloud_terrastodon_registry::register_thing!(ComputeSku);
cloud_terrastodon_registry::register_arbitrary!(ComputeSku);
cloud_terrastodon_registry::register_arbitrary!(Vec<ComputeSku>);
