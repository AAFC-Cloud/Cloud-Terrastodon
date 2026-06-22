use chrono::DateTime;
use chrono::Utc;
use facet_json::RawJson;
use std::collections::VecDeque;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone, Hash, facet::Facet)]
#[facet(json::proxy = String)]
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

impl From<&AzureDevOpsWorkItemQueryId> for String {
    fn from(value: &AzureDevOpsWorkItemQueryId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for AzureDevOpsWorkItemQueryId {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
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
#[derive(Debug, Clone, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemQuery {
    #[facet(rename = "_links")]
    pub links: RawJson<'static>,
    #[facet(recursive_type)]
    pub children: Vec<AzureDevOpsWorkItemQuery>,
    pub created_by: Option<RawJson<'static>>,
    pub created_date: DateTime<Utc>,
    pub has_children: bool,
    pub id: AzureDevOpsWorkItemQueryId,
    pub is_folder: bool,
    pub is_public: bool,
    pub last_modified_by: RawJson<'static>,
    pub last_modified_date: DateTime<Utc>,
    pub name: String,
    pub path: String,
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
