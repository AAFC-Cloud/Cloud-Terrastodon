use crate::Scope;
use crate::StorageAccountBlobContainerName;
use crate::StorageAccountId;
use arbitrary::Arbitrary;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// The ARM identity of a blob container within an account's default blob service.
///
/// See the `id` property in the [2025-06-01 container schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#listcontaineritem).
/// The account and container name retain their domain validation; replacing
/// either field with another valid value preserves this resource route.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary, Facet)]
#[facet(proxy = String)]
pub struct StorageAccountBlobContainerId {
    pub storage_account_id: StorageAccountId,
    pub container_name: StorageAccountBlobContainerName,
}

impl Display for StorageAccountBlobContainerId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}/blobServices/default/containers/{}",
            self.storage_account_id.expanded_form(),
            self.container_name
        )
    }
}

impl FromStr for StorageAccountBlobContainerId {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self> {
        let segments = value.split('/').collect::<Vec<_>>();
        ensure!(
            segments.len() == 13
                && segments[9].eq_ignore_ascii_case("blobServices")
                && segments[10].eq_ignore_ascii_case("default")
                && segments[11].eq_ignore_ascii_case("containers"),
            "expected an ARM blob container ID beneath a storage account's default blob service"
        );
        Ok(Self {
            storage_account_id: segments[..9]
                .join("/")
                .parse()
                .wrap_err("invalid storage account in blob container ID")?,
            container_name: segments[12]
                .parse()
                .wrap_err("invalid name in blob container ID")?,
        })
    }
}

crate::impl_facet_string_proxy!(StorageAccountBlobContainerId, value => value.to_string());

cloud_terrastodon_registry::register_thing!(StorageAccountBlobContainerId);
cloud_terrastodon_registry::register_arbitrary!(StorageAccountBlobContainerId);

#[cfg(test)]
mod tests {
    use super::StorageAccountBlobContainerId;

    #[test]
    fn resource_identity_rejects_other_services_and_nested_resources() -> eyre::Result<()> {
        let account = "/subscriptions/11111111-1111-1111-1111-111111111111/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount";
        let value = format!("{account}/blobServices/default/containers/synthetic-container");
        let id: StorageAccountBlobContainerId = value.parse()?;
        assert_eq!(id.to_string(), value);
        assert_eq!(id.container_name.to_string(), "synthetic-container");
        for suffix in [
            "/fileServices/default/shares/synthetic-container",
            "/blobServices/other/containers/synthetic-container",
            "/blobServices/default/containers/synthetic-container/child",
            "/blobServices/default/containers/",
        ] {
            assert!(
                format!("{account}{suffix}")
                    .parse::<StorageAccountBlobContainerId>()
                    .is_err()
            );
        }
        Ok(())
    }
}
