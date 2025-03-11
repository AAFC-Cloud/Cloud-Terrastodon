use std::collections::VecDeque;

use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;

pub fn flatten_queries(
    queries: &[AzureDevOpsWorkItemQuery],
) -> Vec<(Vec<&AzureDevOpsWorkItemQuery>, &AzureDevOpsWorkItemQuery)> {
    let mut rtn = Vec::new();
    let mut to_visit = VecDeque::new();
    for query in queries {
        to_visit.push_back((vec![], query));
    }
    while let Some((parents, query)) = to_visit.pop_front() {
        rtn.push((parents.to_vec(), query));
        for child in &query.children {
            to_visit.push_front(([parents.clone(), vec![child]].concat(), child));
        }
    }
    rtn
}
