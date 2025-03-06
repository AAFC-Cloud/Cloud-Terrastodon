use std::ops::Deref;
use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_core_tofu_types::prelude::Sanitizable;
use cloud_terrastodon_core_tofu_types::prelude::TofuAzureDevOpsResourceKind;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu_types::prelude::TofuResourceReference;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevopsProjectId(Uuid);
impl Deref for AzureDevopsProjectId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevopsProjectId {
    pub fn new(uuid: Uuid) -> AzureDevopsProjectId {
        AzureDevopsProjectId(uuid)
    }
}
impl FromStr for AzureDevopsProjectId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevopsProjectId::new(uuid))
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevopsProjectName(String);
impl Deref for AzureDevopsProjectName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for AzureDevopsProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AzureDevopsProjectName {
    pub fn new(name: String) -> AzureDevopsProjectName {
        AzureDevopsProjectName(name)
    }
}
impl FromStr for AzureDevopsProjectName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureDevopsProjectName::new(s.to_string()))
    }
}
impl AsRef<str> for AzureDevopsProjectName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum AzureDevopsProjectState {
    #[serde(rename = "wellFormed")]
    WellFormed,
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum AzureDevopsProjectVisibility {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public, // just assuming this exists
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevopsProject {
    pub abbreviation: Option<String>,
    #[serde(rename = "defaultTeamImageUrl")]
    pub default_team_image_url: Option<String>,
    pub description: Option<String>,
    pub id: AzureDevopsProjectId,
    #[serde(rename = "lastUpdateTime")]
    pub last_update_time: DateTime<Utc>,
    pub name: AzureDevopsProjectName,
    pub revision: u16,
    pub state: AzureDevopsProjectState,
    pub url: String,
    pub visibility: AzureDevopsProjectVisibility,
}

impl From<AzureDevopsProject> for TofuImportBlock {
    fn from(project: AzureDevopsProject) -> Self {
        TofuImportBlock {
            provider: TofuProviderReference::Inherited,
            id: project.id.to_string(),
            to: TofuResourceReference::AzureDevOps {
                kind: TofuAzureDevOpsResourceKind::Project,
                name: project.name.sanitize(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let id = Uuid::new_v4().to_string();
        let _project_id = id.parse::<AzureDevopsProjectId>().unwrap();
    }
}
