use crate::AzureDevOpsAgentPoolQueue;
use crate::AzureDevOpsBuildDefinitionId;
use crate::AzureDevOpsBuildDefinitionName;
use crate::AzureDevOpsBuildDefinitionQuality;
use crate::AzureDevOpsBuildDefinitionQueueStatus;
use crate::AzureDevOpsBuildDefinitionSummary;
use crate::AzureDevOpsBuildDefinitionType;
use crate::AzureDevOpsBuildDefinitionUri;
use crate::AzureDevOpsBuildDefinitionUrl;
use crate::AzureDevOpsBuildFolderPath;
use crate::AzureDevOpsIdentityReference;
use crate::AzureDevOpsProjectReference;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;

/// A definition reference returned by the build definitions list endpoint.
///
/// The folder path is required so folder pruning never treats an unknown path
/// as the root folder. Builds may embed a shallower definition summary instead.
///
/// Microsoft documentation: [BuildDefinitionReference](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/list?view=azure-devops-rest-7.1#builddefinitionreference).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildDefinitionReference {
    pub id: AzureDevOpsBuildDefinitionId,
    pub name: AzureDevOpsBuildDefinitionName,
    pub path: AzureDevOpsBuildFolderPath,
    pub uri: AzureDevOpsBuildDefinitionUri,
    pub created_date: DateTime<Utc>,
    pub revision: Option<i32>,
    pub quality: Option<AzureDevOpsBuildDefinitionQuality>,
    pub authored_by: Option<AzureDevOpsIdentityReference>,
    pub drafts: Option<Vec<AzureDevOpsBuildDefinitionSummary>>,
    pub draft_of: Option<AzureDevOpsBuildDefinitionSummary>,
    /// Default agent queue metadata, when included in the shallow reference.
    pub queue: Option<AzureDevOpsAgentPoolQueue>,
    pub queue_status: Option<AzureDevOpsBuildDefinitionQueueStatus>,
    pub r#type: Option<AzureDevOpsBuildDefinitionType>,
    pub url: Option<AzureDevOpsBuildDefinitionUrl>,
    pub project: Option<AzureDevOpsProjectReference>,
    #[facet(rename = "_links")]
    pub links: Option<ArbitraryJson>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionReference);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsBuildDefinitionReference>);
