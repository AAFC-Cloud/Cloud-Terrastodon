use std::ops::Deref;
use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevopsProjectId::new(uuid))
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
    pub name: String,
    pub revision: u16,
    pub state: AzureDevopsProjectState,
    pub url: String,
    pub visibility: AzureDevopsProjectVisibility,
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
