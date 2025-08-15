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
use arbitrary::Arbitrary;
use eyre::Context;
use eyre::Result;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error;
use std::str::FromStr;

pub const STORAGE_ACCOUNT_ID_PREFIX: &str = "/providers/Microsoft.Storage/storageAccounts/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct StorageAccountId {
    pub resource_group_id: ResourceGroupId,
    pub storage_account_name: StorageAccountName,
}
impl StorageAccountId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
        storage_account_name: impl Into<StorageAccountName>,
    ) -> StorageAccountId {
        StorageAccountId {
            resource_group_id: resource_group_id.into(),
            storage_account_name: storage_account_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, storage_account_name: N) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<StorageAccountName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let storage_account_name = storage_account_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert storage_account_name")?;
        Ok(StorageAccountId {
            resource_group_id,
            storage_account_name,
        })
    }
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
            storage_account_name: name,
        }
    }
}

impl FromStr for StorageAccountId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        StorageAccountId::try_from_expanded(s)
    }
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
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
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

#[cfg(test)]
mod test {
    use super::StorageAccountId;
    use crate::prelude::ResourceGroupId;
    use crate::prelude::StorageAccountName;
    use crate::prelude::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(StorageAccountId::try_new("", "").is_err());
        StorageAccountId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "aaa",
        )?;
        StorageAccountId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "aaa",
        )?;
        StorageAccountId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "aaa",
        )?;
        StorageAccountId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            StorageAccountName::try_new("aaa")?,
        );
        Ok(())
    }

    #[test]
    pub fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            // construct arbitrary id
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = StorageAccountId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: StorageAccountId = serialized.parse()?;
            assert_eq!(id, deserialized);
        }
        Ok(())
    }
}
