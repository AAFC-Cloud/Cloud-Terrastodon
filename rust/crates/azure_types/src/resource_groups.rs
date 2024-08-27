use crate::naming::validate_resource_group_name;
use crate::prelude::strip_prefix_get_slug_and_leading_slashed_remains;
use crate::prelude::SubscriptionScoped;
use crate::prelude::SUBSCRIPTION_ID_PREFIX;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromSubscriptionScoped;
use crate::subscriptions::SubscriptionId;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use cloud_terrasotodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrasotodon_core_tofu_types::prelude::TofuAzureRMResourceKind;
use cloud_terrasotodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrasotodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrasotodon_core_tofu_types::prelude::TofuResourceReference;
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;
use uuid::Uuid;

pub const RESOURCE_GROUP_ID_PREFIX: &str = "/resourceGroups/";
#[derive(Debug, Clone)]
pub struct ResourceGroupId {
    expanded: String,
}
impl ResourceGroupId {
    pub fn new(subscription_id: &SubscriptionId, resource_group_name: String) -> ResourceGroupId {
        let expanded = format!(
            "{}{}{}",
            subscription_id.expanded_form(),
            RESOURCE_GROUP_ID_PREFIX,
            resource_group_name
        );
        ResourceGroupId { expanded }
    }
}
impl SubscriptionScoped for ResourceGroupId {}

impl PartialEq for ResourceGroupId {
    fn eq(&self, other: &Self) -> bool {
        self.expanded.to_lowercase() == other.expanded.to_lowercase()
    }
}

impl Eq for ResourceGroupId {}

impl Hash for ResourceGroupId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.expanded.to_lowercase().hash(state);
    }
}
impl std::fmt::Display for ResourceGroupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.expanded.as_str())
    }
}

impl NameValidatable for ResourceGroupId {
    fn validate_name(name: &str) -> Result<()> {
        validate_resource_group_name(name)
    }
}
impl HasPrefix for ResourceGroupId {
    fn get_prefix() -> &'static str {
        RESOURCE_GROUP_ID_PREFIX
    }
}
impl TryFromSubscriptionScoped for ResourceGroupId {
    unsafe fn new_subscription_scoped_unchecked(expanded: &str) -> Self {
        ResourceGroupId {
            expanded: expanded.to_owned(),
        }
    }
}

impl FromStr for ResourceGroupId {
    type Err = anyhow::Error;

    fn from_str(expanded_form: &str) -> Result<Self, Self::Err> {
        // /subscriptions/0000-000-00000/resourceGroups/My-Resource-Group/optional/other/stuff

        // "0000-000-0000", maybe "/resourceGroups/My-Resource-Group/optional/other/stuff"
        let (sub_id, remaining) = strip_prefix_get_slug_and_leading_slashed_remains(
            expanded_form,
            SUBSCRIPTION_ID_PREFIX,
        )
        .context(format!(
            "Tried to parse {:?} as resource group id, but prefix {} was missing",
            expanded_form, SUBSCRIPTION_ID_PREFIX
        ))?;

        // "0000-000-0000" => Uuid{0000-000-0000}
        let sub_id = sub_id.parse::<Uuid>().context(format!("Tried to parse {:?} as a resource group id, but the subscription id {:?} isn't a valid guid", expanded_form, sub_id))?;
        let sub_id = SubscriptionId::new(sub_id);

        // maybe "/resourceGroups/..." => Some("/resourceGroups/...")
        let Some(remaining) = remaining else {
            bail!("Tried to parse {:?} as resource group id, but the stuff after the subscription id was missing", expanded_form)
        };

        // "My-Resource-Group", "optional/other/stuff"
        let (rg_name, _remaining) =
            strip_prefix_get_slug_and_leading_slashed_remains(remaining, RESOURCE_GROUP_ID_PREFIX)
                .context(format!(
            "Tried to parse {:?} as a resource group id, but chunk {:?} was missing prefix {:?}",
            expanded_form, remaining, RESOURCE_GROUP_ID_PREFIX
        ))?;

        Ok(ResourceGroupId::new(&sub_id, rg_name.to_owned()))
    }
}

impl Scope for ResourceGroupId {
    fn expanded_form(&self) -> &str {
        &self.expanded
    }

    fn try_from_expanded(expanded: &str) -> Result<Self> {
        ResourceGroupId::try_from_expanded_subscription_scoped(expanded)
    }

    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::ResourceGroup(self.clone())
    }
    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::ResourceGroup
    }
}

impl Serialize for ResourceGroupId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ResourceGroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = expanded
            .parse()
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ResourceGroup {
    pub id: ResourceGroupId,
    pub subscription_id: SubscriptionId,
    pub location: String,
    pub managed_by: Option<String>,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub tags: Option<HashMap<String, String>>,
}

impl HasScope for ResourceGroup {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &ResourceGroup {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for ResourceGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
impl From<ResourceGroup> for TofuImportBlock {
    fn from(resource_group: ResourceGroup) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: resource_group.id.to_string(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::ResourceGroup,
                name: format!(
                    "{}__{}",
                    resource_group.name,
                    resource_group.id.subscription_id().short_form()
                )
                .sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let id = ResourceGroupId::new(&SubscriptionId::new(Uuid::nil()), "bruh".to_string());
        let expanded = id.expanded_form();
        let x: ResourceGroupId = serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(x, id);
        assert_eq!(x.expanded_form(), expanded);
        assert_eq!(x.to_string(), expanded);

        Ok(())
    }
}
