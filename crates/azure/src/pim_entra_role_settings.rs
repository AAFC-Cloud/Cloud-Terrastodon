use crate::management_groups::fetch_root_management_group;
use cloud_terrastodon_azure_types::prelude::PimEntraRoleSettings;
use cloud_terrastodon_azure_types::prelude::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use std::path::PathBuf;

pub struct EntraPimRoleSettingsRequest {
    pub role_definition_id: Uuid,
}

pub fn fetch_entra_pim_role_settings(role_definition_id: Uuid) -> EntraPimRoleSettingsRequest {
    EntraPimRoleSettingsRequest { role_definition_id }
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for EntraPimRoleSettingsRequest {
    type Output = PimEntraRoleSettings;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "pim_roleSettings",
            self.role_definition_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let tenant_id = fetch_root_management_group().await?.tenant_id;
        let url = format!(
            "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleSettings?{}",
            format_args!(
                "$select={}&$filter={}",
                "id,roleDefinitionId,userMemberSettings",
                format_args!("(roleDefinition/id eq '{}')", self.role_definition_id,),
            )
        );

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", &url]);
        cmd.cache(CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "pim_roleSettings",
            self.role_definition_id.to_string().as_ref(),
        ])));

        #[derive(Deserialize)]
        struct Response {
            value: Vec<PimEntraRoleSettings>,
        }

        let mut result: Result<Response, _> = cmd.run().await;
        if result.is_err() {
            // single retry - sometimes this returns a gateway error
            result = cmd.run().await;
        }
        let mut resp = result?;

        if resp.value.len() != 1 {
            bail!("Expected a single result, got {}", resp.value.len());
        }
        Ok(resp.value.pop().unwrap())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraPimRoleSettingsRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pim_entra_role_assignments::fetch_my_entra_pim_role_assignments;
    use crate::prelude::test_helpers::expect_aad_premium_p2_license;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(role_assignments) =
            expect_aad_premium_p2_license(fetch_my_entra_pim_role_assignments().await).await?
        else {
            return Ok(());
        };
        println!("Found {} role assignments", role_assignments.len());
        for role_assignment in role_assignments {
            let role_setting =
                fetch_entra_pim_role_settings(role_assignment.role_definition_id).await?;
            println!("- {:?}", role_setting);
            assert!(role_setting.get_maximum_grant_period()?.as_secs() % (60 * 30) == 0);
        }
        Ok(())
    }
}
