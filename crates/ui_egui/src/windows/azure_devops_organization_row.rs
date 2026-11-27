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
    pub tracked: bool,
}

impl AzureDevOpsOrganizationRow {
    pub(super) fn new(
        organization: &AzureDevOpsOrganizationUrl,
        tenant_id: Option<AzureTenantId>,
        tracked: bool,
    ) -> Self {
        Self {
            organization: organization.clone(),
            name: organization.name().to_owned(),
            url: organization.expanded_form(),
            tenant_id,
            tracked,
        }
    }
}
