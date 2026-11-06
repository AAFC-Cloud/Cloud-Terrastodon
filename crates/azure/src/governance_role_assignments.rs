use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraPrincipalId;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use std::path::PathBuf;
use std::sync::Arc;

/// See also: https://github.com/Azure/azure-cli/issues/28854
pub async fn fetch_governance_role_assignments_for_principal(
    principal_id: impl Into<EntraPrincipalId>,
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<Arc<GovernanceRoleAssignment>>> {
    let principal_id: EntraPrincipalId = principal_id.into();
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id eq '{}')",
        principal_id
    );
    MicrosoftGraphHelper::new(
        url,
        Some(CacheKey::from_relative_path(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "governance_role_assignments".to_string(),
            auth_context.tenant_id.to_string(),
            principal_id.to_string(),
        ]))?),
        auth_context,
    )
    .fetch_all(executor)
    .await
}

#[cfg(test)]
mod test {
    use crate::auth::fetch_current_user;
    use crate::fetch_governance_role_assignments_for_principal;
    use crate::get_test_tenant_id;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                let me = fetch_current_user(&auth_context, &executor).await?.id;
                let Some(governance_role_assignments) = expect_aad_premium_p2_license(
                    fetch_governance_role_assignments_for_principal(&me, &auth_context, &executor)
                        .await,
                    cloud_terrastodon_auth_types::AzureBearerAuthSource::AzureCli,
                    tenant_id,
                    &executor,
                )
                .await?
                else {
                    return Ok(());
                };

                assert!(!governance_role_assignments.is_empty());

                Ok(())
            })
            .await
    }
}
