use crate::prelude::gather_from_management_groups;
use anyhow::Result;
use azure_types::policy_definitions::PolicyDefinition;
use azure_types::prelude::ManagementGroup;
use azure_types::prelude::ManagementGroupId;
use azure_types::prelude::SubscriptionId;
use azure_types::scopes::Scope;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn fetch_all_policy_definitions() -> Result<HashMap<ManagementGroup, Vec<PolicyDefinition>>> {
    gather_from_management_groups(async |mg: ManagementGroup, _pb| {
        fetch_policy_definitions(Some(mg.id.clone()), None).await
    })
    .await
}

pub async fn fetch_policy_definitions(
    management_group_id: Option<ManagementGroupId>,
    subscription_id: Option<SubscriptionId>,
) -> Result<Vec<PolicyDefinition>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "definition", "list", "--output", "json"]);
    let mut cache_key = PathBuf::new();
    cache_key.push("az policy definition list");
    match (management_group_id, subscription_id) {
        (Some(management_group_id), Some(subscription_id)) => {
            cmd.args(["--management-group", management_group_id.short_form()]);
            cmd.args(["--subscription", subscription_id.short_form()]);
            cache_key.push(format!(
                "--management-group {} --subscription {}",
                management_group_id.short_form(),
                subscription_id
            ));
        }
        (Some(management_group_id), None) => {
            cmd.args(["--management-group", management_group_id.short_form()]);
            cache_key.push(format!(
                "--management-group {}",
                management_group_id.short_form()
            ));
        }
        (None, Some(subscription_id)) => {
            cmd.args(["--subscription", subscription_id.short_form()]);
            cache_key.push(format!("--subscription {}", subscription_id.short_form()))
        }
        (None, None) => {
            cache_key.push("(unscoped, default subscription)");
        }
    }
    cmd.use_cache_dir(cache_key);
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_definitions(None, None).await?;
        println!("Found {} policy definitions:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
