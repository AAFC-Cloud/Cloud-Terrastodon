use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsProjectState {
    #[facet(rename = "wellFormed")]
    WellFormed,
}
#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsProjectVisibility {
    #[facet(rename = "private")]
    Private,
    #[facet(rename = "public")]
    Public, // just assuming this exists
}

#[derive(Debug, Eq, PartialEq, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsProject {
    pub abbreviation: Option<String>,
    pub default_team_image_url: Option<String>,
    pub description: Option<String>,
    pub id: AzureDevOpsProjectId,
    pub last_update_time: DateTime<Utc>,
    pub name: AzureDevOpsProjectName,
    pub revision: u16,
    pub state: AzureDevOpsProjectState,
    pub url: String,
    pub visibility: AzureDevOpsProjectVisibility,
}

impl From<AzureDevOpsProject> for HclImportBlock {
    fn from(project: AzureDevOpsProject) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: project.id.to_string(),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Project,
                name: project.name.sanitize(),
            },
        }
    }
}
