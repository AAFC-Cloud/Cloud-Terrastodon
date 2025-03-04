use crate::prelude::ResourceGraphHelper;
use crate::prelude::gather_from_subscriptions;
use cloud_terrastodon_core_azure_types::prelude::RoleAssignment;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_azure_types::prelude::Subscription;
use cloud_terrastodon_core_azure_types::prelude::ThinRoleAssignment;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_all_role_assignments() -> Result<HashMap<Subscription, Vec<RoleAssignment>>> {
    info!("Fetching role assignments");
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
            "az",
            "role",
            "assignment",
            "list",
            "subscription",
            &sub.name.replace(" ", "_"),
        ]));
        cmd.run().await
    })
    .await?;

    let ra_count = role_assignments.values().map(|x| x.len()).sum::<usize>();
    let sub_count = role_assignments.keys().len();
    info!(
        "Found {} role assignments across {} subscriptions",
        ra_count, sub_count
    );
    Ok(role_assignments)
}

pub async fn fetch_all_role_assignments_v2() -> Result<Vec<ThinRoleAssignment>> {
    info!("Fetching role assignments (v2)");
    let mut query = ResourceGraphHelper::new(
        r#"
authorizationresources
| where type =~ "microsoft.authorization/roleassignments"
| project
    id,
    scope=properties.scope,
    role_definition_id=properties.roleDefinitionId,
    principal_id=properties.principalId
"#,
        CacheBehaviour::Some {
            path: PathBuf::from("role_assignments"),
            valid_for: Duration::from_hours(4),
        },
    );
    let role_assignments: Vec<ThinRoleAssignment> = query.collect_all().await?;
    info!("Found {} role assignments", role_assignments.len());
    Ok(role_assignments)
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use cloud_terrastodon_core_azure_types::prelude::RoleAssignmentId;
    use cloud_terrastodon_core_azure_types::prelude::SubscriptionScoped;
    use cloud_terrastodon_core_azure_types::prelude::SubscriptionScopedRoleAssignmentId;
    use eyre::bail;
    use itertools::Itertools;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_role_assignments().await?;
        let total = result.iter().map(|(_, v)| v.len()).sum::<usize>();
        println!("Found {} role assignments:", total);
        assert!(total > 25);
        for (sub, items) in result {
            println!("Subscription: {}", sub.name);
            for v in items {
                println!(" - {}", v);
            }
        }
        Ok(())
    }
    #[tokio::test]
    async fn it_works_v2() -> Result<()> {
        let result = fetch_all_role_assignments_v2().await?;
        println!("Found {} role assignments:", result.len());
        assert!(result.len() > 25);
        Ok(())
    }

    #[tokio::test]
    async fn count_matches() -> Result<()> {
        let thick_count = fetch_all_role_assignments()
            .await?
            .into_iter()
            .flat_map(|(_, v)| v)
            .map(|ra| ra.id)
            .unique()
            .count();
        let thin_count = fetch_all_role_assignments_v2()
            .await?
            .into_iter()
            .map(|ra| ra.id)
            .unique()
            .count();
        assert!(thick_count <= thin_count);
        assert!(thin_count - thick_count < 50); // this constant isn't significant, they just shouldn't be too far apart
        Ok(())
    }

    #[tokio::test]
    async fn scopes_make_sense() -> Result<()> {
        let result = fetch_all_role_assignments().await?;
        for (sub, items) in result {
            for ra in items {
                if ra.scope.to_lowercase() == sub.id.expanded_form().to_lowercase() {
                    let RoleAssignmentId::SubscriptionScoped(ref sub_ra_id) = ra.id else {
                        bail!(
                            "role assignment {} isn't proper subtype, should be {:?} but was {:?}",
                            ra.id.expanded_form(),
                            type_name::<SubscriptionScopedRoleAssignmentId>(),
                            ra
                        );
                    };
                    assert_eq!(sub_ra_id.subscription_id(), sub.id);
                } else {
                    if matches!(ra.id, RoleAssignmentId::SubscriptionScoped(_)) {
                        bail!(
                            "Subscription scoped role assignments should have scope matching their subscription"
                        )
                    }
                }
            }
        }
        Ok(())
    }
}
