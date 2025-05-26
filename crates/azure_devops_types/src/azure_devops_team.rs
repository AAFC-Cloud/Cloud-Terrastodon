use std::ops::Deref;
use std::str::FromStr;

use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::prelude::AzureDevOpsProjectId;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTeamId(Uuid);
impl Deref for AzureDevOpsTeamId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsTeamId {
    pub fn new(uuid: Uuid) -> AzureDevOpsTeamId {
        AzureDevOpsTeamId(uuid)
    }
}
impl FromStr for AzureDevOpsTeamId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsTeamId::new(uuid))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AzureDevOpsTeam {
    description: String,
    id: AzureDevOpsTeamId,
    #[serde(rename = "identityUrl")]
    identity_url: String,
    name: String,
    #[serde(rename = "projectId")]
    project_id: AzureDevOpsProjectId,
    #[serde(rename = "projectName")]
    project_name: String,
    url: String,
}

impl From<AzureDevOpsTeam> for HCLImportBlock {
    fn from(team: AzureDevOpsTeam) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: format!("{}/{}", team.project_id, *team.id),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Team,
                name: format!("project_{}_team_{}", team.project_name, team.name).sanitize(),
            },
        }
    }
}
