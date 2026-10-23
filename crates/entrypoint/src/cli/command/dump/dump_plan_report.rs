use super::dump_plan_diagram_path_proxy::DumpPlanDiagramPathProxy;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_registry::DiscoveryPlan;
use std::path::PathBuf;

/// An offline inventory plan and the explicit scope/policy supplied by its caller.
#[derive(Debug, facet::Facet)]
pub struct DumpPlanReport {
    pub organization: Option<AzureDevOpsOrganizationUrl>,
    pub tenant: Option<AzureTenantId>,
    /// Requested policy only; no credentials were resolved or verified.
    pub auth_source: AuthSource,
    pub plan: DiscoveryPlan,
    #[facet(proxy = DumpPlanDiagramPathProxy)]
    pub diagram: Option<PathBuf>,
}
