use arbitrary::Arbitrary;
use facet::Facet;

/// Progress of an account's point-in-time blob restore.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/storage-accounts/get-properties?view=rest-storagerp-2025-06-01#blobrestoreprogressstatus).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobRestoreProgressStatus {
    InProgress,
    Complete,
    Failed,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobRestoreProgressStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "InProgress" => Self::InProgress,
            "Complete" => Self::Complete,
            "Failed" => Self::Failed,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountBlobRestoreProgressStatus> for String {
    fn from(value: &AzureStorageAccountBlobRestoreProgressStatus) -> Self {
        match value {
            AzureStorageAccountBlobRestoreProgressStatus::InProgress => "InProgress",
            AzureStorageAccountBlobRestoreProgressStatus::Complete => "Complete",
            AzureStorageAccountBlobRestoreProgressStatus::Failed => "Failed",
            AzureStorageAccountBlobRestoreProgressStatus::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobRestoreProgressStatus);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobRestoreProgressStatus);
