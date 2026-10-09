use crate::AzureDevOpsBuildScheduleDays;
use arbitrary::Arbitrary;

/// A schedule associated with a build definition trigger.
///
/// Microsoft REST documentation: [containing BuildTrigger schema](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/definitions/get?view=azure-devops-rest-7.1#buildtrigger).
/// The REST reference documents the base trigger. Schedule fields are described by
/// Microsoft's [Schedule interface](https://learn.microsoft.com/en-us/javascript/api/azure-devops-extension-api/schedule)
/// and [TypeScript source](https://github.com/microsoft/azure-devops-node-api/blob/master/api/interfaces/BuildInterfaces.ts#L2532).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildSchedule {
    pub branch_filters: Option<Vec<String>>,
    /// The API's flags-enum spelling, including combinations of named days.
    /// See Microsoft's [wire model](https://github.com/microsoft/azure-devops-go-api/blob/dev/azuredevops/build/models.go#L1628).
    pub days_to_build: Option<AzureDevOpsBuildScheduleDays>,
    pub schedule_job_id: Option<String>,
    pub schedule_only_with_changes: Option<bool>,
    pub start_hours: Option<i32>,
    pub start_minutes: Option<i32>,
    pub time_zone_id: Option<String>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildSchedule);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildSchedule);
