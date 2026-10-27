use super::dump_plan_diagram_path_proxy::DumpPlanDiagramPathProxy;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_registry::DiscoveryPlan;
use std::path::PathBuf;

/// An offline inventory plan and the explicit scope/policy supplied by its caller.
#[derive(Debug, Clone, facet::Facet)]
pub struct DumpPlanReport {
    pub organization: Option<AzureDevOpsOrganizationUrl>,
    pub tenant: Option<AzureTenantId>,
    /// Requested policy only; no credentials were resolved or verified.
    pub auth_source: AuthSource,
    pub plan: DiscoveryPlan,
    #[facet(proxy = DumpPlanDiagramPathProxy)]
    pub diagram: Option<PathBuf>,
}

impl DumpPlanReport {
    pub(super) fn explain_assumptions(&mut self) {
        self.plan.diagnostics.push("This is a structural plan, not an execution schedule: collections and optional fields may be empty, permissions may fail, and a live runner must preserve organization/project/tenant provenance.".into());
        if matches!(
            self.auth_source,
            AuthSource::Auto | AuthSource::WorkloadIdentity | AuthSource::Browser
        ) {
            self.plan.diagnostics.push("Authentication is a declared input assumption only; source resolution, tenant configuration, and credentials have not been checked.".into());
        }
        if self.tenant.is_some() && self.auth_source == AuthSource::PersonalAccessToken {
            self.plan.diagnostics.push("A DevOps PAT cannot supply an Azure tenant authentication context; Azure inventory remains blocked.".into());
        }
    }
}
