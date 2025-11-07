use crate::prelude::ResourceGroupId;
use crate::prelude::ResourceGroupName;
use crate::prelude::SubscriptionId;
use crate::prelude::SubscriptionScoped;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::serde_helpers::deserialize_default_if_null;
use cloud_terrastodon_hcl_types::prelude::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ResourceGroup {
    pub id: ResourceGroupId,
    pub subscription_id: SubscriptionId,
    pub location: String,
    pub managed_by: Option<String>,
    pub name: ResourceGroupName,
    pub properties: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl AsScope for ResourceGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &ResourceGroup {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for ResourceGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}
impl From<ResourceGroup> for HclImportBlock {
    fn from(resource_group: ResourceGroup) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: resource_group.id.to_string(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::ResourceGroup,
                name: format!(
                    "{}__{}",
                    resource_group.name,
                    resource_group.id.subscription_id().as_hyphenated()
                )
                .sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ResourceGroupName;
    use crate::slug::Slug;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let id = ResourceGroupId::new(
            SubscriptionId::new(Uuid::nil()),
            ResourceGroupName::try_new("bruh")?,
        );
        let expanded = id.expanded_form();
        let x: ResourceGroupId = serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(x, id);
        assert_eq!(x.expanded_form(), expanded);
        assert_eq!(x.to_string(), expanded);

        Ok(())
    }
}
