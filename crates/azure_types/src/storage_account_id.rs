use crate::prelude::ResourceGroupId;
use crate::prelude::StorageAccountName;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromResourceGroupScoped;
use crate::slug::HasSlug;
use crate::slug::Slug;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;

pub const STORAGE_ACCOUNT_ID_PREFIX: &str = "/providers/Microsoft.Storage/storageAccounts/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StorageAccountId {
    pub resource_group_id: ResourceGroupId,
    pub storage_account_name: StorageAccountName,
}
impl HasSlug for StorageAccountId {
    type Name = StorageAccountName;

    fn name(&self) -> &Self::Name {
        &self.storage_account_name
    }
}
impl AsRef<ResourceGroupId> for StorageAccountId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl AsRef<StorageAccountName> for StorageAccountId {
    fn as_ref(&self) -> &StorageAccountName {
        &self.storage_account_name
    }
}

impl NameValidatable for StorageAccountId {
    fn validate_name(name: &str) -> Result<()> {
        StorageAccountName::try_new(name).map(|_| ())
    }
}
impl HasPrefix for StorageAccountId {
    fn get_prefix() -> &'static str {
        STORAGE_ACCOUNT_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for StorageAccountId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        StorageAccountId {
            resource_group_id,
            storage_account_name: name.into(),
        }
    }
    // unsafe fn new_resource_group_scoped_unchecked(expanded: &str) -> Self {
    //     StorageAccountId::ResourceGroupScoped {
    //         expanded: expanded.to_string(),
    //     }
    // }
}

impl Scope for StorageAccountId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        StorageAccountId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            STORAGE_ACCOUNT_ID_PREFIX,
            self.storage_account_name
        )
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
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for StorageAccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = StorageAccountId::try_from_expanded(expanded.as_str())
            .map_err(|e| D::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}
