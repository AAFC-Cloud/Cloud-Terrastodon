use anyhow::Result;
use azure_types::policy_definitions::PolicyDefinition;
use azure_types::scopes::Scope;
use azure_types::scopes::ScopeImpl;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_policy_definitions(
    scope: Option<ScopeImpl>,
    subscription: Option<String>,
) -> Result<Vec<PolicyDefinition>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "definition", "list", "--output", "json"]);
    let mut cache = PathBuf::new();
    cache.push("ignore");
    cache.push("policy_definitions");
    if let Some(scope) = scope {
        cmd.args(["--management-group", &scope.short_name()]);
        cache.push(scope.short_name())
    }
    if let Some(subscription) = subscription {
        cmd.args(["--subscription", &subscription]);
        cache.push(subscription)
    }
    cmd.use_cache_dir(Some(cache));
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
