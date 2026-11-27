use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::AzureDevOpsUserArgument;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

/// A successful entitlement inventory contained no entry for the selected user.
#[derive(Debug)]
pub struct AzureDevOpsUserLicenseEntitlementNotFound {
    pub organization: AzureDevOpsOrganizationUrl,
    pub user: AzureDevOpsUserArgument<'static>,
}

impl Display for AzureDevOpsUserLicenseEntitlementNotFound {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "No license entitlement found for user {} in organization {}",
            self.user, self.organization
        )
    }
}

impl Error for AzureDevOpsUserLicenseEntitlementNotFound {}
