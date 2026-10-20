use crate::domain::DesiredGroup;
use cloud_terrastodon::app::AuthContext;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::credentials::AzureTenantAuthContext;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;

/// Bind the entire selected scope before any Graph reads or credential exchanges.
pub(crate) fn preflight_tenants(
    groups: &[DesiredGroup],
    auth: &AuthContext,
) -> Result<BTreeMap<AzureTenantId, AzureTenantAuthContext>> {
    let required = {
        let mut tenants: Vec<_> = groups.iter().map(|group| group.tenant_id).collect();
        tenants.sort_by_key(ToString::to_string);
        tenants.dedup();
        tenants
    };
    if let Some(workload) = auth.workload_identity() {
        let missing: Vec<_> = required
            .iter()
            .filter(|tenant| **tenant != workload.tenant_id)
            .map(ToString::to_string)
            .collect();
        ensure!(
            missing.is_empty(),
            "Tenant authentication preflight failed: required [{}]; available workload-identity tenant [{}]; missing [{}]. No directory reads were started. Select a tenant explicitly with --only-tenant and its matching service connection, or supply matching credentials.",
            required
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            workload.tenant_id,
            missing.join(", ")
        );
    }
    required
        .into_iter()
        .map(|tenant| {
            auth.bind_to_azure_tenant(tenant)
                .map(|binding| (tenant, binding))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::preflight_tenants;
    use crate::DesireDocument;
    use crate::domain::DesiredGroup;
    use crate::domain::ReconciliationMode;
    use cloud_terrastodon::app::AuthContext;
    use cloud_terrastodon::app::AuthSource;
    use cloud_terrastodon::credentials::WorkloadIdentityConfig;

    fn group(tenant: &str) -> DesiredGroup {
        DesiredGroup {
            tenant_id: tenant.parse().unwrap(),
            group_id: "33333333-3333-4333-8333-333333333333".parse().unwrap(),
            display_name: "Synthetic".parse().unwrap(),
            members_mode: ReconciliationMode::Ignore,
            members: Vec::new(),
            owners_mode: ReconciliationMode::Ignore,
            owners: Vec::new(),
        }
    }

    #[test]
    fn mixed_tenants_fail_before_credential_exchange_and_explicit_selection_is_bounded() {
        let first = "11111111-1111-4111-8111-111111111111";
        let second = "22222222-2222-4222-8222-222222222222";
        let auth = AuthContext::explicit(AuthSource::WorkloadIdentity)
            .with_workload_identity(WorkloadIdentityConfig {
                tenant_id: first.parse().unwrap(),
                client_id: "44444444-4444-4444-8444-444444444444".parse().unwrap(),
                federated_token: "synthetic-unused-assertion".into(),
            })
            .unwrap();
        let desire = DesireDocument::try_new(
            vec![group(first), group(second)],
            "synthetic-document".into(),
        )
        .unwrap();
        let error = preflight_tenants(desire.groups(), &auth).unwrap_err();
        assert!(error.to_string().contains(second));
        assert!(error.to_string().contains("No directory reads"));

        let first_tenant = first.parse().unwrap();
        let selected = desire.select_tenant(first_tenant).unwrap();
        assert_eq!(selected.source_digest(), "synthetic-document");
        let bindings = preflight_tenants(selected.groups(), &auth).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[&first_tenant].tenant_id.to_string(), first);
        assert!(selected.select_tenant(second.parse().unwrap()).is_err());
    }
}
