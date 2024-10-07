use std::ops::Deref;
use std::str::FromStr;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
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
    type Err = anyhow::Error;

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

    pub size: u32,

    #[serde(rename = "sshUrl")]
    pub ssh_url: String,

    pub url: String,

    #[serde(rename = "validRemoteUrls")]
    pub valid_remote_urls: Option<Value>,

    #[serde(rename = "webUrl")]
    pub web_url: String,
}
