use crate::prelude::ResourceGroupId;
use crate::prelude::RouteTableName;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromResourceGroupScoped;
use crate::slug::HasSlug;
use crate::slug::Slug;
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::str::FromStr;
pub const ROUTE_TABLE_ID_PREFIX: &str = "/providers/Microsoft.Network/routeTables/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Arbitrary)]
pub struct RouteTableId {
    pub resource_group_id: ResourceGroupId,
    pub route_table_name: RouteTableName,
}

impl RouteTableId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        route_table_name: impl Into<RouteTableName>,
    ) -> Self {
        Self {
            resource_group_id: resource_group_id.into(),
            route_table_name: route_table_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, route_table_name: N) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<RouteTableName>,
        N::Error: Into<eyre::Error>,
    {
        Ok(Self {
            resource_group_id: resource_group_id
                .try_into()
                .map_err(Into::into)
                .wrap_err("Failed to convert resource_group_id")?,
            route_table_name: route_table_name
                .try_into()
                .map_err(Into::into)
                .context("Failed to convert to RouteTableName")?,
        })
    }
}

impl HasSlug for RouteTableId {
    type Name = RouteTableName;

    fn name(&self) -> &Self::Name {
        &self.route_table_name
    }
}

impl AsRef<ResourceGroupId> for RouteTableId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}

impl AsRef<RouteTableName> for RouteTableId {
    fn as_ref(&self) -> &RouteTableName {
        &self.route_table_name
    }
}

impl NameValidatable for RouteTableId {
    fn validate_name(name: &str) -> Result<()> {
        RouteTableName::try_new(name).map(|_| ())
    }
}

impl HasPrefix for RouteTableId {
    fn get_prefix() -> &'static str {
        ROUTE_TABLE_ID_PREFIX
    }
}

impl TryFromResourceGroupScoped for RouteTableId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        Self {
            resource_group_id,
            route_table_name: name,
        }
    }
}

impl Scope for RouteTableId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        RouteTableId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            ROUTE_TABLE_ID_PREFIX,
            self.route_table_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::RouteTable
    }

    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        crate::scopes::ScopeImpl::RouteTable(self.clone())
    }
}

impl FromStr for RouteTableId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        RouteTableId::try_from_expanded(s)
    }
}

impl Serialize for RouteTableId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.expanded_form().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RouteTableId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::try_from_expanded(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::SubscriptionId;
    use std::str::FromStr;

    #[test]
    fn test_route_table_id_serialization_deserialization() -> eyre::Result<()> {
        let sub_id = SubscriptionId::from_str("00000000-0000-0000-0000-000000000000")?;
        let rg_id = ResourceGroupId::new(sub_id, ResourceGroupName::try_new("test-rg").unwrap());
        let rt_id = RouteTableId::try_new(rg_id, "test-route-table")?;

        let serialized = serde_json::to_string(&rt_id)?;
        let expected_str = "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/test-rg/providers/Microsoft.Network/routeTables/test-route-table".to_string();
        assert_eq!(serialized, serde_json::to_string(&expected_str)?);

        let deserialized: RouteTableId = serde_json::from_str(&serialized)?;
        assert_eq!(rt_id, deserialized);

        Ok(())
    }

    #[test]
    fn test_scope_roundtrip() -> eyre::Result<()> {
        let sub_id = SubscriptionId::from_str("11111111-1111-1111-1111-111111111111")?;
        let rg_id = ResourceGroupId::new(
            sub_id,
            ResourceGroupName::try_new("myResourceGroup").unwrap(),
        );
        let original_id = RouteTableId::try_new(rg_id, "myRouteTable")?;
        let expanded = original_id.expanded_form();
        let parsed_id = RouteTableId::try_from_expanded(&expanded)?;
        assert_eq!(original_id, parsed_id);
        Ok(())
    }

    #[test]
    fn test_name_validation() -> eyre::Result<()> {
        assert!(RouteTableId::validate_name("valid-route-table-name").is_ok());
        assert!(RouteTableId::validate_name("").is_err()); // Empty
        Ok(())
    }
}
