use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use facet::Facet;

/// Display values for a known DevOps organization or Server collection.
#[derive(Facet)]
pub(super) struct AzureDevOpsOrganizationRow {
    pub organization: AzureDevOpsOrganizationUrl,
    pub name: String,
    pub url: String,
    pub tenant_id: Option<AzureTenantId>,
    pub different_tenant: bool,
}

impl AzureDevOpsOrganizationRow {
    pub(super) fn new(
        organization: &AzureDevOpsOrganizationUrl,
        tenant_id: Option<AzureTenantId>,
        account_tenant: Option<AzureTenantId>,
    ) -> Self {
        Self {
            organization: organization.clone(),
            name: organization.name().to_owned(),
            url: organization.expanded_form(),
            tenant_id,
            different_tenant: tenant_id
                .zip(account_tenant)
                .is_some_and(|(org, account)| org != account),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organizations_only_dim_when_both_tenants_are_known_and_differ() -> eyre::Result<()> {
        let organization = "test-org".parse()?;
        let tenant = "00000000-0000-0000-0000-000000000001".parse()?;
        let other = "00000000-0000-0000-0000-000000000002".parse()?;
        assert!(
            AzureDevOpsOrganizationRow::new(&organization, Some(tenant), Some(other))
                .different_tenant
        );
        assert!(
            !AzureDevOpsOrganizationRow::new(&organization, Some(tenant), Some(tenant))
                .different_tenant
        );
        assert!(
            !AzureDevOpsOrganizationRow::new(&organization, None, Some(tenant)).different_tenant
        );
        assert!(
            !AzureDevOpsOrganizationRow::new(&organization, Some(tenant), None).different_tenant
        );
        Ok(())
    }
}
