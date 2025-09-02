use crate::prelude::KeyVaultId;
use crate::prelude::KeyVaultName;
use crate::prelude::KeyVaultProperties;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use crate::serde_helpers::deserialize_default_if_null;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct KeyVault {
    pub id: KeyVaultId,
    pub name: KeyVaultName,
    pub location: String,
    pub properties: KeyVaultProperties,
    #[serde(deserialize_with = "deserialize_default_if_null")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl AsScope for KeyVault {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &KeyVault {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for KeyVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}
impl From<KeyVault> for HCLImportBlock {
    fn from(storage_account: KeyVault) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: storage_account.id.expanded_form().to_owned(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::KeyVault,
                name: storage_account.name.sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::ResourceGroupId;
    use crate::prelude::ResourceGroupName;
    use crate::prelude::SubscriptionId;
    use crate::slug::Slug;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.KeyVault/Vaults/bruh
        let expanded = KeyVaultId {
            resource_group_id: ResourceGroupId::new(
                SubscriptionId::new(Uuid::new_v4()),
                ResourceGroupName::try_new("MY-RG")?,
            ),
            key_vault_name: KeyVaultName::try_new("bruh")?,
        };
        let id: KeyVaultId = serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn not_ambiguous() -> Result<()> {
        let expanded = format!(
            "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.KeyVault/Vaults/bruh",
            nil = Uuid::nil()
        );
        let id = KeyVaultId::try_from_expanded(&expanded)?;
        dbg!(id);
        Ok(())
    }
}
