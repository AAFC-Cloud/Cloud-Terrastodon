use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use cloud_terrastodon_hcl_types::prelude::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

use crate::prelude::AzureDevOpsProject;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsRepoId(Uuid);
impl Deref for AzureDevOpsRepoId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsRepoId {
    pub fn new(uuid: Uuid) -> AzureDevOpsRepoId {
        AzureDevOpsRepoId(uuid)
    }
}
impl FromStr for AzureDevOpsRepoId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsRepoId::new(uuid))
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsRepo {
    #[serde(rename = "defaultBranch")]
    pub default_branch: Option<String>,

    pub id: AzureDevOpsRepoId,

    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,

    #[serde(rename = "isFork")]
    pub is_fork: Option<bool>,

    #[serde(rename = "isInMaintenance")]
    pub is_in_maintenance: bool,

    pub name: String,

    #[serde(rename = "parentRepository")]
    pub parent_repository: Option<Value>,

    pub project: AzureDevOpsProject,

    #[serde(rename = "remoteUrl")]
    pub remote_url: String,

    pub size: u64,

    #[serde(rename = "sshUrl")]
    pub ssh_url: String,

    pub url: String,

    #[serde(rename = "validRemoteUrls")]
    pub valid_remote_urls: Option<Value>,

    #[serde(rename = "webUrl")]
    pub web_url: String,
}

impl From<AzureDevOpsRepo> for HCLImportBlock {
    fn from(repo: AzureDevOpsRepo) -> Self {
        HCLImportBlock {
            provider: HCLProviderReference::Inherited,
            id: format!("{}/{}", repo.project.id, *repo.id),
            to: ResourceBlockReference::AzureDevOps {
                kind: AzureDevOpsResourceBlockKind::Repo,
                name: format!("project_{}_repo_{}", repo.project.name, repo.name).sanitize(),
            },
        }
    }
}
