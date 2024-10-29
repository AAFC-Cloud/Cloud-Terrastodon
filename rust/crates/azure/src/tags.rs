use std::collections::HashMap;

use anyhow::bail;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::ResourceId;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_azure_types::prelude::ScopeImpl;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

use crate::prelude::invoke_batch_request;
use crate::prelude::BatchRequest;
use crate::prelude::BatchRequestEntry;

#[derive(Debug, Serialize, Deserialize)]
pub struct TagContentProperties {
    tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagContent {
    id: ResourceId,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    properties: TagContentProperties,
}
impl TagContent {
    pub fn validate(&self) {
        assert_eq!(self.name, "default");
        assert_eq!(self.kind, "Microsoft.Resources/tags");
    }
}

pub async fn get_tags_for_resources(
    resource_ids: Vec<ScopeImpl>,
) -> Result<Vec<HashMap<String, String>>> {
    let url_tail = "/providers/Microsoft.Resources/tags/default?api-version=2022-09-01";
    let batch = BatchRequest {
        requests: resource_ids
            .into_iter()
            .map(|id| BatchRequestEntry::new_get(format!("{}{}", id.expanded_form(), url_tail)))
            .collect_vec(),
    };
    let resp = invoke_batch_request::<_, TagContent>(&batch).await?;
    let mut results = Vec::with_capacity(resp.responses.len());
    for response in resp.responses {
        response.content.validate();
        results.push(response.content.properties.tags);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_resource_groups;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let resource_groups = fetch_all_resource_groups().await?;
        let tags = get_tags_for_resources(
            resource_groups
                .iter()
                .take(5)
                .map(|r| r.id.as_scope())
                .collect_vec(),
        )
        .await?;
        assert_eq!(tags.len(), 5);
        assert!(tags.iter().any(|x| !x.is_empty()));
        Ok(())
    }
}
