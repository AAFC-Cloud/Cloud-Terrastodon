use crate::AzureDevOpsAgentPoolQueue;
use crate::AzureDevOpsBuild;
use crate::AzureDevOpsBuildAuthorizationScope;
use crate::AzureDevOpsBuildDefinitionId;
use crate::AzureDevOpsBuildDefinitionName;
use crate::AzureDevOpsBuildDefinitionQuality;
use crate::AzureDevOpsBuildDefinitionQueueStatus;
use crate::AzureDevOpsBuildDefinitionSummary;
use crate::AzureDevOpsBuildDefinitionType;
use crate::AzureDevOpsBuildDefinitionUri;
use crate::AzureDevOpsBuildDefinitionUrl;
use crate::AzureDevOpsBuildDefinitionVariable;
use crate::AzureDevOpsBuildDemand;
use crate::AzureDevOpsBuildFolderPath;
use crate::AzureDevOpsBuildMetric;
use crate::AzureDevOpsBuildOption;
use crate::AzureDevOpsBuildProcess;
use crate::AzureDevOpsBuildRepository;
use crate::AzureDevOpsBuildRetentionPolicy;
use crate::AzureDevOpsBuildTrigger;
use crate::AzureDevOpsBuildVariableGroup;
use crate::AzureDevOpsIdentityReference;
use crate::AzureDevOpsProjectReference;
use crate::AzureDevOpsReferenceLinks;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::BTreeMap;

/// A full build definition returned by the definitions GET endpoint.
///
/// Microsoft documentation: [schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#builddefinition).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildDefinition {
    pub id: AzureDevOpsBuildDefinitionId,
    pub name: AzureDevOpsBuildDefinitionName,
    pub path: AzureDevOpsBuildFolderPath,
    pub uri: AzureDevOpsBuildDefinitionUri,
    pub created_date: DateTime<Utc>,
    pub revision: i32,
    pub r#type: AzureDevOpsBuildDefinitionType,
    pub url: AzureDevOpsBuildDefinitionUrl,
    pub project: AzureDevOpsProjectReference,
    pub quality: AzureDevOpsBuildDefinitionQuality,
    pub queue_status: AzureDevOpsBuildDefinitionQueueStatus,
    pub authored_by: AzureDevOpsIdentityReference,
    #[facet(rename = "_links")]
    pub links: AzureDevOpsReferenceLinks,
    /// Full definitions include their process, unlike list references.
    pub process: AzureDevOpsBuildProcess,
    /// Repository identities can use providers other than Azure Repos.
    pub repository: AzureDevOpsBuildRepository,
    pub badge_enabled: Option<bool>,
    pub build_number_format: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub drop_location: Option<String>,
    pub draft_of: Option<AzureDevOpsBuildDefinitionSummary>,
    pub drafts: Option<Vec<AzureDevOpsBuildDefinitionSummary>>,
    pub demands: Option<Vec<AzureDevOpsBuildDemand>>,
    pub job_authorization_scope: Option<AzureDevOpsBuildAuthorizationScope>,
    pub job_cancel_timeout_in_minutes: Option<i32>,
    pub job_timeout_in_minutes: Option<i32>,
    pub latest_build: Option<AzureDevOpsBuild>,
    pub latest_completed_build: Option<AzureDevOpsBuild>,
    pub metrics: Option<Vec<AzureDevOpsBuildMetric>>,
    pub options: Option<Vec<AzureDevOpsBuildOption>>,
    /// Designer inputs and data-source bindings contain task-specific schemas.
    /// They remain an explicit opaque subtree in this read-only response model.
    pub process_parameters: Option<ArbitraryJson>,
    /// The API's genuinely open, primitive-valued PropertiesCollection.
    pub properties: Option<ArbitraryJson>,
    pub queue: Option<AzureDevOpsAgentPoolQueue>,
    pub retention_rules: Option<Vec<AzureDevOpsBuildRetentionPolicy>>,
    pub tags: Option<Vec<String>>,
    pub triggers: Option<Vec<AzureDevOpsBuildTrigger>>,
    pub variable_groups: Option<Vec<AzureDevOpsBuildVariableGroup>>,
    pub variables: Option<BTreeMap<String, AzureDevOpsBuildDefinitionVariable>>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildDefinition);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildDefinition);
