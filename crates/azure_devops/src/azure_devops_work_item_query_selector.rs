use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemQueryId;
use facet::Facet;

#[derive(Debug, Clone, Arbitrary, Facet)]
#[repr(u8)]
pub enum AzureDevOpsWorkItemQuerySelector {
    Id(AzureDevOpsWorkItemQueryId),
    Wiql(String),
}
