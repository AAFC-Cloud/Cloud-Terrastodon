use crate::AzureDevOpsBuildDefinitionId;
use crate::AzureDevOpsBuildDefinitionName;
use crate::AzureDevOpsBuildDefinitionQueueStatus;
use crate::AzureDevOpsBuildDefinitionType;
use crate::AzureDevOpsBuildDefinitionUri;
use crate::AzureDevOpsBuildDefinitionUrl;
use crate::AzureDevOpsBuildFolderPath;
use crate::AzureDevOpsProjectReference;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// A possibly shallow definition reference embedded in a build response.
///
/// An omitted path must not be used to establish that a build folder is empty.
///
/// Microsoft documentation: [DefinitionReference](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#definitionreference).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildDefinitionSummary {
    pub id: AzureDevOpsBuildDefinitionId,
    pub name: Option<AzureDevOpsBuildDefinitionName>,
    pub path: Option<AzureDevOpsBuildFolderPath>,
    pub uri: Option<AzureDevOpsBuildDefinitionUri>,
    pub created_date: Option<DateTime<Utc>>,
    pub revision: Option<i32>,
    pub queue_status: Option<AzureDevOpsBuildDefinitionQueueStatus>,
    pub r#type: Option<AzureDevOpsBuildDefinitionType>,
    pub url: Option<AzureDevOpsBuildDefinitionUrl>,
    pub project: Option<AzureDevOpsProjectReference>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinitionSummary);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinitionSummary);
