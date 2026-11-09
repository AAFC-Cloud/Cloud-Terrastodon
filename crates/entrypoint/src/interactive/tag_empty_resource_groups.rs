use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::ResourceGraphHelper;
use cloud_terrastodon_azure::ResourceGroupId;
use cloud_terrastodon_azure::ResourceTagsId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::replace_tags_for_resources;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn tag_empty_resource_group_menu(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
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
    #[derive(Facet)]
    struct Row {
        id: ResourceGroupId,
        tags: HashMap<String, String>,
    }
    let empty_resource_groups = ResourceGraphHelper::new(
        query,
        Some(CacheKey {
            scope: CacheKey::from_relative_path(PathBuf::from_iter([
                "az",
                "resource_graph",
                "empty-resource-groups",
            ]))?
            .scope,
            valid_for: Duration::ZERO,
        }),
        auth_context.tenant_id,
        auth_context.auth_source()?,
    )
    .collect_all::<Row>(executor)
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
        auth_context,
        empty_resource_groups
            .into_iter()
            .map(|rg| {
                let mut tags = rg.tags.clone();
                tags.insert(tag_key.to_owned(), tag_value.to_owned());
                (ResourceTagsId::from_scope(&rg.id), tags)
            })
            .collect(),
        executor,
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
