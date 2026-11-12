use crate::AzureResourceGroupId;
use crate::AzureStorageAccountName;
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
use facet::Facet;
use std::str::FromStr;

pub const STORAGE_ACCOUNT_ID_PREFIX: &str = "/providers/Microsoft.Storage/storageAccounts/";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Arbitrary, Facet)]
#[facet(json::proxy = String)]
pub struct AzureStorageAccountId {
    pub resource_group_id: AzureResourceGroupId,
    pub storage_account_name: AzureStorageAccountName,
}
crate::impl_facet_string_proxy!(AzureStorageAccountId, value => value.expanded_form());
impl AzureStorageAccountId {
    pub fn new(
        resource_group_id: impl Into<AzureResourceGroupId>,
        storage_account_name: impl Into<AzureStorageAccountName>,
    ) -> AzureStorageAccountId {
        AzureStorageAccountId {
            resource_group_id: resource_group_id.into(),
            storage_account_name: storage_account_name.into(),
        }
    }

    pub fn try_new<R, N>(resource_group_id: R, storage_account_name: N) -> Result<Self>
    where
        R: TryInto<AzureResourceGroupId>,
        R::Error: Into<eyre::Error>,
        N: TryInto<AzureStorageAccountName>,
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
        Ok(AzureStorageAccountId {
            resource_group_id,
            storage_account_name,
        })
    }
}

impl HasSlug for AzureStorageAccountId {
    type Name = AzureStorageAccountName;

    fn name(&self) -> &Self::Name {
        &self.storage_account_name
    }
}
impl AsRef<AzureResourceGroupId> for AzureStorageAccountId {
    fn as_ref(&self) -> &AzureResourceGroupId {
        &self.resource_group_id
    }
}
impl AsRef<AzureStorageAccountName> for AzureStorageAccountId {
    fn as_ref(&self) -> &AzureStorageAccountName {
        &self.storage_account_name
    }
}

impl NameValidatable for AzureStorageAccountId {
    fn validate_name(name: &str) -> Result<()> {
        AzureStorageAccountName::try_new(name).map(|_| ())
    }
}
impl HasPrefix for AzureStorageAccountId {
    fn get_prefix() -> &'static str {
        STORAGE_ACCOUNT_ID_PREFIX
    }
}
impl TryFromResourceGroupScoped for AzureStorageAccountId {
    unsafe fn new_resource_group_scoped_unchecked(
        _expanded: &str,
        resource_group_id: AzureResourceGroupId,
        name: Self::Name,
    ) -> Self {
        AzureStorageAccountId {
            resource_group_id,
            storage_account_name: name,
        }
    }
}

impl FromStr for AzureStorageAccountId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        AzureStorageAccountId::try_from_expanded(s)
    }
}

impl Scope for AzureStorageAccountId {
    type Err = <Self as std::str::FromStr>::Err;
    fn try_from_expanded(expanded: &str) -> Result<Self> {
        AzureStorageAccountId::try_from_expanded_resource_group_scoped(expanded)
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

cloud_terrastodon_registry::register_thing!(AzureStorageAccountId);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountId);

#[cfg(test)]
mod test {
    use super::AzureStorageAccountId;
    use crate::AzureResourceGroupId;
    use crate::AzureStorageAccountName;
    use crate::SubscriptionId;
    use crate::scopes::Scope;
    use crate::slug::Slug;
    use arbitrary::Arbitrary;
    use arbitrary::Unstructured;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        assert!(AzureStorageAccountId::try_new("", "").is_err());
        AzureStorageAccountId::try_new(
            "/subscriptions/eefb00d7-c277-4c2c-a7de-ba3a11cf2110/resourceGroups/myRG",
            "aaa",
        )?;
        AzureStorageAccountId::try_new(
            AzureResourceGroupId::try_new("95c30970-3b9b-47d6-84a2-31f0e0cdfc8e", "myRG")?,
            "aaa",
        )?;
        AzureStorageAccountId::try_new(
            AzureResourceGroupId::try_new(
                SubscriptionId::try_new("d4917068-8792-4f47-9a6d-330f202cd438")?,
                "myRG",
            )?,
            "aaa",
        )?;
        AzureStorageAccountId::new(
            AzureResourceGroupId::try_new(
                "/subscriptions/ac9c7dce-2d4e-4bd2-865d-4a2de1ff5df4",
                "MyRG",
            )?,
            AzureStorageAccountName::try_new("aaa")?,
        );
        Ok(())
    }

    #[test]
    pub fn round_trip() -> eyre::Result<()> {
        for i in 0..100 {
            // construct arbitrary id
            let data = &[i; 16];
            let mut data = Unstructured::new(data);
            let id = AzureStorageAccountId::arbitrary(&mut data)?;
            let serialized = id.expanded_form();
            let deserialized: AzureStorageAccountId = serialized.parse()?;
            assert_eq!(id, deserialized);
            let json = facet_json::to_string(&serialized)?;
            assert_eq!(facet_json::to_string(&id)?, json);
            assert_eq!(facet_json::from_str::<AzureStorageAccountId>(&json)?, id);
        }
        Ok(())
    }
}
