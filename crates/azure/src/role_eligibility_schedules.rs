use cloud_terrastodon_azure_types::prelude::RoleEligibilitySchedule;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

pub struct MyEntraRoleEligibilityScheduleListRequest;

pub fn fetch_my_role_eligibility_schedules() -> MyEntraRoleEligibilityScheduleListRequest {
    MyEntraRoleEligibilityScheduleListRequest
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for MyEntraRoleEligibilityScheduleListRequest {
    type Output = Vec<RoleEligibilitySchedule>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "roleEligibilitySchedules",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let url = "https://management.azure.com/providers/Microsoft.Authorization/roleEligibilitySchedules?api-version=2020-10-01&$filter=asTarget()";

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.cache(self.cache_key());
        cmd.args(["rest", "--method", "GET", "--url", url]);

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
}

cloud_terrastodon_command::impl_cacheable_into_future!(MyEntraRoleEligibilityScheduleListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::test_helpers::expect_aad_premium_p2_license;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(found) =
            expect_aad_premium_p2_license(fetch_my_role_eligibility_schedules().await).await?
        else {
            return Ok(());
        };
        assert!(!found.is_empty());
        for x in found {
            println!("- {x}");
        }
        Ok(())
    }
}
