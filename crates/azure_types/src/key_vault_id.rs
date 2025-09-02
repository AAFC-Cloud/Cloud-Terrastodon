use crate::prelude::ResourceGroupId;
use crate::prelude::KeyVaultName;
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
use std::str::FromStr;

pub const KEY_VAULT_ID_PREFIX: &str = "/providers/Microsoft.KeyVault/vaults/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub struct KeyVaultId {
    pub resource_group_id: ResourceGroupId,
    pub key_vault_name: KeyVaultName,
}
impl KeyVaultId {
    pub fn new(
        resource_group_id: impl Into<ResourceGroupId>,
    key_vault_name: impl Into<KeyVaultName>,
    ) -> KeyVaultId {
        KeyVaultId {
            resource_group_id: resource_group_id.into(),
            key_vault_name: key_vault_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, key_vault_name: N) -> Result<Self>
    where
        R: TryInto<ResourceGroupId>,
        R::Error: Into<eyre::Error>,
    N: TryInto<KeyVaultName>,
        N::Error: Into<eyre::Error>,
    {
        let resource_group_id = resource_group_id
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert resource_group_id")?;
        let key_vault_name = key_vault_name
            .try_into()
            .map_err(Into::into)
            .wrap_err("Failed to convert key_vault_name")?;
        Ok(KeyVaultId {
            resource_group_id,
            key_vault_name,
        })
    }
}

impl HasSlug for KeyVaultId {
    type Name = KeyVaultName;

    fn name(&self) -> &Self::Name {
        &self.key_vault_name
    }
}
impl AsRef<ResourceGroupId> for KeyVaultId {
    fn as_ref(&self) -> &ResourceGroupId {
        &self.resource_group_id
    }
}
impl AsRef<KeyVaultName> for KeyVaultId {
    fn as_ref(&self) -> &KeyVaultName {
        &self.key_vault_name
    }
}

impl NameValidatable for KeyVaultId {
    fn validate_name(name: &str) -> Result<()> {
        KeyVaultName::try_new(name).map(|_| ())
    }
}
impl HasPrefix for KeyVaultId {
    fn get_prefix() -> &'static str {
        KEY_VAULT_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for KeyVaultId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: ResourceGroupId,
        name: Self::Name,
    ) -> Self {
        KeyVaultId {
            resource_group_id,
            key_vault_name: name,
        }
    }
}

impl FromStr for KeyVaultId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        KeyVaultId::try_from_expanded(s)
    }
}

impl Scope for KeyVaultId {
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        KeyVaultId::try_from_expanded_resource_group_scoped(expanded)
    }

    fn expanded_form(&self) -> String {
        format!(
            "{}{}{}",
            self.resource_group_id.expanded_form(),
            KEY_VAULT_ID_PREFIX,
            self.key_vault_name
        )
    }

    fn kind(&self) -> ScopeImplKind {
        ScopeImplKind::KeyVault
    }
    fn as_scope_impl(&self) -> crate::scopes::ScopeImpl {
        ScopeImpl::KeyVault(self.clone())
    }
}

impl Serialize for KeyVaultId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.expanded_form())
    }
}

impl<'de> Deserialize<'de> for KeyVaultId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expanded = String::deserialize(deserializer)?;
        let id = KeyVaultId::try_from_expanded(expanded.as_str())
            .map_err(|e| serde::de::Error::custom(format!("{e:?}")))?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::KeyVaultId;
    use crate::prelude::KeyVaultName;
    use crate::prelude::ResourceGroupId;
    use crate::prelude::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(KeyVaultId::try_new("", "").is_err());
        KeyVaultId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "aaa",
        )?;
        KeyVaultId::try_new(
            ResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "aaa",
        )?;
        KeyVaultId::try_new(
            ResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "aaa",
        )?;
        KeyVaultId::new(
            ResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            KeyVaultName::try_new("abc")?,
        );
        Ok(())
    }

    #[test]
    pub fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            // construct arbitrary id
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = KeyVaultId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: KeyVaultId = serialized.parse()?;
            assert_eq!(id, deserialized);
        }
        Ok(())
    }
}
