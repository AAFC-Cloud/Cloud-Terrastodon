use crate::AsScope;
use crate::AzureLocationName;
use crate::AzureResourceGroupId;
use crate::ResourceProperties;
use crate::ResourceType;
use crate::Scope;
use crate::ScopeImpl;
use crate::ScopeImplKind;
use crate::scopes::HasPrefix;
use crate::scopes::TryFromResourceScoped;
use crate::slug::HasSlug;
use arbitrary::Arbitrary;
use compact_str::CompactString;
use eyre::Result;
use facet::Facet;
use std::collections::HashMap;
use std::str::FromStr;

/// This is the ID for an ill-defined resource that is specifically the child of a resource group.
/// Some things are children of things that are children of resource groups, which this would not apply to.
/// At some point, this should be replaced with ScopeImpl or something in the fields where this type is used.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Facet, Arbitrary)]
#[facet(json::proxy = String)]
pub struct ResourceId {
    pub resource_group_id: AzureResourceGroupId,
    pub resource_type: ResourceType,
    pub resource_name: CompactString,
}
crate::impl_facet_string_proxy!(ResourceId, value => value.expanded_form());
impl ResourceId {
    pub fn new(
        resource_group_id: AzureResourceGroupId,
        resource_type: ResourceType,
        name: impl Into<CompactString>,
    ) -> Self {
        Self {
            resource_group_id,
            resource_type,
            resource_name: name.into(),
        }
    }
}
impl HasSlug for ResourceId {
    type Name = CompactString;

    fn name(&self) -> &Self::Name {
        &self.resource_name
    }
}
impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded_form().to_string().as_str())
    }
}
impl HasPrefix for ResourceId {
    fn get_prefix() -> &'static str {
        ""
    }
}
impl TryFromResourceScoped for ResourceId {
    unsafe fn new_resource_scoped_unchecked(
        _expanded: &str,
        resource_id: ResourceId,
        _resource_name: Self::Name,
    ) -> Self {
        resource_id
    }
}

impl Scope for ResourceId {
    type Err = <Self as std::str::FromStr>::Err;
    fn expanded_form(&self) -> String {
        format!(
            "{}/providers/{}/{}",
            self.resource_group_id, self.resource_type, self.resource_name
        )
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceId::try_from_expanded_resource_scoped(expanded)
    }

    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::Resource(self.clone())
    }
    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::Unknown
    }
}
impl FromStr for ResourceId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::try_from_expanded(s)
    }
}

#[derive(Debug, Eq, PartialEq, Facet)]
pub struct Resource {
    pub id: ScopeImpl,
    #[facet(proxy = crate::ResourceTypeProxy)]
    pub kind: ResourceType,
    pub name: String,
    /// Region projected by [Azure Resource Graph](https://learn.microsoft.com/en-us/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2024-04-01).
    /// Older snapshots and resources with no reported location can omit it.
    #[facet(default)]
    pub location: Option<AzureLocationName>,
    #[facet(default, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    #[facet(default, proxy = Option<ResourceProperties>)]
    pub properties: ResourceProperties,
}

impl<'a> Arbitrary<'a> for Resource {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let name = String::arbitrary(u)?;
        Ok(Self {
            id: ScopeImpl::arbitrary(u)?,
            kind: ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
            properties: ResourceProperties {
                display_name: Some(name.clone()),
                ..ResourceProperties::default()
            },
            name,
            location: Option::arbitrary(u)?,
            tags: HashMap::arbitrary(u)?,
        })
    }
}
impl AsScope for Resource {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl Resource {
    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.properties.display_name.as_deref()
    }
}
impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.display_name() {
            Some(display_name) => f.write_fmt(format_args!(
                "{:64} {:63} {}",
                display_name, self.name, self.id
            )),
            None => f.write_fmt(format_args!("{:128} {}", self.name, self.id)),
        }
    }
}

cloud_terrastodon_registry::register_thing!(Resource);
cloud_terrastodon_registry::register_arbitrary!(Resource);

#[cfg(test)]
mod test {
    use crate::ArbitraryJson;
    use crate::AzureResourceGroupId;
    use crate::AzureResourceGroupName;
    use crate::ResourceId;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use cloud_terrastodon_azure_resource_types::ResourceType;
    use uuid::Uuid;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET/subnets/MY-Subnet/providers/Microsoft.Authorization/roleAssignments/{nil}
        let id = ResourceId::new(
            AzureResourceGroupId::new(
                SubscriptionId::new(Uuid::nil()),
                AzureResourceGroupName::try_new("MY-RG")?,
            ),
            ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
            "MY-VNET",
        );
        assert_eq!(
            id.expanded_form(),
            format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET",
                nil = Uuid::nil()
            )
        );
        Ok(())
    }

    #[test]
    fn resource_id_json_roundtrips() -> eyre::Result<()> {
        let id = ResourceId::new(
            AzureResourceGroupId::new(
                SubscriptionId::new(Uuid::nil()),
                AzureResourceGroupName::try_new("MY-RG")?,
            ),
            ResourceType::MICROSOFT_DOT_NETWORK_SLASH_VIRTUALNETWORKS,
            "MY-VNET",
        );
        crate::facet_json_equivalence::assert_json_serialize_equivalent(&id)?;
        crate::facet_json_equivalence::assert_json_roundtrip_equivalent::<ResourceId>(
            "\"/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET\"",
        )?;
        Ok(())
    }

    #[test]
    fn deserializes_resource_properties() -> eyre::Result<()> {
        let id = format!(
            "/subscriptions/{nil}/resourceGroups/my-rg/providers/Microsoft.Network/publicIPAddresses/my-ip",
            nil = Uuid::nil()
        );
        let resource: crate::Resource = facet_json::from_str(&format!(
            r#"{{
            "id": "{id}",
            "kind": "Microsoft.Network/publicIPAddresses",
            "name": "my-ip",
            "properties": {{
                "displayName": "My IP",
                "provisioningState": "Succeeded",
                "ipAddress": "203.0.113.10"
            }},
            "tags": {{
                "env": "test"
            }}
        }}"#
        ))?;

        assert_eq!(resource.display_name(), Some("My IP"));
        assert_eq!(
            resource.properties.provisioning_state.as_deref(),
            Some("Succeeded")
        );
        assert_eq!(
            resource.properties.other.get("ipAddress"),
            Some(&ArbitraryJson::from_text("203.0.113.10"))
        );
        assert!(!resource.properties.other.contains_key("displayName"));
        assert!(!resource.properties.other.contains_key("provisioningState"));
        let round_tripped: crate::Resource =
            facet_json::from_str(&facet_json::to_string(&resource)?)?;
        assert_eq!(round_tripped, resource);

        Ok(())
    }

    #[test]
    fn deserializes_null_properties_to_empty_object() -> eyre::Result<()> {
        let id = format!(
            "/subscriptions/{nil}/resourceGroups/my-rg/providers/Contoso.Widgets/widgets/my-widget",
            nil = Uuid::nil()
        );
        let resource: crate::Resource = facet_json::from_str(&format!(
            r#"{{
            "id": "{id}",
            "kind": "Contoso.Widgets/widgets",
            "name": "my-widget",
            "properties": null,
            "tags": null
        }}"#
        ))?;

        assert_eq!(resource.display_name(), None);
        assert_eq!(resource.properties, crate::ResourceProperties::default());
        assert!(resource.tags.is_empty());

        Ok(())
    }
}
