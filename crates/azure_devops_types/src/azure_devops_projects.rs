use std::ops::Deref;
use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_hcl_types::prelude::AzureDevOpsResourceBlockKind;
use cloud_terrastodon_hcl_types::prelude::HCLImportBlock;
use cloud_terrastodon_hcl_types::prelude::HCLProviderReference;
use cloud_terrastodon_hcl_types::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct AzureDevOpsProjectId(Uuid);
impl std::fmt::Display for AzureDevOpsProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Deref for AzureDevOpsProjectId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsProjectId {
    pub fn new(uuid: Uuid) -> AzureDevOpsProjectId {
        AzureDevOpsProjectId(uuid)
    }
}
impl FromStr for AzureDevOpsProjectId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsProjectId::new(uuid))
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct AzureDevOpsProjectName(String);
impl Deref for AzureDevOpsProjectName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for AzureDevOpsProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AzureDevOpsProjectName {
    pub fn new(name: String) -> AzureDevOpsProjectName {
        AzureDevOpsProjectName(name)
    }
}
impl FromStr for AzureDevOpsProjectName {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AzureDevOpsProjectName::new(s.to_string()))
    }
}
impl AsRef<str> for AzureDevOpsProjectName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let id = Uuid::new_v4().to_string();
        let _project_id = id.parse::<AzureDevOpsProjectId>().unwrap();
    }
}
