use crate::AsScope;
use crate::ResourceGroupId;
use crate::ResourceType;
use crate::Scope;
use crate::ScopeImpl;
use crate::ScopeImplKind;
use crate::scopes::HasPrefix;
use crate::scopes::TryFromResourceScoped;
use crate::slug::HasSlug;
use compact_str::CompactString;
use eyre::Result;
use facet_json::RawJson;
use std::collections::HashMap;
use std::str::FromStr;

/// This is the ID for an ill-defined resource that is specifically the child of a resource group.
/// Some things are children of things that are children of resource groups, which this would not apply to.
/// At some point, this should be replaced with ScopeImpl or something in the fields where this type is used.
#[derive(Debug, Clone, Eq, PartialEq, Hash, facet::Facet)]
#[facet(opaque, json::proxy = String)]
pub struct ResourceId {
    pub resource_group_id: ResourceGroupId,
    pub resource_type: ResourceType,
    pub resource_name: CompactString,
}
crate::impl_facet_string_proxy!(ResourceId, value => value.expanded_form());
impl ResourceId {
    pub fn new(
        resource_group_id: ResourceGroupId,
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

#[derive(Debug, Clone, Eq, PartialEq, facet::Facet)]
#[facet(opaque, proxy = RawJson<'static>)]
pub struct ResourcePropertyValue {
    raw: RawJson<'static>,
    string: Option<String>,
}

impl ResourcePropertyValue {
    pub fn as_str(&self) -> Option<&str> {
        self.string.as_deref()
    }
}

impl From<RawJson<'static>> for ResourcePropertyValue {
    fn from(raw: RawJson<'static>) -> Self {
        let string = facet_json::from_str::<String>(raw.as_str()).ok();
        Self { raw, string }
    }
}

impl From<&ResourcePropertyValue> for RawJson<'static> {
    fn from(value: &ResourcePropertyValue) -> Self {
        value.raw.clone()
    }
}

#[derive(Debug, Eq, PartialEq, facet::Facet)]
pub struct Resource {
    pub id: ScopeImpl,
    #[facet(opaque, proxy = crate::ResourceTypeProxy)]
    pub kind: ResourceType,
    pub name: String,
    #[facet(default, opaque, proxy = crate::StringMapDefaultNullProxy)]
    pub tags: HashMap<String, String>,
    #[facet(
        default,
        opaque,
        proxy = crate::HashMapDefaultNullProxy<ResourcePropertyValue>
    )]
    pub properties: HashMap<String, ResourcePropertyValue>,
}
impl AsScope for Resource {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl Resource {
    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.properties.get("displayName")?.as_str()
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

#[cfg(test)]
mod test {
    use crate::ResourceGroupId;
    use crate::ResourceGroupName;
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
            ResourceGroupId::new(
                SubscriptionId::new(Uuid::nil()),
                ResourceGroupName::try_new("MY-RG")?,
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
            ResourceGroupId::new(
                SubscriptionId::new(Uuid::nil()),
                ResourceGroupName::try_new("MY-RG")?,
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
            resource
                .properties
                .get("provisioningState")
                .and_then(crate::ResourcePropertyValue::as_str),
            Some("Succeeded")
        );
        assert_eq!(
            resource
                .properties
                .get("ipAddress")
                .and_then(crate::ResourcePropertyValue::as_str),
            Some("203.0.113.10")
        );

        Ok(())
    }

    #[test]
    fn deserializes_null_properties_to_empty_map() -> eyre::Result<()> {
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
        assert!(resource.properties.is_empty());
        assert!(resource.tags.is_empty());

        Ok(())
    }
}
