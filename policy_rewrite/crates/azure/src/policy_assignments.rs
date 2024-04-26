use anyhow::Result;
use azure_types::policy_assignments::PolicyAssignment;
use azure_types::scopes::AsScope;
use azure_types::scopes::Scope;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_policy_assignments(
    scope: Option<&impl AsScope>,
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
    cache_key.push("policy_assignments");
    if let Some(scope) = scope {
        let scope = scope.as_scope();
        cmd.args(["--scope", &scope.expanded_form()]);
        cache_key.push(scope.short_name());
    }
    if let Some(subscription) = subscription {
        cmd.args(["--subscription", &subscription]);
        cache_key.push(subscription)
    }
    cmd.use_cache_dir(Some(cache_key));
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use azure_types::management_groups::ManagementGroupId;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_assignments(None::<&ManagementGroupId>, None).await?;
        println!("Found {} policy assignments:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
