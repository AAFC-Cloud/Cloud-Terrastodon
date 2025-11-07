use crate::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_hcl_types::prelude::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsTeamId(Uuid);
impl std::fmt::Display for AzureDevOpsTeamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
    pub description: String,
    pub id: AzureDevOpsTeamId,
    #[serde(rename = "identityUrl")]
    pub identity_url: String,
    pub name: String,
    #[serde(rename = "projectId")]
    pub project_id: AzureDevOpsProjectId,
    #[serde(rename = "projectName")]
    pub project_name: String,
    pub url: String,
}

impl From<AzureDevOpsTeam> for HclImportBlock {
    fn from(team: AzureDevOpsTeam) -> Self {
        HclImportBlock {
            provider: HclProviderReference::Inherited,
            id: format!("{}/{}", team.project_id, *team.id),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Team,
                name: format!("project_{}_team_{}", team.project_name, team.name).sanitize(),
            },
        }
    }
}
