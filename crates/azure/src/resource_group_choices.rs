use crate::fetch_all_resource_groups;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ResourceGroup;
use cloud_terrastodon_user_input::Choice;

/// Returns (Resource group, Subscription name)
pub async fn get_resource_group_choices(
    tenant_id: AzureTenantId,
) -> eyre::Result<Vec<Choice<ResourceGroup>>> {
    let resource_groups = fetch_all_resource_groups(tenant_id).await?;

    let mut choices = Vec::new();
    for rg in resource_groups {
        choices.push(Choice {
            key: format!(
                "{:90} - {:16} - {}",
                rg.name.to_owned(),
                rg.subscription_name,
                rg.id
            ),
            value: rg,
        });
    }
    // sort by subscription id
    choices.sort_by(|c1, c2| c1.subscription_name.cmp(&c2.subscription_name));
    // sort by resource group name
    choices.sort_by(|c1, c2| c1.name.cmp(&c2.name));

    Ok(choices)
}
