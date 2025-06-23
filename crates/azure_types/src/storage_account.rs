use crate::prelude::StorageAccountId;
use crate::prelude::StorageAccountName;
use crate::prelude::SubscriptionId;
use crate::scopes::AsScope;
use crate::scopes::Scope;
use cloud_terrastodon_hcl_types::prelude::AzureRMResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

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
    #[serde(deserialize_with = "crate::serde_helpers::deserialize_null_default")]
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl AsScope for StorageAccount {
    fn as_scope(&self) -> &impl Scope {
        &self.id
    }
}
impl AsScope for &StorageAccount {
    fn as_scope(&self) -> &impl Scope {
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
    use super::*;
    use crate::prelude::ResourceGroupId;
    use crate::prelude::ResourceGroupName;
    use crate::slug::Slug;
    use eyre::Result;
    use uuid::Uuid;

    #[test]
    fn deserializes() -> Result<()> {
        // /subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Storage/storageAccounts/bruh
        let expanded = StorageAccountId {
            resource_group_id: ResourceGroupId::new(
                SubscriptionId::new(Uuid::new_v4()),
                ResourceGroupName::try_new("MY-RG")?,
            ),
            storage_account_name: StorageAccountName::try_new("bruh")?,
        };
        let id: StorageAccountId =
            serde_json::from_str(serde_json::to_string(&expanded)?.as_str())?;
        assert_eq!(id, expanded);
        Ok(())
    }

    #[test]
    fn not_ambiguous() -> Result<()> {
        let expanded = format!(
            "/subscriptions/{nil}/resourceGroups/MY-RG/providers/Microsoft.Storage/storageAccounts/bruh",
            nil = Uuid::nil()
        );
        let id = StorageAccountId::try_from_expanded(&expanded)?;
        dbg!(id);
        Ok(())
    }
}
