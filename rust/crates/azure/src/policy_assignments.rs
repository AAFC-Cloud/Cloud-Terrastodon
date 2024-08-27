use crate::prelude::gather_from_management_groups;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::ManagementGroup;
use cloud_terrastodon_core_azure_types::prelude::PolicyAssignment;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_azure_types::prelude::ScopeImpl;
use cloud_terrastodon_core_azure_types::prelude::SubscriptionId;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn fetch_all_policy_assignments(
) -> Result<HashMap<ManagementGroup, Vec<PolicyAssignment>>> {
    gather_from_management_groups(async |mg: ManagementGroup, _pb| {
        fetch_policy_assignments(Some(mg.id.as_scope()), None).await
    })
    .await
}

pub async fn fetch_policy_assignments(
    scope: Option<ScopeImpl>,
    subscription_id: Option<SubscriptionId>,
) -> Result<Vec<PolicyAssignment>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "policy",
        "assignment",
        "list",
        "--disable-scope-strict-match",
        "--output",
        "json",
    ]);
    let mut cache_key = PathBuf::new();
    cache_key.push("az policy assignment list");
    match (scope, subscription_id) {
        (Some(scope), Some(subscription_id)) => {
            cmd.args(["--scope", scope.expanded_form()]);
            cmd.args(["--subscription", subscription_id.short_form()]);
            cache_key.push(format!(
                "--scope {} --subscription {}",
                scope.short_form(),
                subscription_id.short_form()
            ));
        }
        (Some(scope), None) => {
            cmd.args(["--scope", scope.expanded_form()]);
            cache_key.push(format!("--scope {}", scope.short_form()));
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
        let result = fetch_policy_assignments(None, None).await?;
        println!("Found {} policy assignments:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
