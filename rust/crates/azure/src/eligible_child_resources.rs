use crate::management_groups::fetch_root_management_group;
use crate::resource_groups::fetch_all_resource_groups;
use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::EligibleChildResource;
use cloud_terrastodon_core_azure_types::prelude::EligibleChildResourceKind;
use cloud_terrastodon_core_azure_types::prelude::HasScope;
use cloud_terrastodon_core_azure_types::prelude::Scope;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Eq, PartialEq, Debug, Default)]
pub enum FetchChildrenBehaviour {
    #[default]
    Unspecified,
    GetAllChildren,
}

// https://learn.microsoft.com/en-us/rest/api/authorization/eligible-child-resources/get?view=rest-authorization-2020-10-01&tabs=HTTP
pub async fn fetch_eligible_child_resources(
    scope: &impl Scope,
    behaviour: FetchChildrenBehaviour,
) -> Result<Vec<EligibleChildResource>> {
    let scope = scope.expanded_form();
    let scope = scope.strip_prefix('/').unwrap_or(scope);
    let mut url = format!(
        "https://management.azure.com/{scope}/providers/Microsoft.Authorization/eligibleChildResources?api-version=2020-10-01"
    );
    if behaviour == FetchChildrenBehaviour::GetAllChildren {
        url.push_str("&getAllChildren=true");
    }
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);

    let mut cache_chunks = PathBuf::from("az rest --method GET --url eligibleChildResources");
    scope
        .split("/")
        .filter(|x| !x.is_empty())
        .for_each(|x| cache_chunks.push(x));

    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: cache_chunks,
        valid_for: Duration::from_hours(1),
    });

    #[derive(Deserialize)]
    struct Response {
        value: Vec<EligibleChildResource>,
    }

    let resp: Response = cmd.run().await?;
    Ok(resp.value)
}

pub async fn fetch_all_eligible_resource_containers() -> Result<Vec<EligibleChildResource>> {
    let root_mg = fetch_root_management_group().await?;
    let scope = root_mg.scope();
    let mut resource_containers =
        fetch_eligible_child_resources(scope, FetchChildrenBehaviour::GetAllChildren).await?;
    // this contains management groups and subscriptions

    let rgs = fetch_all_resource_groups()
        .await?
        .into_iter()
        .map(|x| EligibleChildResource {
            name: x.name.to_owned(),
            kind: EligibleChildResourceKind::ResourceGroup,
            id: x.scope().as_scope(),
        });
    resource_containers.extend(rgs);
    // extend to include resource groups

    Ok(resource_containers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::management_groups::fetch_root_management_group;
    use crate::subscriptions::fetch_all_subscriptions;
    use cloud_terrastodon_core_azure_types::prelude::HasScope;
    use cloud_terrastodon_core_azure_types::prelude::Scope;
    use cloud_terrastodon_core_user_input::prelude::pick;
    use cloud_terrastodon_core_user_input::prelude::pick_many;
    use cloud_terrastodon_core_user_input::prelude::Choice;
    use cloud_terrastodon_core_user_input::prelude::FzfArgs;
    use itertools::Itertools;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        let mg = fetch_root_management_group().await?;
        let scope = mg.scope();
        let found =
            fetch_eligible_child_resources(scope, FetchChildrenBehaviour::GetAllChildren).await?;
        assert!(found.len() > 0);
        for x in found {
            println!("- {x:?}");
        }
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn it_works2() -> Result<()> {
        let found = fetch_all_eligible_resource_containers().await?;
        assert!(found.len() > 0);
        for x in found {
            println!("- {x:?}");
        }
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn it_works3() -> Result<()> {
        let rg = fetch_all_resource_groups()
            .await?
            .into_iter()
            .find(|x| x.name == "OPSSc-Dom-Sandbox-RG")
            .unwrap();
        let found =
            fetch_eligible_child_resources(rg.scope(), FetchChildrenBehaviour::GetAllChildren)
                .await;
        assert!(
            found.is_err(),
            "GetAllChildren is only supported for management groups?"
        );
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn it_works4() -> Result<()> {
        let subs = fetch_all_subscriptions().await?;
        let sub = subs.first().unwrap();
        let found =
            fetch_eligible_child_resources(sub.scope(), FetchChildrenBehaviour::GetAllChildren)
                .await;
        assert!(
            found.is_err(),
            "GetAllChildren is only supported for management groups?"
        );
        Ok(())
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works_interactive() -> Result<()> {
        let mg = fetch_root_management_group().await?;
        let mut scope = mg.scope().as_scope().to_owned();
        loop {
            println!("{}", scope);
            let next_scope = pick(FzfArgs {
                choices: fetch_eligible_child_resources(&scope, FetchChildrenBehaviour::default())
                    .await?
                    .into_iter()
                    .map(|x| Choice {
                        key: x.name.to_owned(),
                        value: x,
                    })
                    .collect_vec(),
                prompt: None,
                header: Some("Choose a scope".to_string()),
            })?;
            scope = next_scope.value.id;
        }
    }
    #[test_log::test(tokio::test)]
    #[ignore]
    async fn it_works_interactive2() -> Result<()> {
        let chosen = pick_many(FzfArgs {
            choices: fetch_all_eligible_resource_containers()
                .await?
                .into_iter()
                .map(|x| Choice {
                    key: x.to_string(),
                    value: x,
                })
                .collect(),
            prompt: None,
            header: None,
        })?;
        assert!(chosen.len() > 0);
        Ok(())
    }
}
