use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::AzureResourceTagsPatchOperation;
use cloud_terrastodon_azure::AzureResourceTagsUpdateRequest;
use cloud_terrastodon_azure::ResourceGraphHelper;
use cloud_terrastodon_azure::ResourceTagsId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn tag_empty_resource_group_menu(
    authentication: &AzureBearerAuthenticationSelection,
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
| project id
   "#;
    #[derive(Facet)]
    struct Row {
        id: AzureResourceGroupId,
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
        *authentication,
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
    let result = empty_resource_groups
        .into_iter()
        .map_concurrent(executor, NonZeroUsize::try_from(20)?, async |child, rg| {
            AzureResourceTagsUpdateRequest {
                authentication: *authentication,
                id: Cow::Owned(ResourceTagsId::from_scope(&rg.id)),
                operation: AzureResourceTagsPatchOperation::Merge,
                tags: HashMap::from([(tag_key.to_owned(), tag_value.to_owned())]),
            }
            .execute(&child)
            .await
        })
        .await?;

    info!(
        "Tagged {} resource groups with {}={}",
        result.len(),
        tag_key,
        tag_value
    );

    Ok(())
}
