use crate::prelude::AsScope;
use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceType;
use crate::prelude::Scope;
use crate::prelude::ScopeImpl;
use crate::prelude::ScopeImplKind;
use crate::scopes::HasPrefix;
use crate::scopes::TryFromResourceScoped;
use crate::slug::HasSlug;
use compact_str::CompactString;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
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
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Resource {
    pub id: ScopeImpl,
    pub kind: ResourceType,
    pub name: String,
    pub display_name: Option<String>,
    pub tags: Option<HashMap<String, String>>,
}
impl AsScope for Resource {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "display_name={:64?}\tname={:64?}\tid={}",
            self.display_name, self.name, self.id
        ))
    }
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_azure_resource_types::ResourceType;
    use uuid::Uuid;

    use crate::prelude::ResourceGroupId;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::ResourceId;
    use crate::prelude::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;

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
        println!("{:?}", id.expanded_form());
        assert_eq!(
            id.expanded_form(),
            format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Network/virtualNetworks/MY-VNET",
                nil = Uuid::nil()
            )
        );
        Ok(())
    }
}
