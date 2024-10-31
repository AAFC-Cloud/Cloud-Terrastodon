use std::collections::HashMap;

use anyhow::bail;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::ResourceId;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_azure_types::prelude::ScopeImpl;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use crate::prelude::invoke_batch_request;
use crate::prelude::BatchRequest;
use crate::prelude::BatchRequestEntry;
use crate::prelude::BatchResponse;
use crate::prelude::HttpMethod;

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
    let results = extract_tags_from_response(resp);
    Ok(results)
}

fn extract_tags_from_response(resp: BatchResponse<TagContent>) -> Vec<HashMap<String, String>> {
    let mut results = Vec::with_capacity(resp.responses.len());
    for response in resp.responses {
        response.content.validate();
        results.push(response.content.properties.tags);
    }
    results
}

pub async fn set_tags_for_resources(
    resource_tags: HashMap<ScopeImpl, HashMap<String, String>>,
) -> Result<Vec<HashMap<String, String>>> {
    let url_tail = "/providers/Microsoft.Resources/tags/default?api-version=2022-09-01";
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
    use std::io::Write;

    use cloud_terrastodon_core_fzf::pick;
    use cloud_terrastodon_core_fzf::Choice;
    use cloud_terrastodon_core_fzf::FzfArgs;

    use crate::prelude::fetch_all_resource_groups;
    use crate::prelude::fetch_all_resources;

    use super::*;

    #[tokio::test]
    async fn get_tags_test() -> Result<()> {
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

    #[tokio::test]
    #[ignore]
    async fn set_tags_test() -> Result<()> {
        let resource_groups = fetch_all_resource_groups().await?;
        let resource_group = pick(FzfArgs {
            choices: resource_groups
                .into_iter()
                .map(|rg| Choice {
                    key: rg.id.expanded_form().to_string(),
                    value: rg,
                })
                .collect_vec(),
            header: Some("Choose a resource group".to_string()),
            prompt: None,
        })?;
        let resources = fetch_all_resources()
            .await?
            .into_iter()
            .filter(|res| {
                res.id
                    .expanded_form()
                    .starts_with(resource_group.id.expanded_form())
            })
            .collect_vec();
        let resources = pick(FzfArgs {
            choices: resources.into_iter().map(|r| Choice {
                key: r.id.expanded_form().to_string(),
                value: r
            }).collect_vec(),
            header: Some("Choose resources to tag".to_string()),
            prompt: None,
        })?;
        print!("Tag key: ");
        std::io::stdout().flush()?;
        let tag_key = read_line()
        // let resources = fetch_all_resources().await?.into_iter().filter(|res| res.id)
        Ok(())
    }
}
