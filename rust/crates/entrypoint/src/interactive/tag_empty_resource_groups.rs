use std::path::PathBuf;
use std::time::Duration;
use anyhow::Result;
use cloud_terrastodon_core_azure::prelude::ResourceGraphHelper;
use cloud_terrastodon_core_azure::prelude::ResourceGroupId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use itertools::Itertools;
use serde::Deserialize;
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
| project id
   "#;
    #[derive(Deserialize)]
    struct Row {
        id: ResourceGroupId,
    }
    let resource_group_ids = ResourceGraphHelper::new(
        query,
        CacheBehaviour::Some {
            path: PathBuf::from("empty-resource-groups"),
            valid_for: Duration::from_mins(0),
        },
    )
    .collect_all::<Row>()
    .await?
    .into_iter()
    .map(|x| x.id)
    .collect_vec();
    info!("Found {} empty resource groups", resource_group_ids.len());
    for rg in resource_group_ids.iter() {
        info!("- {}", rg.short_form());
    }
    

    Ok(())
}
