use anyhow::Result;
use azure_types::policy_definitions::PolicyDefinition;
use azure_types::scopes::Scope;
use azure_types::scopes::ScopeImpl;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_policy_definitions(
    management_group: Option<ScopeImpl>,
    subscription: Option<String>,
) -> Result<Vec<PolicyDefinition>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "definition", "list", "--output", "json"]);
    let mut cache_key = PathBuf::new();
    cache_key.push("ignore");
    cache_key.push("az policy definition list");
    match (management_group, subscription) {
        (Some(management_group), Some(subscription)) => {
            cmd.args(["--management-group", management_group.short_name()]);
            cmd.args(["--subscription", &subscription]);
            cache_key.push(format!(
                "--management-group {} --subscription {}",
                management_group.short_name(),
                subscription
            ));
        }
        (Some(management_group), None) => {
            cmd.args(["--management-group", management_group.short_name()]);
            cache_key.push(format!("--management-group {}", management_group.short_name()));
        }
        (None, Some(subscription)) => {
            cmd.args(["--subscription", &subscription]);
            cache_key.push(format!("--subscription {}", subscription))
        }
        (None, None) => {
            cache_key.push("(unscoped, default subscription)");
        }
    }
    cmd.use_cache_dir(Some(cache_key));
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_definitions(None, None).await?;
        println!("Found {} policy definitions:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
