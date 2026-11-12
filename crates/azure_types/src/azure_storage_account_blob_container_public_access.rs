use arbitrary::Arbitrary;
use facet::Facet;

/// Public read access permitted by a container.
///
/// [Storage Resource Provider 2025-06-01 schema](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01#publicaccess).
/// Release builds preserve new service values; debug builds expose missing vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum AzureStorageAccountBlobContainerPublicAccess {
    Container,
    Blob,
    None,
    #[arbitrary(skip)]
    Unknown(String),
}

impl From<String> for AzureStorageAccountBlobContainerPublicAccess {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Container" => Self::Container,
            "Blob" => Self::Blob,
            "None" => Self::None,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Unknown {} value: {value:?}", std::any::type_name::<Self>());
                #[cfg(not(debug_assertions))]
                Self::Unknown(value)
            }
        }
    }
}

impl From<&AzureStorageAccountBlobContainerPublicAccess> for String {
    fn from(value: &AzureStorageAccountBlobContainerPublicAccess) -> Self {
        match value {
            AzureStorageAccountBlobContainerPublicAccess::Container => "Container",
            AzureStorageAccountBlobContainerPublicAccess::Blob => "Blob",
            AzureStorageAccountBlobContainerPublicAccess::None => "None",
            AzureStorageAccountBlobContainerPublicAccess::Unknown(value) => value,
        }
        .to_owned()
    }
}

cloud_terrastodon_registry::register_thing!(AzureStorageAccountBlobContainerPublicAccess);
cloud_terrastodon_registry::register_arbitrary!(AzureStorageAccountBlobContainerPublicAccess);
