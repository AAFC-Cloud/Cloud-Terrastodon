use anyhow::Result;
use azure_types::policy_initiatives::PolicyInitiative;
use azure_types::scopes::AsScope;
use azure_types::scopes::Scope;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_policy_initiatives(
    management_group: Option<&impl AsScope>,
    subscription: Option<String>,
) -> Result<Vec<PolicyInitiative>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["policy", "set-definition", "list", "--output", "json"]);
    let mut cache = PathBuf::new();
    cache.push("ignore");
    cache.push("policy_initiatives");
    if let Some(scope) = management_group {
        let scope = scope.as_scope();
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
    use azure_types::management_groups::ManagementGroupId;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_policy_initiatives(None::<&ManagementGroupId>, None).await?;
        println!("Found {} policy initiatives:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}
