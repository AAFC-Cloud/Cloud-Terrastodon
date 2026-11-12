use crate::ArbitraryJson;
use crate::AzureStorageAccountId;
use crate::AzureStorageAccountKind;
use crate::AzureStorageAccountName;
use crate::AzureStorageAccountSku;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use arbitrary::Arbitrary;
use cloud_terrastodon_hcl_types::AzureRmResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;
use facet::Facet;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
pub struct AzureStorageAccount {
    pub id: AzureStorageAccountId,
    pub name: AzureStorageAccountName,
    pub kind: AzureStorageAccountKind,
    pub location: String,
    pub sku: AzureStorageAccountSku,
    pub properties: ArbitraryJson,
    #[facet(default)]
    pub tags: HashMap<String, String>,
}

impl AsScope for AzureStorageAccount {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &AzureStorageAccount {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for AzureStorageAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}
impl From<AzureStorageAccount> for HclImportBlock {
    fn from(storage_account: AzureStorageAccount) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: storage_account.id.expanded_form().to_owned(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRmResourceBlockKind::StorageAccount,
                name: storage_account.name.sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::AzureResourceGroupId;
    use crate::AzureResourceGroupName;
    use crate::SubscriptionId;
    use crate::slug::Slug;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Storage/storageAccounts/bruh
        let expanded = AzureStorageAccountId {
            resource_group_id: AzureResourceGroupId::new(
                SubscriptionId::new(Uuid::new_v4()),
                AzureResourceGroupName::try_new("MY-RG")?,
            ),
            storage_account_name: AzureStorageAccountName::try_new("bruh")?,
        };
        let id: AzureStorageAccountId = facet_json::from_str(&facet_json::to_string(&expanded)?)?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn not_ambiguous() -> Result<()> {
        let expanded = format!(
            "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Storage/storageAccounts/bruh",
            nil = Uuid::nil()
        );
        let id = AzureStorageAccountId::try_from_expanded(&expanded)?;
        assert_eq!(id.expanded_form(), expanded);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccount);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccount);
