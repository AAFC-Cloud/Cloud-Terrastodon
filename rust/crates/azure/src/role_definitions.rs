use crate::prelude::ResourceGraphHelper;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::RoleDefinition;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_role_definitions() -> Result<Vec<RoleDefinition>> {
    let role_assignments = ResourceGraphHelper::new(
        r#"authorizationresources
| where type =~ "microsoft.authorization/roledefinitions"
| project id, properties
| extend
    assignable_scopes = properties.assignableScopes,
    description = properties.description,
    permissions = properties.permissions,
    display_name = properties.roleName,
    ['kind'] = properties.type
| project-away properties"#
            .to_string(),
        CacheBehaviour::Some {
            path: PathBuf::from("role-definitions"),
            valid_for: Duration::from_days(1),
        },
    )
    .collect_all()
    .await?;
    Ok(role_assignments)
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use cloud_terrastodon_core_azure_types::prelude::Scope;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let results = fetch_all_role_definitions().await?;
        println!("Found {} role definitions:", results.len());
        for role in results {
            println!(
                "- {:?} {} ({})",
                role.kind,
                role.display_name,
                role.id.short_form()
            );
        }
        Ok(())
    }

    #[test]
    fn understanding_context() -> Result<()> {
        let e1 = anyhow!("Something went wrong! (base error)").context("e1 context");
        let e2 = e1.context("e2 context");

        println!("{e2:#}\n=====\n{e2:#?}\n=====");
        Ok(())
    }
}
