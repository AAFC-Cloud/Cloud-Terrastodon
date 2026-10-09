use crate::AzureDevOpsBuildDefinitionSummary;
use crate::AzureDevOpsBuildDefinitionTriggerType;
use crate::AzureDevOpsBuildForks;
use crate::AzureDevOpsBuildPipelineTriggerSettings;
use crate::AzureDevOpsBuildSchedule;
use arbitrary::Arbitrary;

/// A definition trigger with its published trigger-specific properties.
///
/// Microsoft REST documentation: [BuildTrigger schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildtrigger).
/// The REST reference lists `triggerType`; trigger-specific properties are described
/// in Microsoft's [TypeScript source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L1439).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildTrigger {
    pub trigger_type: AzureDevOpsBuildDefinitionTriggerType,
    pub branch_filters: Option<Vec<String>>,
    pub path_filters: Option<Vec<String>>,
    pub batch_changes: Option<bool>,
    pub max_concurrent_builds_per_branch: Option<i32>,
    pub polling_interval: Option<i32>,
    pub polling_job_id: Option<String>,
    pub settings_source_type: Option<i32>,
    pub auto_cancel: Option<bool>,
    pub forks: Option<AzureDevOpsBuildForks>,
    pub is_comment_required_for_pull_request: Option<bool>,
    pub pipeline_trigger_settings: Option<AzureDevOpsBuildPipelineTriggerSettings>,
    pub require_comments_for_non_team_member_and_non_contributors: Option<bool>,
    pub require_comments_for_non_team_members_only: Option<bool>,
    pub schedules: Option<Vec<AzureDevOpsBuildSchedule>>,
    pub run_continuous_integration: Option<bool>,
    pub use_workspace_mappings: Option<bool>,
    pub definition: Option<AzureDevOpsBuildDefinitionSummary>,
    pub requires_successful_build: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildTrigger);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildTrigger);
