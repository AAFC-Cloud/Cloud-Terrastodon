use arbitrary::Arbitrary;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use facet::Facet;
use std::fmt::Display;
use std::fmt::Formatter;

/// Why the audit could not determine whether anonymous blob access is unused.
#[derive(Debug, PartialEq, Eq, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureStorageAccountAuditInconclusiveReason {
    MissingAccountSetting,
    ContainerInventoryFailed {
        error: String,
    },
    ForeignContainer {
        container_id: AzureStorageAccountBlobContainerId,
    },
    DeletedContainer {
        container_id: AzureStorageAccountBlobContainerId,
    },
    UnknownContainerAccess {
        container_id: AzureStorageAccountBlobContainerId,
        value: String,
    },
}

impl Display for AzureStorageAccountAuditInconclusiveReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAccountSetting => {
                f.write_str("Azure did not report the account's allowBlobPublicAccess setting")
            }
            Self::ContainerInventoryFailed { error } => {
                write!(f, "Could not enumerate every blob container: {error}")
            }
            Self::ForeignContainer { container_id } => {
                write!(
                    f,
                    "Container inventory included a different account's container: {container_id}"
                )
            }
            Self::DeletedContainer { container_id } => {
                write!(
                    f,
                    "Live container inventory included a soft-deleted container: {container_id}"
                )
            }
            Self::UnknownContainerAccess {
                container_id,
                value,
            } => {
                write!(
                    f,
                    "Container {} has an unrecognized publicAccess setting: {value:?}",
                    container_id.container_name
                )
            }
        }
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountAuditInconclusiveReason);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountAuditInconclusiveReason);
