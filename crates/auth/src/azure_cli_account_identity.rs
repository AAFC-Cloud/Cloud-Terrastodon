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
    pub(crate) account: String,
    pub(crate) kind: String,
}

impl AzureCliAccountIdentity {
    pub(crate) const QUERY: &str = "{tenant:tenantId,account:user.name,kind:user.type}";

    pub(crate) fn validate(&self, tenant: AzureTenantId) -> Result<()> {
        ensure!(
            self.tenant == tenant,
            "Azure CLI session returned a different tenant"
        );
        ensure!(
            !self.account.trim().is_empty(),
            "Azure CLI session has no account identity"
        );
        ensure!(
            self.kind == "user",
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
