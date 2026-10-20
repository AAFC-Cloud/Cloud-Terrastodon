use crate::planning::GroupPlanBlocker;
use crate::planning::GroupPlanWarning;
use crate::planning::RelationshipPlan;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraGroupDisplayName;
use cloud_terrastodon::azure_types::EntraGroupId;

/// Proposed changes and drift diagnostics for one tenant-qualified group.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(rename_all = "camelCase")]
pub struct GroupPlan {
    pub tenant_id: AzureTenantId,
    pub group_id: EntraGroupId,
    pub expected_display_name: EntraGroupDisplayName,
    pub observed_display_name: Option<EntraGroupDisplayName>,
    pub members: Option<RelationshipPlan>,
    pub owners: Option<RelationshipPlan>,
    pub warnings: Vec<GroupPlanWarning>,
    pub blockers: Vec<GroupPlanBlocker>,
}

impl GroupPlan {
    /// Order diagnostics by their typed facts and remove exact duplicates.
    pub(crate) fn normalize_diagnostics(&mut self) {
        self.warnings.sort_by(GroupPlanWarning::compare);
        self.warnings.dedup();
        self.blockers.sort_by(GroupPlanBlocker::compare);
        self.blockers.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::GroupPlan;
    use crate::domain::DesiredPrincipal;
    use crate::domain::GroupRelationship;
    use crate::planning::GroupPlanBlocker;
    use crate::planning::GroupPlanWarning;
    use crate::planning::PrincipalResolutionError;
    use cloud_terrastodon::azure_types::EntraPrincipalId;
    use cloud_terrastodon::azure_types::EntraUserId;

    #[test]
    fn normalization_is_independent_of_collection_order_without_losing_distinct_facts() {
        let user_id: EntraUserId = "30000000-0000-0000-0000-000000000001".parse().unwrap();
        let mut plan = GroupPlan {
            tenant_id: "10000000-0000-0000-0000-000000000001".parse().unwrap(),
            group_id: "20000000-0000-0000-0000-000000000001".parse().unwrap(),
            expected_display_name: "Synthetic group".parse().unwrap(),
            observed_display_name: None,
            members: None,
            owners: None,
            warnings: Vec::new(),
            blockers: Vec::new(),
        };
        for observed in [Some(String::new()), None, Some(String::new())] {
            plan.warnings
                .push(GroupPlanWarning::PrincipalDisplayNameMismatch {
                    principal_id: user_id.into(),
                    expected: "Person".into(),
                    observed,
                });
        }
        // Unknown and typed wrappers identify the same object. Their wrapper
        // must not separate equal diagnostics and defeat adjacent deduplication.
        plan.warnings
            .push(GroupPlanWarning::PrincipalDisplayNameMismatch {
                principal_id: EntraPrincipalId::new(*user_id.as_ref()),
                expected: "Person".into(),
                observed: None,
            });
        for upn in ["Alex@example.invalid", "alex@example.invalid"] {
            for error in [
                PrincipalResolutionError::MissingIdentity,
                PrincipalResolutionError::NilPrincipalId,
                PrincipalResolutionError::MissingIdentity,
            ] {
                plan.blockers
                    .push(GroupPlanBlocker::PrincipalResolutionFailed {
                        relationship: GroupRelationship::Members,
                        reference: DesiredPrincipal {
                            id: Some(user_id.into()),
                            user_principal_name: Some(upn.parse().unwrap()),
                            display_name: None,
                        },
                        error,
                    });
            }
        }
        let mut reversed = plan.clone();
        reversed.warnings.reverse();
        reversed.blockers.reverse();

        plan.normalize_diagnostics();
        reversed.normalize_diagnostics();

        assert_eq!(plan, reversed);
        assert_eq!(
            facet_json::to_string(&plan).unwrap(),
            facet_json::to_string(&reversed).unwrap(),
            "equivalent diagnostics must produce the same review artifact"
        );
        assert_eq!(plan.warnings.len(), 2, "missing and empty remain distinct");
        assert_eq!(
            plan.blockers.len(),
            4,
            "retain exact UPN spelling and error facts"
        );
        let normalized = plan.clone();
        plan.normalize_diagnostics();
        assert_eq!(plan, normalized);
    }
}
