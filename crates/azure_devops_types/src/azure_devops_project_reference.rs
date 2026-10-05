use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use crate::AzureDevOpsProjectReferenceState;
use crate::AzureDevOpsProjectReferenceVisibility;
use crate::AzureDevOpsProjectUrl;
use crate::AzureDevOpsTeamImageUrl;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// A shallow project reference, with missing metadata preserved.
///
/// Microsoft's [TeamProjectReference](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/builds/list?view=azure-devops-rest-7.1#teamprojectreference)
/// may be embedded without the fields required by a full project object.
/// State and visibility retain unknown response values as well as known ones.
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsProjectReference {
    pub id: Option<AzureDevOpsProjectId>,
    pub name: Option<AzureDevOpsProjectName>,
    pub abbreviation: Option<String>,
    pub description: Option<String>,
    pub revision: Option<i64>,
    pub state: Option<AzureDevOpsProjectReferenceState>,
    pub visibility: Option<AzureDevOpsProjectReferenceVisibility>,
    pub last_update_time: Option<DateTime<Utc>>,
    pub url: Option<AzureDevOpsProjectUrl>,
    pub default_team_image_url: Option<AzureDevOpsTeamImageUrl>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectReference);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectReference);
