use cloud_terrastodon_azure_types::prelude::RoleEligibilitySchedule;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_my_role_eligibility_schedules() -> Result<Vec<RoleEligibilitySchedule>> {
    let url = "https://management.azure.com/providers/Microsoft.Authorization/roleEligibilitySchedules?api-version=2020-10-01&$filter=asTarget()";
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", url]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "rest", "GET", "roleEligibilitySchedules"]),
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
        assert!(!found.is_empty());
        for x in found {
            println!("- {x}");
        }
        Ok(())
    }
}
