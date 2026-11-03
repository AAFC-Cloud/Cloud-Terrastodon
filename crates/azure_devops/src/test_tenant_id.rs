use cloud_terrastodon_azure_types::AzureTenantId;
use eyre::Context;

/// Live tests must select their Azure tenant explicitly before accessing services.
pub(crate) fn test_tenant_id() -> eyre::Result<AzureTenantId> {
    std::env::var("CT_TEST_AZDO_TENANT_ID")
        .wrap_err("live Azure DevOps tests require CT_TEST_AZDO_TENANT_ID")?
        .parse()
        .wrap_err("CT_TEST_AZDO_TENANT_ID must be an Azure tenant UUID")
}
