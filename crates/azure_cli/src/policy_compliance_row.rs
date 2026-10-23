use cloud_terrastodon_azure::PolicyAssignmentId;
use cloud_terrastodon_azure::ResourceGraphHelper;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use indoc::indoc;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

#[derive(Debug, facet::Facet)]
pub(crate) struct PolicyComplianceRow {
    pub policy_assignment_id: PolicyAssignmentId,
    pub policy_definition_reference_id: Option<String>,
    pub compliant_count: u32,
    pub noncompliant_count: u32,
}

pub(crate) async fn fetch_all_policy_compliance(
    auth_context: &AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> eyre::Result<Vec<PolicyComplianceRow>> {
    info!("Fetching all policy compliance information");
    let query = indoc! {r#"
policyResources
| where type =~ 'Microsoft.PolicyInsights/PolicyStates'
| where properties.complianceState !~ "Unknown"
| summarize
    compliant_count = countif(tostring(properties.complianceState) == "Compliant"),
    noncompliant_count = countif(tostring(properties.complianceState) == "NonCompliant")
    by policy_assignment_id = tostring(properties.policyAssignmentId),
       policy_definition_reference_id = tostring(properties.policyDefinitionReferenceId)
| project
    policy_assignment_id,
    policy_definition_reference_id,
    compliant_count,
    noncompliant_count
    "#};

    let rtn = ResourceGraphHelper::new(
        query,
        Some(CacheKey {
            path: PathBuf::from_iter(["az", "resource_graph", "policy-compliance"]),
            valid_for: Duration::from_mins(15),
        }),
        auth_context,
        cancellation,
    )
    .collect_all::<PolicyComplianceRow>()
    .await?;
    info!("Found {} policy compliance records", rtn.len());
    Ok(rtn)
}
