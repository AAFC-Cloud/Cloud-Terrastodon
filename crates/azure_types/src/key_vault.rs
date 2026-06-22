use crate::KeyVaultId;
use crate::KeyVaultName;
use crate::KeyVaultProperties;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use std::collections::HashMap;

#[derive(Debug, PartialEq, facet::Facet)]
pub struct KeyVault {
    pub id: KeyVaultId,
    pub name: KeyVaultName,
    pub location: String,
    pub properties: KeyVaultProperties,
    #[facet(default)]
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
impl From<KeyVault> for HclImportBlock {
    fn from(storage_account: KeyVault) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: storage_account.id.expanded_form().to_owned(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::KeyVault,
                name: storage_account.name.sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceGroupId;
    use crate::ResourceGroupName;
    use crate::SubscriptionId;
    use crate::slug::Slug;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.KeyVault/vaults/bruh
        let expanded = KeyVaultId {
            resource_group_id: ResourceGroupId::new(
                SubscriptionId::new(Uuid::new_v4()),
                ResourceGroupName::try_new("MY-RG")?,
            ),
            key_vault_name: KeyVaultName::try_new("bruh")?,
        };
        let id: KeyVaultId = facet_json::from_str(&facet_json::to_string(&expanded)?)?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn not_ambiguous() -> Result<()> {
        let expanded = format!(
            "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.KeyVault/vaults/bruh",
            nil = Uuid::nil()
        );
        let id = KeyVaultId::try_from_expanded(&expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        Ok(())
    }
}
