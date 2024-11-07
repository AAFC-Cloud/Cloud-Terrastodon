use anyhow::Result;
use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureADResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::impl_uuid_traits;
use crate::prelude::UuidWrapper;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct GroupId(pub Uuid);
impl UuidWrapper for GroupId {
    fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
impl_uuid_traits!(GroupId);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Group {
    description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub id: GroupId,
    #[serde(rename = "isAssignableToRole")]
    pub is_assignable_to_role: Option<bool>,
    #[serde(rename = "securityEnabled")]
    pub security_enabled: bool,
}
impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_name)?;
        f.write_str(" (")?;
        f.write_str(self.id.to_string().as_str())?;
        f.write_str(")")?;
        Ok(())
    }
}
impl From<Group> for TofuImportBlock {
    fn from(group: Group) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: format!("/groups/{}", group.id.0.as_hyphenated()),
            to: TofuResourceReference::AzureAD {
                kind: TofuAzureADResourceKind::Group,
                name: format!("{}__{}", group.display_name, group.id).sanitize(),
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
        let expanded = "55555555-5555-5555-5555-555555555555";
        let id: GroupId = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
