use crate::prelude::validate_storage_account_name;
use crate::prelude::ResourceGroupId;
use crate::prelude::StorageAccountName;
use crate::scopes::HasPrefix;
use crate::scopes::NameValidatable;
use crate::scopes::Scope;
use crate::scopes::ScopeImpl;
use crate::scopes::ScopeImplKind;
use crate::scopes::TryFromResourceGroupScoped;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;

pub const STORAGE_ACCOUNT_ID_PREFIX: &str = "/providers/Microsoft.Storage/storageAccounts/";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StorageAccountId2 {
    pub resource_group_id: ResourceGroupId,
    pub storage_account_name: StorageAccountName,
}

// impl FromStr for StorageAccountName {
//     type Err = eyre::Error;

//     fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
//         let naming_restriction = NamingRestriction {
//             rules: vec![
//                 StringRule::LowercaseLettersNumbersAndHyphens,
//                 StringRule::Length(3..=24),
//             ],
//         };
//         todo!()
//     }
// }

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