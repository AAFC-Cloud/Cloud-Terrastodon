use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::get_default_tenant_id;
use cloud_terrastodon_azure::list_tracked_tenant_aliases;
use std::io::Error;
use std::io::ErrorKind;
use teamy_cancellation::CancellationScope;
use teamy_tracing::DiagnosticReport;

/// Local tenant metadata displayed by the tenant manager.
pub(crate) struct AzureTrackedTenantCatalogue {
    pub tenants: Vec<(AzureTenantId, Vec<AzureTenantAlias>)>,
    pub default: Result<Option<AzureTenantId>, DiagnosticReport>,
}

impl AzureTrackedTenantCatalogue {
    pub async fn load(cancellation: &CancellationScope) -> eyre::Result<Self> {
        let mut tenants: Vec<_> = list_tracked_tenant_aliases().await?.into_iter().collect();
        tenants.sort_by_key(|(tenant, _)| *tenant);
        cancellation.bail_if_cancelled()?;
        let default = match get_default_tenant_id(cancellation).await {
            Ok(tenant) => Ok(Some(tenant)),
            Err(error)
                if error
                    .downcast_ref::<Error>()
                    .is_some_and(|source| source.kind() == ErrorKind::NotFound) =>
            {
                Ok(None)
            }
            // A bad default is recoverable by choosing another tenant.
            Err(error) => Err(DiagnosticReport::new(error)),
        };
        Ok(Self { tenants, default })
    }
}
