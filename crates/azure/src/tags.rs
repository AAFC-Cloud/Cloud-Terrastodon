use crate::AzureBatchRequest;
use crate::AzureBatchRequestEntry;
use crate::BatchResponse;
use crate::invoke_batch_request;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ResourceTagsId;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use http::Method;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Facet)]
struct TagContentProperties {
    tags: HashMap<String, String>,
}

#[derive(Debug, Facet)]
struct TagContent {
    id: ResourceTagsId,
    name: String,
    #[facet(rename = "type")]
    kind: String,
    properties: TagContentProperties,
}
impl TagContent {
    pub fn validate(&self) {
        assert_eq!(self.name, "default");
        assert_eq!(self.kind, "Microsoft.Resources/tags");
    }
}

#[derive(Debug, Facet)]
struct TagPatchContent {
    operation: String, // todo: make enum
    properties: TagContentProperties,
}

impl TagPatchContent {
    fn new(operation: &str, tags: HashMap<String, String>) -> Self {
        Self {
            operation: operation.to_string(),
            properties: TagContentProperties { tags },
        }
    }
}

pub async fn get_tags_for_resources(
    authentication: &AzureBearerAuthenticationSelection,
    resource_ids: Vec<ResourceTagsId>,
    executor: &CloudTerrastodonExecutor,
) -> Result<HashMap<ResourceTagsId, HashMap<String, String>>> {
    let url_tail = "?api-version=2022-09-01";
    let mut batch = AzureBatchRequest::new(*authentication).cancellation_safe();
    batch.requests = resource_ids
        .into_iter()
        .map(|id| AzureBatchRequestEntry::new_get(format!("{}{}", id.expanded_form(), url_tail)))
        .collect_vec();
    let resp = invoke_batch_request::<_, TagContent>(&batch, executor).await?;
    let results = extract_tags_from_response(resp);
    Ok(results)
}

/// You MUST invalidate the cache for the affected resources after calling this function
pub async fn replace_tags_for_resources(
    authentication: &AzureBearerAuthenticationSelection,
    resource_tags: HashMap<ResourceTagsId, HashMap<String, String>>,
    executor: &CloudTerrastodonExecutor,
) -> Result<HashMap<ResourceTagsId, HashMap<String, String>>> {
    let url_tail = "?api-version=2022-09-01";
    let mut batch = AzureBatchRequest::new(*authentication);
    batch.requests = resource_tags
        .into_iter()
        .map(|(resource_id, tags)| {
            AzureBatchRequestEntry::new(
                Method::PATCH,
                format!("{}{}", resource_id.expanded_form(), url_tail),
                Some(TagPatchContent::new("Replace", tags)),
            )
        })
        .collect_vec();
    let resp = invoke_batch_request(&batch, executor).await?;
    let tags = extract_tags_from_response(resp);
    Ok(tags)
}

/// You MUST invalidate the cache for the affected resources after calling this function
pub async fn merge_tags_for_resources(
    authentication: &AzureBearerAuthenticationSelection,
    resource_tags: HashMap<ResourceTagsId, HashMap<String, String>>,
    executor: &CloudTerrastodonExecutor,
) -> Result<HashMap<ResourceTagsId, HashMap<String, String>>> {
    let url_tail = "?api-version=2022-09-01";
    let mut batch = AzureBatchRequest::new(*authentication);
    batch.requests = resource_tags
        .into_iter()
        .map(|(resource_id, tags)| {
            AzureBatchRequestEntry::new(
                Method::PATCH,
                format!("{}{}", resource_id.expanded_form(), url_tail),
                Some(TagPatchContent::new("Merge", tags)),
            )
        })
        .collect_vec();
    let resp = invoke_batch_request(&batch, executor).await?;
    let tags = extract_tags_from_response(resp);
    Ok(tags)
}

/// You MUST invalidate the cache for the affected resources after calling this function
pub async fn delete_tags_for_resources(
    authentication: &AzureBearerAuthenticationSelection,
    resource_tags: HashMap<ResourceTagsId, HashMap<String, String>>,
    executor: &CloudTerrastodonExecutor,
) -> Result<HashMap<ResourceTagsId, HashMap<String, String>>> {
    let url_tail = "?api-version=2022-09-01";
    let mut batch = AzureBatchRequest::new(*authentication);
    batch.requests = resource_tags
        .into_iter()
        .map(|(resource_id, tags)| {
            AzureBatchRequestEntry::new(
                Method::PATCH,
                format!("{}{}", resource_id.expanded_form(), url_tail),
                Some(TagPatchContent::new("Delete", tags)),
            )
        })
        .collect_vec();
    let resp = invoke_batch_request(&batch, executor).await?;
    let tags = extract_tags_from_response(resp);
    Ok(tags)
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
