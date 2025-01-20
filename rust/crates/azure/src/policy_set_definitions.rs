use crate::prelude::gather_from_management_groups;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::ManagementGroup;
use cloud_terrastodon_core_azure_types::prelude::ManagementGroupId;
use cloud_terrastodon_core_azure_types::prelude::PolicySetDefinition;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn fetch_all_policy_set_definitions()
-> Result<HashMap<ManagementGroup, Vec<PolicySetDefinition>>> {
    gather_from_management_groups(async |mg: ManagementGroup, _pb| {
        fetch_policy_set_definitions(Some(mg.id.clone()), None).await
    })
    .await
}

pub async fn fetch_policy_set_definitions(
    management_group_id: Option<ManagementGroupId>,
    subscription_id: Option<SubscriptionId>,
) -> Result<Vec<PolicySetDefinition>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "set-definition", "list", "--output", "json"]);
    let mut cache_key = PathBuf::new();
    cache_key.push("az policy set-definition list");
    match (management_group_id, subscription_id) {
        (Some(management_group_id), Some(subscription_id)) => {
            cmd.args(["--management-group", management_group_id.short_form()]);
            cmd.args(["--subscription", subscription_id.short_form()]);
            cache_key.push(format!(
                "--management-group {} --subscription {}",
                management_group_id.short_form(),
                subscription_id.short_form()
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
        let result = fetch_policy_set_definitions(None, None).await?;
        println!("Found {} policy set definitions:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
