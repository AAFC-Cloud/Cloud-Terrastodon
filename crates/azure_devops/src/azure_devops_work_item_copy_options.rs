use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_devops_types::AzureDevOpsWorkItemId;
use facet::Facet;

#[derive(Debug, Clone, arbitrary::Arbitrary, Default, Facet)]
pub struct AzureDevOpsWorkItemCopyOptions {
    pub deep: bool,
    /// An empty set permits discovery of descendants in this organization.
    pub allowed_ids: Vec<AzureDevOpsWorkItemId>,
    /// Optional replacement title for the root only.
    pub title: Option<String>,
    pub keep_parent: bool,
    pub keep_external_relations: bool,
    pub as_of: Option<DateTime<Utc>>,
}
