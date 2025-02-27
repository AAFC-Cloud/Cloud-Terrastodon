use crate::prelude::BatchRequest;
use crate::prelude::BatchRequestEntry;
use crate::prelude::BatchResponse;
use crate::prelude::HttpMethod;
use crate::prelude::invoke_batch_request;
use cloud_terrastodon_core_azure_types::prelude::ResourceTagsId;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use eyre::Result;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct TagContentProperties {
    tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TagContent {
    id: ResourceTagsId,
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
    resource_ids: Vec<ResourceTagsId>,
) -> Result<HashMap<ResourceTagsId, HashMap<String, String>>> {
    let url_tail = "?api-version=2022-09-01";
    let batch = BatchRequest {
        requests: resource_ids
            .into_iter()
            .map(|id| BatchRequestEntry::new_get(format!("{}{}", id.expanded_form(), url_tail)))
            .collect_vec(),
    };
    let resp = invoke_batch_request::<_, TagContent>(&batch).await?;
    let results = extract_tags_from_response(resp);
    Ok(results)
}

fn extract_tags_from_response(
    resp: BatchResponse<TagContent>,
) -> HashMap<ResourceTagsId, HashMap<String, String>> {
    let mut results = HashMap::with_capacity(resp.responses.len());
    for response in resp.responses {
        response.content.validate();
        results.insert(response.content.id, response.content.properties.tags);
    }
    results
}

pub async fn set_tags_for_resources(
    resource_tags: HashMap<ResourceTagsId, HashMap<String, String>>,
) -> Result<HashMap<ResourceTagsId, HashMap<String, String>>> {
    let url_tail = "?api-version=2022-09-01";
    let batch = BatchRequest {
        requests: resource_tags
            .into_iter()
            .map(|(resource_id, tags)| {
                BatchRequestEntry::new(
                    HttpMethod::PATCH,
                    format!("{}{}", resource_id.expanded_form(), url_tail),
                    Some(json!({
                        "operation": "Replace",
                        "properties": json!({
                            "tags": tags,
                        }),
                    })),
                )
            })
            .collect_vec(),
    };
    let resp = invoke_batch_request(&batch).await?;
    let tags = extract_tags_from_response(resp);
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_resource_groups;

    #[tokio::test]
    async fn get_tags_test() -> Result<()> {
        let resource_groups = fetch_all_resource_groups().await?;
        let tags = get_tags_for_resources(
            resource_groups
                .iter()
                .take(5)
                .map(|r| ResourceTagsId::from_scope(r))
                .collect_vec(),
        )
        .await?;
        assert_eq!(tags.len(), 5);
        assert!(tags.values().any(|x| !x.is_empty()));
        Ok(())
    }
}
