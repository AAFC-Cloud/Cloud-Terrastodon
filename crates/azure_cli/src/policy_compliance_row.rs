use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::PolicyAssignmentId;
use cloud_terrastodon_azure::ResourceGraphHelper;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;
use indoc::indoc;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Facet)]
pub(crate) struct PolicyComplianceRow {
    pub policy_assignment_id: PolicyAssignmentId,
    pub policy_definition_reference_id: Option<String>,
    pub compliant_count: u32,
    pub noncompliant_count: u32,
}

pub(crate) async fn fetch_all_policy_compliance(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Vec<Arc<PolicyComplianceRow>>> {
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
            scope: CacheScope::try_from_relative_path(PathBuf::from_iter([
                "az",
                "resource_graph",
                "policy-compliance",
            ]))?,
            valid_for: Duration::from_mins(15),
        }),
        auth_context.tenant_id,
        auth_context.auth_source()?,
    )
    .collect_all::<PolicyComplianceRow>(executor)
    .await?;
    info!("Found {} policy compliance records", rtn.len());
    Ok(rtn)
}
