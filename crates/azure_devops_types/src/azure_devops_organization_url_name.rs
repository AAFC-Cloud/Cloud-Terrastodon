use crate::AzureDevOpsOrganizationName;
use crate::AzureDevOpsProjectCollectionName;

/// The different name vocabularies carried by an organization/collection URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, facet::Facet)]
#[repr(u8)]
pub(crate) enum AzureDevOpsOrganizationUrlName {
    Organization(AzureDevOpsOrganizationName),
    ServerCollection(AzureDevOpsProjectCollectionName),
}
