use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use facet::Facet;
use std::sync::Arc;

/// The observed outcome of one requested user-license change.
#[derive(Facet)]
#[repr(u8)]
pub enum AzureDevOpsLicenseUpdateOutcome {
    NoChangeNeeded(AzureDevOpsUserArgument<'static>),
    SkippedDueToLicenseMatchFailureDueToMissingLicense(AzureDevOpsUserArgument<'static>),
    SkippedDueToLicenseMatchFailure {
        user: AzureDevOpsUserArgument<'static>,
        was: Arc<AzureDevOpsUserLicenseEntitlement>,
    },
    Updated {
        user: AzureDevOpsUserArgument<'static>,
        was: Option<Arc<AzureDevOpsUserLicenseEntitlement>>,
    },
    UpdateDidntTakeEffect {
        user: AzureDevOpsUserArgument<'static>,
        was: Option<Arc<AzureDevOpsUserLicenseEntitlement>>,
        now: Arc<AzureDevOpsUserLicenseEntitlement>,
    },
}
