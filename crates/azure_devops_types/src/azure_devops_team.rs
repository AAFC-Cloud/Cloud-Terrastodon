use std::ops::Deref;
use std::str::FromStr;

use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureDevOpsResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
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

impl From<AzureDevOpsTeam> for TofuImportBlock {
    fn from(team: AzureDevOpsTeam) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: format!("{}/{}", team.project_id.to_string(), team.id.to_string()),
            to: TofuResourceReference::AzureDevOps {
                kind: TofuAzureDevOpsResourceKind::Team,
                name: format!("project_{}_team_{}", team.project_name, team.name).sanitize(),
            },
        }
    }
}
