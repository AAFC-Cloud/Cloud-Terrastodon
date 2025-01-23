use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_core_azure_types::prelude::RoleEligibilitySchedule;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Result;
use serde::Deserialize;

pub async fn fetch_my_role_eligibility_schedules() -> Result<Vec<RoleEligibilitySchedule>> {
    let url = "https://management.azure.com/providers/Microsoft.Authorization/roleEligibilitySchedules?api-version=2020-10-01&$filter=asTarget()";
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", url]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from("az rest --method GET --url roleEligibilitySchedules"),
        valid_for: Duration::from_hours(1),
    });

    #[derive(Deserialize)]
    struct Response {
        value: Vec<RoleEligibilitySchedule>,
    }

    let mut result: Result<Response, _> = cmd.run().await;
    if result.is_err() {
        // single retry - sometimes this returns a gateway error
        result = cmd.run().await;
    }
    Ok(result?.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let found = fetch_my_role_eligibility_schedules().await?;
        assert!(found.len() > 0);
        for x in found {
            println!("- {x}");
        }
        Ok(())
    }
}
