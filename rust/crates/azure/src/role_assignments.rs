use crate::prelude::gather_from_subscriptions;
use crate::prelude::ResourceGraphHelper;
use anyhow::Result;
use azure_types::prelude::RoleAssignment;
use azure_types::prelude::Scope;
use azure_types::prelude::Subscription;
use azure_types::prelude::ThinRoleAssignment;
use command::prelude::CacheBehaviour;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_role_assignments() -> Result<HashMap<Subscription, Vec<RoleAssignment>>> {
    let role_assignments = gather_from_subscriptions(async |sub: Subscription, _pb| {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "role",
            "assignment",
            "list",
            "--include-inherited",
            "--all",
            "--output",
            "json",
            "--subscription",
            sub.id.short_form(),
        ]);
        cmd.use_cache_dir(PathBuf::from_iter([
            "az role assignment list",
            format!("--subscription {}", sub.name).as_str(),
        ]));
        cmd.run().await
    })
    .await?;
    Ok(role_assignments)
}

pub async fn fetch_all_role_assignments_v2() -> Result<Vec<ThinRoleAssignment>> {
    let mut query = ResourceGraphHelper::new(
        r#"
authorizationresources
| where type =~ "microsoft.authorization/roleassignments"
| project
    id,
    scope=properties.scope,
    role_definition_id=properties.roleDefinitionId,
    principal_id=properties.principalId
"#
        .to_string(),
        CacheBehaviour::Some {
            path: PathBuf::from("role_assignments"),
            valid_for: Duration::from_hours(4),
        },
    );
    Ok(query.collect_all().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_role_assignments().await?;
        println!("Found {} role assignments:", result.len());
        for (sub, items) in result {
            println!("Subscription: {}", sub.name);
            for v in items {
                println!(" - {}", v);
            }
        }
        Ok(())
    }
}
