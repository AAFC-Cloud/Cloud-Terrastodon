use crate::AzureDevOpsBuildDefinitionSummary;
use crate::AzureDevOpsBuildId;
use crate::AzureDevOpsBuildNumber;
use crate::AzureDevOpsBuildResult;
use crate::AzureDevOpsBuildStatus;
use crate::AzureDevOpsBuildUrl;
use crate::AzureDevOpsProjectReference;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;

/// A build returned by Azure DevOps. Typed response status/result vocabularies
/// preserve unknown server values without losing their wire spelling.
/// Modeled properties are required; omissions surface as decoding errors.
///
/// See Microsoft's [Build schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#build).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuild {
    pub id: AzureDevOpsBuildId,
    pub build_number: AzureDevOpsBuildNumber,
    pub status: AzureDevOpsBuildStatus,
    pub result: AzureDevOpsBuildResult,
    pub definition: AzureDevOpsBuildDefinitionSummary,
    pub project: AzureDevOpsProjectReference,
    pub source_branch: String,
    pub source_version: String,
    pub queue_time: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub finish_time: DateTime<Utc>,
    pub url: AzureDevOpsBuildUrl,
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
    pub keep_forever: bool,
    pub retained_by_release: bool,
    pub deleted: bool,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuild);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuild);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsBuild>);
