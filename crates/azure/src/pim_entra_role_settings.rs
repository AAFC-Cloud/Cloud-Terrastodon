use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::PimEntraRoleSettings;
use cloud_terrastodon_azure_types::uuid::Uuid;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_rest::RestRequest;
use eyre::Result;
use eyre::bail;
use std::path::PathBuf;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct EntraPimRoleSettingsRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub tenant_id: AzureTenantId,
    pub role_definition_id: Uuid,
}

pub fn fetch_entra_pim_role_settings(
    tenant_id: AzureTenantId,
    role_definition_id: Uuid,
    cancellation: &CancellationScope,
) -> EntraPimRoleSettingsRequest {
    EntraPimRoleSettingsRequest {
        cancellation: cancellation.clone(),
        tenant_id,
        role_definition_id,
    }
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
        let url = format!(
            "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleSettings?{query}",
            tenant_id = self.tenant_id,
            query = format_args!(
                "$select={}&$filter={}",
                "id,roleDefinitionId,userMemberSettings",
                format_args!("(roleDefinition/id eq '{}')", self.role_definition_id,),
            )
        );

        #[derive(facet::Facet)]
        struct Response {
            value: Vec<PimEntraRoleSettings>,
        }

        let request =
            RestRequest::new(http::Method::GET, &url, &self.cancellation)?.cache(self.cache_key());
        let mut result: Result<Response, _> = request.clone().receive().await;
        if result.is_err() {
            self.cancellation.bail_if_cancelled()?;
            // single retry - sometimes this returns a gateway error
            result = request.receive().await;
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
    use crate::get_test_tenant_id;
    use crate::pim_entra_role_assignments::fetch_my_entra_pim_role_assignments;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let Some(role_assignments) = expect_aad_premium_p2_license(
                    fetch_my_entra_pim_role_assignments(&cancellation).await,
                )
                .await?
                else {
                    return Ok(());
                };
                assert!(!role_assignments.is_empty());
                for role_assignment in role_assignments {
                    let role_setting = fetch_entra_pim_role_settings(
                        tenant_id,
                        role_assignment.role_definition_id,
                        &cancellation,
                    )
                    .await?;
                    assert!(role_setting.get_maximum_grant_period()?.as_secs() % (60 * 30) == 0);
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(EntraPimRoleSettingsRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraPimRoleSettingsRequest);

/// Fetch role settings using the delegated PIM app token.
pub async fn fetch_entra_pim_role_settings_with_graph_access_token(
    tenant_id: AzureTenantId,
    role_definition_id: Uuid,
    access_token: &str,
    cancellation: &CancellationScope,
) -> Result<PimEntraRoleSettings> {
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleSettings?{query}",
        query = format_args!(
            "$select={}&$filter={}",
            "id,roleDefinitionId,userMemberSettings",
            format_args!("(roleDefinition/id eq '{}')", role_definition_id),
        )
    );

    #[derive(facet::Facet)]
    struct Response {
        value: Vec<PimEntraRoleSettings>,
    }

    let request = RestRequest::new(http::Method::GET, &url, cancellation)?
        .tenant(tenant_id)
        .bearer_token(access_token);
    let mut result: Result<Response, _> = request.clone().receive().await;
    if result.is_err() {
        cancellation.bail_if_cancelled()?;
        result = request.receive().await;
    }
    let mut response = result?;
    if response.value.len() != 1 {
        bail!("Expected a single result, got {}", response.value.len());
    }
    Ok(response.value.pop().unwrap())
}
cloud_terrastodon_registry::register_into_future!(EntraPimRoleSettingsRequest => PimEntraRoleSettings, effects = [Read]);
