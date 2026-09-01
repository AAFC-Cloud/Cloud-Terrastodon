use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::GovernanceRoleAssignment;
use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::path::PathBuf;

/// See also: https://github.com/Azure/azure-cli/issues/28854
pub async fn fetch_governance_role_assignments_for_principal(
    principal_id: impl Into<PrincipalId>,
    auth_context: &AzureTenantAuthContext,
) -> eyre::Result<Vec<GovernanceRoleAssignment>> {
    let principal_id: PrincipalId = principal_id.into();
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id eq '{}')",
        principal_id
    );
    MicrosoftGraphHelper::new(
        url,
        Some(CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "governance_role_assignments".to_string(),
            auth_context.tenant_id.to_string(),
            principal_id.to_string(),
        ]))),
        auth_context,
    )
    .fetch_all()
    .await
}

#[cfg(test)]
mod test {
    use crate::auth::fetch_current_user;
    use crate::fetch_governance_role_assignments_for_principal;
    use crate::get_test_tenant_id;
    use crate::test_helpers::expect_aad_premium_p2_license;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let me = fetch_current_user().await?.id;
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
        let Some(governance_role_assignments) = expect_aad_premium_p2_license(
            fetch_governance_role_assignments_for_principal(&me, &auth_context).await,
        )
        .await?
        else {
            return Ok(());
        };

        assert!(!governance_role_assignments.is_empty());

        Ok(())
    }
}
