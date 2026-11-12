use crate::AzureResourceGroupListRequest;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureResourceGroup;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use std::sync::Arc;

/// Returns (Resource group, Subscription name)
pub async fn get_resource_group_choices(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<Choice<Arc<AzureResourceGroup>>>> {
    let resource_groups = AzureResourceGroupListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?;

    let mut choices = Vec::new();
    for rg in resource_groups {
        choices.push(Choice {
            key: format!("{:90} - {:16} - {}", rg.name, rg.subscription_name, rg.id),
            value: rg,
        });
    }
    // sort by subscription id
    choices.sort_by(|c1, c2| c1.subscription_name.cmp(&c2.subscription_name));
    // sort by resource group name
    choices.sort_by(|c1, c2| c1.name.cmp(&c2.name));

    Ok(choices)
}
