use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Also known as: QueryHierarchyItem
#[derive(Serialize, Deserialize)]
pub struct AzureDevopsWorkItemQuery {
    #[serde(rename = "_links")]
    pub _links: Value,
    #[serde(rename = "children")]
    #[serde(default)]
    pub children: Vec<AzureDevopsWorkItemQuery>,
    #[serde(rename = "createdBy")]
    pub created_by: Option<Value>,
    #[serde(rename = "createdDate")]
    pub created_date: DateTime<Utc>,
    #[serde(rename = "hasChildren")]
    #[serde(default)]
    pub has_children: bool,
    #[serde(rename = "id")]
    pub id: Value,
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
