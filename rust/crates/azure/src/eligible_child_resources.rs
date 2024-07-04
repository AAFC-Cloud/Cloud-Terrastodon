use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use azure_types::prelude::EligibleChildResource;
use command::prelude::CacheBehaviour;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::Deserialize;

// https://learn.microsoft.com/en-us/rest/api/authorization/eligible-child-resources/get?view=rest-authorization-2020-10-01&tabs=HTTP
pub async fn fetch_eligible_child_resources(
    scope: impl AsRef<str>,
) -> Result<Vec<EligibleChildResource>> {
    let scope = scope.as_ref();
    let url = format!(
        "https://management.azure.com/{scope}/providers/Microsoft.Authorization/eligibleChildResources?api-version=2020-10-01"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az rest --method GET --url eligibleChildResources", scope]),
        valid_for: Duration::from_hours(1),
    });

    #[derive(Deserialize)]
    struct Response {
        value: Vec<EligibleChildResource>,
    }

    let resp: Response = cmd.run().await?;
    Ok(resp.value)
}

#[cfg(test)]
mod tests {
    use azure_types::prelude::HasScope;
    use azure_types::prelude::Scope;
    use crate::management_groups::fetch_all_management_groups;
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let mg = fetch_all_management_groups()
            .await?
            .into_iter()
            .find(|mg| mg.display_name == "Tenant Root Group")
            .unwrap();
        let scope = mg.scope().expanded_form();
        let found = fetch_eligible_child_resources(scope).await?;
        assert!(found.len() > 0);
        for x in found {
            println!("- {x:?}");
        }
        Ok(())
    }
}
