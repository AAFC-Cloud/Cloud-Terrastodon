use crate::AzureDevOpsBuildFolderPath;
use crate::AzureDevOpsIdentityReference;
use crate::AzureDevOpsProjectReference;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;

/// A build-definition folder. The required path identifies the folder;
/// Azure DevOps uses backslash-separated paths, with `\` representing root.
/// Creator and last-change identity metadata is optional in this response model;
/// absent metadata remains unknown.
///
/// Microsoft documentation: [Folder](https://learn.microsoft.com/en-us/rest/api/azure/devops/build/folders/list?view=azure-devops-rest-7.1#folder).
#[derive(Debug, Clone, PartialEq, Eq, Arbitrary, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct AzureDevOpsBuildFolder {
    pub path: AzureDevOpsBuildFolderPath,
    pub description: Option<String>,
    pub project: Option<AzureDevOpsProjectReference>,
    /// The person or process that created the folder, when returned by the service.
    pub created_by: Option<AzureDevOpsIdentityReference>,
    pub created_on: Option<DateTime<Utc>>,
    /// The person or process that last changed the folder.
    pub last_changed_by: Option<AzureDevOpsIdentityReference>,
    pub last_changed_date: Option<DateTime<Utc>>,
}

cloud_terrastodon_registry::register_thing!(AzureDevOpsBuildFolder);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsBuildFolder);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsBuildFolder>);
