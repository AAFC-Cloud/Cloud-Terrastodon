use cloud_terrastodon_azure::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure::prelude::ResourceGroupId;
use cloud_terrastodon_azure::prelude::ResourceTagsId;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::replace_tags_for_resources;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn tag_empty_resource_group_menu() -> Result<()> {
    info!("Fetching empty resource groups");
    let query = r#"
ResourceContainers  
| where type == "microsoft.resources/subscriptions/resourcegroups"  
| extend rgAndSub = strcat(resourceGroup, "--", subscriptionId)  
| join kind=leftouter (  
	Resources  
	| extend rgAndSub = strcat(resourceGroup, "--", subscriptionId)  
	| summarize count() by rgAndSub  
) on rgAndSub  
| where isnull(count_)  
| project id, tags
   "#;
    #[derive(Deserialize)]
    struct Row {
        id: ResourceGroupId,
        tags: HashMap<String, String>,
    }
    let empty_resource_groups = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "empty-resource-groups"]),
            valid_for: Duration::from_mins(0),
        },
    )
    .collect_all::<Row>()
    .await?;
    info!(
        "Found {} empty resource groups",
        empty_resource_groups.len()
    );
    for rg in empty_resource_groups.iter() {
        info!("- {}", rg.id.short_form());
    }

    let tag_key = "CleanupAutomationFlag";
    let tag_value = "ThisResourceContainerIsEmpty";
    info!("Adding tag {}={} to each", tag_key, tag_value);
    let result = replace_tags_for_resources(
        empty_resource_groups
            .into_iter()
            .map(|mut rg| {
                rg.tags.insert(tag_key.to_owned(), tag_value.to_owned());
                (ResourceTagsId::from_scope(&rg.id), rg.tags)
            })
            .collect(),
    )
    .await?;

    info!(
        "Tagged {} resource groups with {}={}",
        result.len(),
        tag_key,
        tag_value
    );

    Ok(())
}
