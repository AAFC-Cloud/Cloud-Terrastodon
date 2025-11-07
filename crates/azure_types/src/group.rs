use crate::prelude::GroupId;
use cloud_terrastodon_hcl_types::prelude::AzureAdResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
impl From<Group> for HclImportBlock {
    fn from(group: Group) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: format!("/groups/{}", group.id.0.as_hyphenated()),
            to: ResourceBlockReference::AzureAD {
                kind: AzureAdResourceBlockKind::Group,
                name: format!("{}__{}", group.display_name, group.id).sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use eyre::Result;

    #[test]
    fn deserializes() -> Result<()> {
        let expanded = "55555555-5555-5555-5555-555555555555";
        let id: GroupId = serde_json::from_str(serde_json::to_string(expanded)?.as_str())?;
        assert_eq!(id.to_string(), expanded);

        Ok(())
    }
}
