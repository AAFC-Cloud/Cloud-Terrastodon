use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_core_azure_types::prelude::RoleDefinition;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_all_role_definitions() -> Result<Vec<RoleDefinition>> {
    let role_definitions = ResourceGraphHelper::new(
        r#"authorizationresources
| where type =~ "microsoft.authorization/roledefinitions"
| project id, properties
| extend
    assignable_scopes = properties.assignableScopes,
    description = properties.description,
    permissions = properties.permissions,
    display_name = properties.roleName,
    ['kind'] = properties.type
| project-away properties"#,
        CacheBehaviour::Some {
            path: PathBuf::from("role-definitions"),
            valid_for: Duration::from_days(1),
        },
    )
    .collect_all()
    .await?;
    info!("Found {} role definitions", role_definitions.len());
    Ok(role_definitions)
}

#[cfg(test)]
mod tests {
    use cloud_terrastodon_core_azure_types::prelude::Scope;
    use eyre::eyre;

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
        let e1 = eyre!("Something went wrong! (base error)").context("e1 context");
        let e2 = e1.context("e2 context");

        println!("{e2:#}\n=====\n{e2:#?}\n=====");
        Ok(())
    }
}
