use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceGroupListRequest;
use cloud_terrastodon_azure::AzureResourceListRequest;
use cloud_terrastodon_azure::AzureResourceTagsPatchOperation;
use cloud_terrastodon_azure::AzureResourceTagsUpdateRequest;
use cloud_terrastodon_azure::ResourceTagsId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use cloud_terrastodon_user_input::prompt_line;
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use tracing::info;

pub async fn tag_resources_menu(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let resource_groups = AzureResourceGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;
    let resource_group = PickerTui::<_>::new()
        .set_header("Choose a resource group")
        .pick_one(
            resource_groups.into_iter().map(|rg| Choice {
                key: rg.id.expanded_form().to_string(),
                value: rg,
            }),
            executor.cancellation(),
        )
        .await?;
    let resources = AzureResourceListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?
    .into_iter()
    .filter(|res| {
        res.id
            .expanded_form()
            .starts_with(&resource_group.id.expanded_form())
    });
    let resources = PickerTui::<_>::new()
        .set_header("Choose resources to tag")
        .pick_many(
            resources.map(|r| Choice {
                key: r.id.expanded_form().to_string(),
                value: r,
            }),
            executor.cancellation(),
        )
        .await?;
    let concurrency = NonZeroUsize::try_from(20)?;
    let tag_key = prompt_line("Enter tag key: ").await?;
    let tag_value = prompt_line("Enter tag value: ").await?;
    let result = resources
        .into_iter()
        .map_concurrent(executor, concurrency, async |child, resource| {
            AzureResourceTagsUpdateRequest {
                authentication: *authentication,
                id: Cow::Owned(ResourceTagsId::from_scope(resource.as_ref())),
                operation: AzureResourceTagsPatchOperation::Merge,
                tags: HashMap::from([(tag_key.clone(), tag_value.clone())]),
            }
            .execute(&child)
            .await
        })
        .await?;
    for tags in &result {
        eyre::ensure!(
            tags.properties.tags.get(&tag_key) == Some(&tag_value),
            "tag update response did not contain the requested value"
        );
    }
    info!("Successfully added tag for {} resources", result.len());
    Ok(())
}
