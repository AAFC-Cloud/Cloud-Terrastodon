use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::prelude::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::AzureDevOpsProjectId;
use crate::prelude::AzureDevOpsProjectName;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum AzureDevOpsProjectState {
    #[serde(rename = "wellFormed")]
    WellFormed,
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum AzureDevOpsProjectVisibility {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public, // just assuming this exists
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsProject {
    pub abbreviation: Option<String>,
    #[serde(rename = "defaultTeamImageUrl")]
    pub default_team_image_url: Option<String>,
    pub description: Option<String>,
    pub id: AzureDevOpsProjectId,
    #[serde(rename = "lastUpdateTime")]
    pub last_update_time: DateTime<Utc>,
    pub name: AzureDevOpsProjectName,
    pub revision: u16,
    pub state: AzureDevOpsProjectState,
    pub url: String,
    pub visibility: AzureDevOpsProjectVisibility,
}

impl From<AzureDevOpsProject> for HCLImportBlock {
    fn from(project: AzureDevOpsProject) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: project.id.to_string(),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Project,
                name: project.name.sanitize(),
            },
        }
    }
}
