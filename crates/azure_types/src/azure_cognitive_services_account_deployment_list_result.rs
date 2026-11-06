use crate::AzureCognitiveServicesAccountDeployment;
use arbitrary::Arbitrary;
use std::sync::Arc;

/// A Cognitive Services deployment inventory, retaining shared deployment records.
/// [DeploymentListResult, API 2025-06-01](https://learn.microsoft.com/en-us/rest/api/microsoftfoundry/accountmanagement/deployments/list?view=rest-microsoftfoundry-accountmanagement-2025-06-01#deploymentlistresult).
#[derive(Debug, PartialEq, Clone, Arbitrary, facet::Facet)]
pub struct AzureCognitiveServicesAccountDeploymentListResult {
    #[facet(
        default,

        proxy = crate::VecDefaultNullProxy<Arc<AzureCognitiveServicesAccountDeployment>>
    )]
    pub value: Vec<Arc<AzureCognitiveServicesAccountDeployment>>,
}
