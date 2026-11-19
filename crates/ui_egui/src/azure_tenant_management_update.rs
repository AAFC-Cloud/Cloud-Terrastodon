use crate::azure_tracked_tenant_catalogue::AzureTrackedTenantCatalogue;
use cloud_terrastodon_azure::MicrosoftGraphOrganization;
use std::sync::Arc;
use teamy_tracing::DiagnosticReport;

/// Results displayed by the tenant manager after background work.
pub(crate) enum AzureTenantManagementUpdate {
    Catalogue(AzureTrackedTenantCatalogue),
    Details(Arc<MicrosoftGraphOrganization>),
}

pub(crate) type AzureTenantManagementResult = Result<AzureTenantManagementUpdate, DiagnosticReport>;
