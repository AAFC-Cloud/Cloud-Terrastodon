use crate::prelude::SubscriptionId;
use crate::prelude::validate_storage_account_name;
use crate::scopes::HasPrefix;
use crate::scopes::HasScope;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromResourceGroupScoped;
use cloud_terrastodon_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_tofu_types::prelude::TofuAzureRMResourceKind;
use cloud_terrastodon_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_tofu_types::prelude::TofuResourceReference;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use serde_json::Value;

pub const STORAGE_ACCOUNT_ID_PREFIX: &str = "/providers/Microsoft.Storage/storageAccounts/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StorageAccountId {
    ResourceGroupScoped { expanded: String },
}

impl NameValidatable for StorageAccountId {
    fn validate_name(name: &str) -> Result<()> {
        validate_storage_account_name(name)
    }
}
impl HasPrefix for StorageAccountId {
    fn get_prefix() -> &'static str {
        STORAGE_ACCOUNT_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for StorageAccountId {
    unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
        StorageAccountId::ResourceGroupScoped {
            expanded: expanded.to_string(),
        }
    }
}

impl Scope for StorageAccountId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        StorageAccountId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> &str {
        match self {
            Self::ResourceGroupScoped { expanded } => expanded,
        }
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::StorageAccount
    }
    fn as_scope(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::StorageAccount(self.clone())
    }
}

impl Serialize for StorageAccountId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for StorageAccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = StorageAccountId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:#}")))?;
        Ok(id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageAccountSKU {
    name: String,
    tier: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StorageAccount {
    pub id: StorageAccountId,
    pub name: String,
    pub kind: String,
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
impl From<StorageAccount> for TofuImportBlock {
    fn from(storage_account: StorageAccount) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: storage_account.id.expanded_form().to_owned(),
            to: TofuResourceReference::AzureRM {
                kind: TofuAzureRMResourceKind::StorageAccount,
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
