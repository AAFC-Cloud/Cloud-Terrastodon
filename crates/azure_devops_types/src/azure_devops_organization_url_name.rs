use crate::AzureDevOpsOrganizationName;
use crate::AzureDevOpsProjectCollectionName;
use facet::Facet;

/// The different name vocabularies carried by an organization/collection URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Facet)]
#[repr(u8)]
pub(crate) enum AzureDevOpsOrganizationUrlName {
    Organization(AzureDevOpsOrganizationName),
    ServerCollection(AzureDevOpsProjectCollectionName),
}
