use crate::prelude::StorageAccountId;
use crate::prelude::StorageAccountName;
use crate::prelude::SubscriptionId;
use crate::scopes::HasScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageAccountSKU {
    name: String,
    tier: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum StorageAccountKind {
    BlockBlobStorage,
    BlobStorage,
    Storage,
    StorageV2,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageAccount {
    pub id: StorageAccountId,
    pub name: StorageAccountName,
    pub kind: StorageAccountKind,
    pub location: String,
    #[serde(rename = "resourceGroup")]
    pub resource_group: String,
    #[serde(rename = "subscriptionId")]
    pub subscription_id: SubscriptionId,
    pub sku: StorageAccountSKU,
    pub properties: Value,
}

impl HasScope for StorageAccount {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}
impl HasScope for &StorageAccount {
    fn scope(&self) -> &impl Scope {
        &self.id
    }
}

impl std::fmt::Display for StorageAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)?;
        Ok(())
    }
}
impl From<StorageAccount> for HCLImportBlock {
    fn from(storage_account: StorageAccount) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: storage_account.id.expanded_form().to_owned(),
            to: ResourceBlockReference::AzureRM {
                kind: AzureRMResourceBlockKind::StorageAccount,
                name: storage_account.name.sanitize(),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::prelude::ResourceGroupId;

    use super::*;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        let nil = Uuid::nil();
        let expanded = StorageAccountId::ResourceGroupScoped {
            expanded: format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Storage/storageAccounts/bruh",
            ),
        };
        let id: StorageAccountId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn not_ambiguous() -> Result<()> {
        let nil = Uuid::nil();
        let expanded = StorageAccountId::ResourceGroupScoped {
            expanded: format!(
                "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Storage/storageAccounts/bruh",
            ),
        };
        assert!(expanded.expanded_form().parse::<ResourceGroupId>().is_err());
        Ok(())
    }
}
