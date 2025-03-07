use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureDevOpsResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

use crate::prelude::AzureDevopsProject;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevopsRepoId(Uuid);
impl Deref for AzureDevopsRepoId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevopsRepoId {
    pub fn new(uuid: Uuid) -> AzureDevopsRepoId {
        AzureDevopsRepoId(uuid)
    }
}
impl FromStr for AzureDevopsRepoId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevopsRepoId::new(uuid))
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevopsRepo {
    #[serde(rename = "defaultBranch")]
    pub default_branch: Option<String>,

    pub id: AzureDevopsRepoId,

    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,

    #[serde(rename = "isFork")]
    pub is_fork: Option<bool>,

    #[serde(rename = "isInMaintenance")]
    pub is_in_maintenance: bool,

    pub name: String,

    #[serde(rename = "parentRepository")]
    pub parent_repository: Option<Value>,

    pub project: AzureDevopsProject,

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

impl From<AzureDevopsRepo> for TofuImportBlock {
    fn from(repo: AzureDevopsRepo) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: format!("{}/{}", repo.project.id.to_string(), repo.id.to_string()),
            to: TofuResourceReference::AzureDevOps {
                kind: TofuAzureDevOpsResourceKind::Repo,
                name: format!("project_{}_repo_{}", repo.project.name, repo.name).sanitize(),
            },
        }
    }
}