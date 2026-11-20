use crate::azure_cli_account_name::AzureCliAccountName;
use cloud_terrastodon_azure_types::AzureCliAccountUserType;
use cloud_terrastodon_azure_types::AzureTenantId;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use std::fmt;

/// The CLI profile account bound to an isolated login generation.
///
/// `az account get-access-token --tenant` still selects a profile account before
/// selecting its tenant. Retain that account separately from the tenant:
/// <https://github.com/Azure/azure-cli/blob/dev/src/azure-cli-core/azure/cli/core/_profile.py>.
#[derive(Clone, PartialEq, Eq, Facet)]
pub(crate) struct AzureCliAccountIdentity {
    pub(crate) tenant: AzureTenantId,
    // Accept the original keys in saved session identities.
    #[facet(alias = "account", sensitive)]
    pub(crate) account_name: AzureCliAccountName,
    #[facet(alias = "kind")]
    pub(crate) account_type: AzureCliAccountUserType,
}

impl AzureCliAccountIdentity {
    pub(crate) const QUERY: &str =
        "{tenant:tenantId,account_name:user.name,account_type:user.type}";

    pub(crate) fn validate(&self, tenant: AzureTenantId) -> Result<()> {
        ensure!(
            self.tenant == tenant,
            "Azure CLI session returned a different tenant"
        );
        ensure!(
            self.account_type == AzureCliAccountUserType::User,
            "isolated Azure CLI login requires an interactive user; use workload identity for applications"
        );
        Ok(())
    }
}

impl fmt::Debug for AzureCliAccountIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AzureCliAccountIdentity([REDACTED])")
    }
}
