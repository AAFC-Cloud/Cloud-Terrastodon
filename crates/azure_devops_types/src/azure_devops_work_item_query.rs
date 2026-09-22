use crate::AzureDevOpsWorkItemQueryFlattenedHierarchyEntry;
use crate::AzureDevOpsWorkItemQueryId;
use arbitrary::Arbitrary;
use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure_types::ArbitraryJson;
use std::collections::VecDeque;

/// Also known as: QueryHierarchyItem
#[derive(Debug, Clone, facet::Facet, Arbitrary)]
#[facet(rename_all = "camelCase")]
pub struct AzureDevOpsWorkItemQuery {
    #[facet(rename = "_links")]
    pub links: ArbitraryJson,
    #[facet(recursive_type)]
    #[facet(default)]
    pub children: Vec<AzureDevOpsWorkItemQuery>,
    pub created_by: Option<ArbitraryJson>,
    pub created_date: DateTime<Utc>,
    #[facet(default)]
    pub has_children: bool,
    pub id: AzureDevOpsWorkItemQueryId,
    #[facet(default)]
    pub is_folder: bool,
    pub is_public: bool,
    pub last_modified_by: ArbitraryJson,
    pub last_modified_date: DateTime<Utc>,
    pub name: String,
    pub path: String,
    pub url: String,
    pub wiql: Option<String>,
    pub query_type: Option<crate::QueryType>,
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

cloud_terrastodon_registry::register_thing!(AzureDevOpsWorkItemQuery);
cloud_terrastodon_registry::register_arbitrary!(AzureDevOpsWorkItemQuery);
cloud_terrastodon_registry::register_arbitrary!(Vec<AzureDevOpsWorkItemQuery>);
