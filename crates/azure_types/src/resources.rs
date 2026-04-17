use crate::AsScope;
use crate::ResourceGroupId;
use crate::ResourceType;
use crate::Scope;
use crate::ScopeImpl;
use crate::ScopeImplKind;
use crate::scopes::HasPrefix;
use crate::scopes::TryFromResourceScoped;
use crate::serde_helpers::deserialize_default_if_null;
use crate::slug::HasSlug;
use compact_str::CompactString;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

/// This is the ID for an ill-defined resource that is specifically the child of a resource group.
/// Some things are children of things that are children of resource groups, which this would not apply to.
/// At some point, this should be replaced with ScopeImpl or something in the fields where this type is used.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResourceId {
    pub resource_group_id: ResourceGroupId,
    pub resource_type: ResourceType,
    pub resource_name: CompactString,
}
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

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Resource {
    pub id: ScopeImpl,
    pub kind: ResourceType,
    pub name: String,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub properties: HashMap<String, Value>,
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
    use serde_json::json;
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
    fn deserializes_resource_properties() -> eyre::Result<()> {
        let resource: crate::Resource = serde_json::from_value(json!({
            "id": format!(
                "/subscriptions/{nil}/resourceGroups/my-rg/providers/Microsoft.Network/publicIPAddresses/my-ip",
                nil = Uuid::nil()
            ),
            "kind": "Microsoft.Network/publicIPAddresses",
            "name": "my-ip",
            "properties": {
                "displayName": "My IP",
                "provisioningState": "Succeeded",
                "ipAddress": "203.0.113.10"
            },
            "tags": {
                "env": "test"
            }
        }))?;

        assert_eq!(resource.display_name(), Some("My IP"));
        assert_eq!(
            resource.properties.get("provisioningState"),
            Some(&json!("Succeeded"))
        );
        assert_eq!(
            resource.properties.get("ipAddress"),
            Some(&json!("203.0.113.10"))
        );

        Ok(())
    }

    #[test]
    fn deserializes_null_properties_to_empty_map() -> eyre::Result<()> {
        let resource: crate::Resource = serde_json::from_value(json!({
            "id": format!(
                "/subscriptions/{nil}/resourceGroups/my-rg/providers/Contoso.Widgets/widgets/my-widget",
                nil = Uuid::nil()
            ),
            "kind": "Contoso.Widgets/widgets",
            "name": "my-widget",
            "properties": null,
            "tags": null
        }))?;

        assert_eq!(resource.display_name(), None);
        assert!(resource.properties.is_empty());
        assert!(resource.tags.is_empty());

        Ok(())
    }
}
