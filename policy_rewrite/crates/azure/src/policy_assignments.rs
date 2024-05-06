use anyhow::Result;
use azure_types::policy_assignments::PolicyAssignment;
use azure_types::scopes::Scope;
use azure_types::scopes::ScopeImpl;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_policy_assignments(
    scope: Option<ScopeImpl>,
    subscription: Option<String>,
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
    cache_key.push("ignore");
    cache_key.push("az policy assignment list");
    match (scope, subscription) {
        (Some(scope), Some(subscription)) => {
            cmd.args(["--scope", &scope.expanded_form()]);
            cmd.args(["--subscription", &subscription]);
            cache_key.push(format!(
                "--scope {} --subscription {}",
                scope.short_name(),
                subscription
            ));
        }
        (Some(scope), None) => {
            cmd.args(["--scope", &scope.expanded_form()]);
            cache_key.push(format!("--scope {}", scope.short_name()));
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
        let result = fetch_policy_assignments(None, None).await?;
        println!("Found {} policy assignments:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
