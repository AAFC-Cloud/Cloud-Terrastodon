use arbitrary::Arbitrary;

/// Effective protection settings for an individual pipeline trigger.
///
/// Microsoft REST documentation: [containing BuildTrigger schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildtrigger).
/// The REST reference documents the base trigger. Protection settings are described by
/// Microsoft's [PipelineTriggerSettings interface](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/pipelinetriggersettings)
/// and [TypeScript source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L2204).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildPipelineTriggerSettings {
    pub builds_enabled_for_forks: Option<bool>,
    pub enforce_job_auth_scope_for_forks: Option<bool>,
    pub enforce_no_access_to_secrets_from_forks: Option<bool>,
    pub fork_protection_enabled: Option<bool>,
    pub is_comment_required_for_pull_request: Option<bool>,
    pub require_comments_for_non_team_member_and_non_contributors: Option<bool>,
    pub require_comments_for_non_team_members_only: Option<bool>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildPipelineTriggerSettings);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildPipelineTriggerSettings);
