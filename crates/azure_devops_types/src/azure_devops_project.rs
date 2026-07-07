use crate::AzureDevOpsProjectId;
use crate::AzureDevOpsProjectName;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::HclImportBlock;
use cloud_terrastodon_hcl_types::HclProviderReference;
use cloud_terrastodon_hcl_types::ResourceBlockReference;
use cloud_terrastodon_hcl_types::Sanitizable;

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsProjectState {
    #[facet(rename = "wellFormed")]
    WellFormed,
}
#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
#[repr(C)]
pub enum AzureDevOpsProjectVisibility {
    #[facet(rename = "private")]
    Private,
    /// This variant has not been confirmed to actually exist
    #[facet(rename = "public")]
    Public,
}

#[derive(Debug, Eq, PartialEq, Clone, Arbitrary, facet::Facet)]
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectState);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectState);
cloud_terrastodon_registry::register_thing!(AzureDevOpsProjectVisibility);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProjectVisibility);
cloud_terrastodon_registry::register_thing!(AzureDevOpsProject);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsProject);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsProject>);
