use crate::AsScope;
use crate::AzureLocationName;
use crate::AzureNetworkSecurityGroupProperties;
use crate::AzureResourceGroupId;
use crate::ResourceId;
use crate::ResourceType;
use crate::Scope;
use crate::StringMapDefaultNullProxy;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use facet::Facet;
use std::collections::HashMap;

/// A network security group from Azure Resource Manager or Azure Resource Graph.
/// [Azure Network 2026-01-01 resource schema](https://learn.microsoft.com/en-us/rest/api/virtualnetwork/network-security-groups/get?view=rest-virtualnetwork-2026-01-01#networksecuritygroup).
#[derive(Debug, Clone, Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureNetworkSecurityGroup {
    pub id: ResourceId,
    pub name: String,
    pub location: AzureLocationName,
    pub etag: Option<String>,
    #[facet(rename = "type")]
    pub resource_type: ResourceType,
    #[facet(default, proxy = StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    pub properties: AzureNetworkSecurityGroupProperties,
}

impl<'a> Arbitrary<'a> for AzureNetworkSecurityGroup {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let name = format!("nsg-{}", u32::arbitrary(input)?);
        let resource_type = ResourceType::MICROSOFT_DOT_NETWORK_SLASH_NETWORKSECURITYGROUPS;
        Ok(Self {
            id: ResourceId::new(
                AzureResourceGroupId::arbitrary(input)?,
                resource_type.clone(),
                name.as_str(),
            ),
            name,
            location: AzureLocationName::arbitrary(input)?,
            etag: Option::arbitrary(input)?,
            resource_type,
            tags: HashMap::arbitrary(input)?,
            properties: AzureNetworkSecurityGroupProperties::arbitrary(input)?,
        })
    }
}

impl AsScope for AzureNetworkSecurityGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

cloud_terrastodon_registry::register_thing!(AzureNetworkSecurityGroup);
cloud_terrastodon_registry::register_arbitrary!(AzureNetworkSecurityGroup);
