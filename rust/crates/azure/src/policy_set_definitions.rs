use anyhow::Result;
use azure_types::policy_set_definitions::PolicySetDefinition;
use azure_types::scopes::Scope;
use azure_types::scopes::ScopeImpl;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_policy_set_definitions(
    management_group: Option<ScopeImpl>,
    subscription: Option<String>,
) -> Result<Vec<PolicySetDefinition>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "set-definition", "list", "--output", "json"]);
    let mut cache_key = PathBuf::new();
    cache_key.push("az policy set-definition list");
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
            cache_key.push(format!(
                "--management-group {}",
                management_group.short_name()
            ));
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
        let result = fetch_policy_set_definitions(None, None).await?;
        println!("Found {} policy set definitions:", result.len());
        for v in result {
            println!("- {}", v);
        }
        Ok(())
    }
}
