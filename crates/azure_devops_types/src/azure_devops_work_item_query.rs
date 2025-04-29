use std::collections::VecDeque;
use std::ops::Deref;
use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub struct AzureDevOpsWorkItemQueryId(Uuid);
impl std::fmt::Display for AzureDevOpsWorkItemQueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Deref for AzureDevOpsWorkItemQueryId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AzureDevOpsWorkItemQueryId {
    pub fn new(uuid: Uuid) -> AzureDevOpsWorkItemQueryId {
        AzureDevOpsWorkItemQueryId(uuid)
    }
}
impl FromStr for AzureDevOpsWorkItemQueryId {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(s)?;
        Ok(AzureDevOpsWorkItemQueryId::new(uuid))
    }
}
/// Also known as: QueryHierarchyItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDevOpsWorkItemQuery {
    #[serde(rename = "_links")]
    pub _links: Value,
    #[serde(rename = "children")]
    #[serde(default)]
    pub children: Vec<AzureDevOpsWorkItemQuery>,
    #[serde(rename = "createdBy")]
    pub created_by: Option<Value>,
    #[serde(rename = "createdDate")]
    pub created_date: DateTime<Utc>,
    #[serde(rename = "hasChildren")]
    #[serde(default)]
    pub has_children: bool,
    #[serde(rename = "id")]
    pub id: AzureDevOpsWorkItemQueryId,
    #[serde(rename = "isFolder")]
    #[serde(default)]
    pub is_folder: bool,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    #[serde(rename = "lastModifiedBy")]
    pub last_modified_by: Value,
    #[serde(rename = "lastModifiedDate")]
    pub last_modified_date: DateTime<Utc>,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "url")]
    pub url: String,
}

pub struct AzureDevOpsWorkItemQueryFlattenedHierarchyEntry<'a> {
    pub parents: Vec<&'a AzureDevOpsWorkItemQuery>,
    pub child: &'a AzureDevOpsWorkItemQuery,
}

impl AzureDevOpsWorkItemQuery {
    pub fn flatten<'a>(&'a self) -> Vec<AzureDevOpsWorkItemQueryFlattenedHierarchyEntry<'a>> {
        Self::flatten_many([self])
    }

    pub fn flatten_many<'a>(
        queries: impl IntoIterator<Item = &'a AzureDevOpsWorkItemQuery>,
    ) -> Vec<AzureDevOpsWorkItemQueryFlattenedHierarchyEntry<'a>> {
        let mut rtn = Vec::new();
        let mut to_visit = VecDeque::new();
        for query in queries {
            to_visit.push_back(AzureDevOpsWorkItemQueryFlattenedHierarchyEntry {
                parents: vec![],
                child: query,
            });
        }
        while let Some(entry) = to_visit.pop_front() {
            for child in &entry.child.children {
                to_visit.push_front(AzureDevOpsWorkItemQueryFlattenedHierarchyEntry {
                    parents: [entry.parents.clone(), vec![child]].concat(),
                    child,
                });
            }
            rtn.push(entry);
        }
        rtn
    }
}
