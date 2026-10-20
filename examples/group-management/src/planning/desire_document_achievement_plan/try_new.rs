use super::DesireDocumentAchievementPlan;
use crate::DesireDocument;
use crate::domain::DesiredPrincipal;
use crate::domain::GroupRelationship;
use crate::domain::GroupSnapshot;
use crate::domain::ObservedPrincipal;
use crate::domain::PrincipalResolution;
use crate::domain::ReconciliationMode;
use crate::planning::GroupPlan;
use crate::planning::GroupPlanBlocker;
use crate::planning::GroupPlanWarning;
use crate::planning::PrincipalResolutionError;
use crate::planning::RelationshipPlan;
use chrono::Utc;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraDirectoryObjectType;
use eyre::Result;
use eyre::ensure;
use std::collections::HashSet;

impl DesireDocumentAchievementPlan {
    /// Compare desired relationships against fresh, tenant-qualified observations.
    ///
    /// The desire document has already validated local input. Directory/identity
    /// discrepancies become blockers in the reviewable plan. Except for its creation
    /// timestamp, ordering and output depend only on the supplied inputs; no
    /// credentials, caches, or directory data are consulted here.
    pub fn try_new(
        document: &DesireDocument,
        snapshots: &[GroupSnapshot],
        resolutions: &[PrincipalResolution],
    ) -> Result<Self> {
        let desired = document.groups();
        let mut snapshot_keys = HashSet::new();
        for snapshot in snapshots {
            ensure!(
                snapshot_keys.insert((snapshot.tenant_id, snapshot.group_id)),
                "Duplicate observed group {} in tenant {}",
                snapshot.group_id,
                snapshot.tenant_id
            );
        }
        let mut groups = Vec::with_capacity(desired.len());
        for group in desired {
            let mut plan = GroupPlan {
                tenant_id: group.tenant_id,
                group_id: group.group_id,
                expected_display_name: group.display_name.clone(),
                observed_display_name: None,
                members: None,
                owners: None,
                warnings: Vec::new(),
                blockers: Vec::new(),
            };
            let Some(snapshot) = snapshots.iter().find(|snapshot| {
                snapshot.tenant_id == group.tenant_id && snapshot.group_id == group.group_id
            }) else {
                plan.blockers.push(GroupPlanBlocker::MissingSnapshot {
                    tenant_id: group.tenant_id,
                    group_id: group.group_id,
                });
                groups.push(plan);
                continue;
            };
            plan.observed_display_name = Some(snapshot.display_name.clone());
            if group.display_name != snapshot.display_name {
                plan.warnings
                    .push(GroupPlanWarning::GroupDisplayNameMismatch {
                        expected: group.display_name.clone(),
                        observed: snapshot.display_name.clone(),
                    });
            }
            if group.members_mode != ReconciliationMode::Ignore {
                plan.members = Some(plan_relationship(
                    group.tenant_id,
                    GroupRelationship::Members,
                    group.members_mode,
                    &group.members,
                    &snapshot.members,
                    snapshot.members_complete,
                    resolutions,
                    &mut plan.warnings,
                    &mut plan.blockers,
                ));
            }
            if group.owners_mode != ReconciliationMode::Ignore {
                plan.owners = Some(plan_relationship(
                    group.tenant_id,
                    GroupRelationship::Owners,
                    group.owners_mode,
                    &group.owners,
                    &snapshot.owners,
                    snapshot.owners_complete,
                    resolutions,
                    &mut plan.warnings,
                    &mut plan.blockers,
                ));
            }
            plan.normalize_diagnostics();
            groups.push(plan);
        }
        groups.sort_by_key(|group| (group.tenant_id, group.group_id.to_string()));
        let mut document_tenant_ids: Vec<_> = desired.iter().map(|group| group.tenant_id).collect();
        document_tenant_ids.sort();
        document_tenant_ids.dedup();
        Ok(Self {
            schema_version: 2,
            created_at: Utc::now(),
            source_digest: document.source_digest().to_owned(),
            selected_tenant_id: None,
            document_tenant_ids,
            excluded_tenant_ids: Vec::new(),
            groups,
        })
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "These are the explicit inputs of one relationship comparison; a parameter bag would hide its dependencies"
)]
fn plan_relationship(
    tenant_id: AzureTenantId,
    relationship: GroupRelationship,
    mode: ReconciliationMode,
    desired: &[DesiredPrincipal],
    observed: &[ObservedPrincipal],
    inventory_complete: bool,
    resolutions: &[PrincipalResolution],
    warnings: &mut Vec<GroupPlanWarning>,
    blockers: &mut Vec<GroupPlanBlocker>,
) -> RelationshipPlan {
    let blocker_count = blockers.len();
    if mode == ReconciliationMode::Exact && !inventory_complete {
        blockers.push(GroupPlanBlocker::IncompleteExactInventory { relationship });
    }
    if mode == ReconciliationMode::Additive && !inventory_complete {
        warnings.push(GroupPlanWarning::IncompleteAdditiveInventory { relationship });
    }
    let mut resolved = Vec::with_capacity(desired.len());
    let mut ids = HashSet::new();
    for reference in desired {
        match resolve_principal(tenant_id, reference, resolutions, warnings) {
            Ok(principal) => {
                if let Some(blocker) = principal_eligibility_blocker(relationship, &principal) {
                    blockers.push(blocker);
                }
                if ids.insert(principal.id) {
                    resolved.push(principal);
                } else {
                    blockers.push(GroupPlanBlocker::DuplicateDesiredPrincipal {
                        relationship,
                        principal_id: principal.id,
                    });
                }
            }
            Err(error) => blockers.push(GroupPlanBlocker::PrincipalResolutionFailed {
                relationship,
                reference: reference.clone(),
                error,
            }),
        }
    }
    let mut observed = observed.to_vec();
    {
        let principals: &mut [ObservedPrincipal] = &mut observed;
        principals.sort_by_key(|principal| principal.id.to_string());
    };
    for principal in &observed {
        if let Some(blocker) = principal_eligibility_blocker(relationship, principal) {
            blockers.push(blocker);
        }
    }
    for pair in observed.windows(2) {
        if pair[0].id == pair[1].id {
            blockers.push(GroupPlanBlocker::DuplicateObservedPrincipal {
                relationship,
                principal_id: pair[0].id,
            });
        }
    }
    if observed.iter().any(|principal| principal.id.is_nil()) {
        blockers.push(GroupPlanBlocker::NilObservedPrincipal { relationship });
    }
    {
        let principals: &mut [ObservedPrincipal] = &mut resolved;
        principals.sort_by_key(|principal| principal.id.to_string());
    };
    if !inventory_complete {
        for principal in &resolved {
            if principal.kind == EntraDirectoryObjectType::ServicePrincipal
                && !observed.iter().any(|present| present.id == principal.id)
            {
                blockers.push(GroupPlanBlocker::ServicePrincipalPresenceUnknown {
                    relationship,
                    principal_id: principal.id,
                });
            }
        }
    }
    let mut add = Vec::new();
    let mut remove = Vec::new();
    // Never emit executable-looking changes for a relationship with unresolved
    // identities or an unsafe baseline. Other relationships can still be reviewed.
    if blockers.len() == blocker_count {
        let observed_ids: HashSet<_> = observed.iter().map(|principal| principal.id).collect();
        add.extend(
            resolved
                .iter()
                .filter(|principal| !observed_ids.contains(&principal.id))
                .cloned(),
        );
        if mode == ReconciliationMode::Exact {
            remove.extend(
                observed
                    .iter()
                    .filter(|principal| !ids.contains(&principal.id))
                    .cloned(),
            );
        }
    }
    RelationshipPlan {
        mode,
        inventory_complete,
        observed,
        desired: resolved,
        add,
        remove,
    }
}

/// Unsupported baseline objects block a relationship as well as desired objects,
/// so exact reconciliation cannot propose their removal without an owned policy.
fn principal_eligibility_blocker(
    relationship: GroupRelationship,
    principal: &ObservedPrincipal,
) -> Option<GroupPlanBlocker> {
    match principal.kind {
        EntraDirectoryObjectType::Group if relationship == GroupRelationship::Owners => {
            Some(GroupPlanBlocker::GroupCannotBeOwner {
                principal_id: principal.id,
            })
        }
        EntraDirectoryObjectType::User
        | EntraDirectoryObjectType::Group
        | EntraDirectoryObjectType::ServicePrincipal => None,
        EntraDirectoryObjectType::Device | EntraDirectoryObjectType::OrganizationalContact => {
            Some(GroupPlanBlocker::UnsupportedPrincipalKind {
                relationship,
                principal_id: principal.id,
                actual_kind: principal.kind,
            })
        }
    }
}

fn resolve_principal(
    tenant_id: AzureTenantId,
    desired: &DesiredPrincipal,
    resolutions: &[PrincipalResolution],
    warnings: &mut Vec<GroupPlanWarning>,
) -> Result<ObservedPrincipal, PrincipalResolutionError> {
    let mut matches = resolutions.iter().filter(|resolution| {
        resolution.tenant_id == tenant_id && resolution.reference.same_identity(desired)
    });
    let resolution = matches
        .next()
        .ok_or(PrincipalResolutionError::LookupNotCompleted { tenant_id })?;
    if matches.next().is_some() {
        return Err(PrincipalResolutionError::DuplicateLookupRecords { tenant_id });
    }
    if let (Some(expected_upn), Some(principal)) =
        (&desired.user_principal_name, &resolution.by_upn)
    {
        if principal.kind != EntraDirectoryObjectType::User {
            return Err(PrincipalResolutionError::UpnReturnedNonUser {
                upn: expected_upn.clone(),
                actual_kind: principal.kind,
            });
        }
        if !principal
            .user_principal_name
            .as_ref()
            .is_some_and(|actual| actual.matches(expected_upn))
        {
            return Err(PrincipalResolutionError::UpnMismatch {
                upn: expected_upn.clone(),
                principal_id: principal.id,
                observed: principal.user_principal_name.clone(),
            });
        }
    }
    let principal = if let Some(id) = desired.id {
        let principal =
            resolution
                .by_id
                .as_ref()
                .ok_or(PrincipalResolutionError::PinnedPrincipalNotFound {
                    tenant_id,
                    principal_id: id,
                })?;
        if principal.id != id {
            return Err(PrincipalResolutionError::PinnedIdMismatch {
                expected: id,
                observed: principal.id,
            });
        }
        if let Some(expected_upn) = &desired.user_principal_name {
            if principal.kind != EntraDirectoryObjectType::User {
                return Err(PrincipalResolutionError::PinnedPrincipalNotUser {
                    principal_id: id,
                    upn: expected_upn.clone(),
                    actual_kind: principal.kind,
                });
            }
            if let Some(upn_principal) = &resolution.by_upn
                && upn_principal.id != id
            {
                return Err(PrincipalResolutionError::UpnIdMismatch {
                    upn: expected_upn.clone(),
                    pinned_id: id,
                    observed_id: upn_principal.id,
                });
            }
            match &principal.user_principal_name {
                Some(actual_upn) if !actual_upn.matches(expected_upn) => {
                    warnings.push(GroupPlanWarning::PrincipalUserPrincipalNameMismatch {
                        principal_id: id,
                        expected: expected_upn.clone(),
                        observed: actual_upn.clone(),
                    });
                }
                None => warnings.push(GroupPlanWarning::PrincipalUserPrincipalNameUnavailable {
                    principal_id: id,
                    expected: expected_upn.clone(),
                }),
                Some(_) if resolution.by_upn.is_none() => {
                    return Err(PrincipalResolutionError::ConflictingUpnLookup {
                        principal_id: id,
                        upn: expected_upn.clone(),
                    });
                }
                Some(_) => {}
            }
        }
        principal
    } else {
        let expected_upn = desired
            .user_principal_name
            .as_ref()
            .ok_or(PrincipalResolutionError::MissingIdentity)?;
        resolution
            .by_upn
            .as_ref()
            .ok_or_else(|| PrincipalResolutionError::UpnNotFound {
                tenant_id,
                upn: expected_upn.clone(),
            })?
    };
    if principal.id.is_nil() {
        return Err(PrincipalResolutionError::NilPrincipalId);
    }
    if let Some(expected_name) = &desired.display_name
        && principal.display_name.as_ref() != Some(expected_name)
    {
        warnings.push(GroupPlanWarning::PrincipalDisplayNameMismatch {
            principal_id: principal.id,
            expected: expected_name.clone(),
            observed: principal.display_name.clone(),
        });
    }
    Ok(principal.clone())
}

#[cfg(test)]
mod tests {
    use super::DesireDocumentAchievementPlan;
    use crate::DesireDocument;
    use crate::domain::DesiredGroup;
    use crate::domain::DesiredPrincipal;
    use crate::domain::GroupRelationship;
    use crate::domain::GroupSnapshot;
    use crate::domain::ObservedPrincipal;
    use crate::domain::PrincipalResolution;
    use crate::domain::ReconciliationMode;
    use crate::planning::GroupPlanBlocker;
    use crate::planning::GroupPlanWarning;
    use crate::planning::PrincipalResolutionError;
    use cloud_terrastodon::azure_types::AzureTenantId;
    use cloud_terrastodon::azure_types::EntraDirectoryObjectType;

    fn tenant(number: u8) -> AzureTenantId {
        format!("10000000-0000-0000-0000-{number:012}")
            .parse()
            .unwrap()
    }

    fn observed(number: u8) -> ObservedPrincipal {
        ObservedPrincipal {
            id: format!("30000000-0000-0000-0000-{number:012}")
                .parse()
                .unwrap(),
            kind: EntraDirectoryObjectType::User,
            display_name: Some(format!("Person {number}")),
            user_principal_name: Some(format!("person{number}@example.invalid").parse().unwrap()),
        }
    }

    fn reference(principal: &ObservedPrincipal) -> DesiredPrincipal {
        DesiredPrincipal {
            id: Some(principal.id),
            user_principal_name: None,
            display_name: None,
        }
    }

    fn resolution(principal: &ObservedPrincipal) -> PrincipalResolution {
        PrincipalResolution {
            tenant_id: tenant(1),
            reference: reference(principal),
            by_id: Some(principal.clone()),
            by_upn: None,
        }
    }

    fn document(groups: Vec<DesiredGroup>) -> DesireDocument {
        DesireDocument::try_new(groups, "synthetic-input".into()).unwrap()
    }

    #[test]
    fn a_upn_cannot_annotate_a_pinned_non_user_object() {
        let mut group = observed(1);
        group.kind = EntraDirectoryObjectType::Group;
        group.user_principal_name = None;
        let mut reference = reference(&group);
        reference.user_principal_name = Some("alex@example.invalid".parse().unwrap());
        let mut lookup = resolution(&group);
        lookup.reference = reference.clone();
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Additive, vec![reference])]),
            &[snapshot(Vec::new())],
            &[lookup],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(plan.change_count(), 0);
        assert!(matches!(
            &plan.groups[0].blockers[..],
            [GroupPlanBlocker::PrincipalResolutionFailed {
                relationship: GroupRelationship::Members,
                error: PrincipalResolutionError::PinnedPrincipalNotUser {
                    principal_id,
                    actual_kind: EntraDirectoryObjectType::Group,
                    ..
                },
                ..
            }] if *principal_id == group.id
        ));
    }

    fn desired(mode: ReconciliationMode, principals: Vec<DesiredPrincipal>) -> DesiredGroup {
        DesiredGroup {
            tenant_id: tenant(1),
            group_id: "20000000-0000-0000-0000-000000000001".parse().unwrap(),
            display_name: "Example group".parse().unwrap(),
            members_mode: mode,
            members: principals,
            owners_mode: ReconciliationMode::Ignore,
            owners: Vec::new(),
        }
    }

    fn snapshot(members: Vec<ObservedPrincipal>) -> GroupSnapshot {
        GroupSnapshot {
            tenant_id: tenant(1),
            group_id: "20000000-0000-0000-0000-000000000001".parse().unwrap(),
            display_name: "Example group".parse().unwrap(),
            members,
            owners: Vec::new(),
            members_complete: true,
            owners_complete: true,
        }
    }

    #[test]
    fn additive_keeps_unlisted_members_and_ignored_owners_unmanaged() {
        let existing = observed(1);
        let missing = observed(2);
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Additive,
                vec![reference(&missing)],
            )]),
            &[snapshot(vec![existing])],
            &[resolution(&missing)],
        )
        .unwrap();
        let members = plan.groups[0].members.as_ref().unwrap();
        assert!(!plan.has_blockers());
        assert_eq!(members.add, vec![missing]);
        assert!(members.remove.is_empty());
        assert!(plan.groups[0].owners.is_none());
        assert_eq!(plan.change_count(), 1);
    }

    #[test]
    fn exact_lists_plan_both_additions_and_removals_from_complete_inventory() {
        let existing = observed(1);
        let missing = observed(2);
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Exact,
                vec![reference(&missing)],
            )]),
            &[snapshot(vec![existing.clone()])],
            &[resolution(&missing)],
        )
        .unwrap();
        let members = plan.groups[0].members.as_ref().unwrap();
        assert!(!plan.has_blockers());
        assert_eq!(members.add, vec![missing]);
        assert_eq!(members.remove, vec![existing]);
    }

    #[test]
    fn empty_exact_and_additive_lists_manage_relationships_while_ignore_leaves_them_unmanaged() {
        let observed = snapshot(vec![observed(1)]);
        let exact = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Exact, Vec::new())]),
            std::slice::from_ref(&observed),
            &[],
        )
        .unwrap();
        let additive = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Additive, Vec::new())]),
            std::slice::from_ref(&observed),
            &[],
        )
        .unwrap();
        let ignored = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Ignore, Vec::new())]),
            &[observed],
            &[],
        )
        .unwrap();
        assert_eq!(exact.change_count(), 1);
        assert_eq!(exact.groups[0].members.as_ref().unwrap().remove.len(), 1);
        assert_eq!(additive.change_count(), 0);
        assert!(additive.groups[0].members.is_some());
        assert_eq!(ignored.change_count(), 0);
        assert!(ignored.groups[0].members.is_none());
        assert!(ignored.groups[0].owners.is_none());
    }

    #[test]
    fn ignored_relationships_do_not_examine_incomplete_or_unsupported_inventory() {
        let mut device = observed(1);
        device.kind = EntraDirectoryObjectType::Device;
        device.user_principal_name = None;
        let mut contact = observed(2);
        contact.kind = EntraDirectoryObjectType::OrganizationalContact;
        contact.user_principal_name = None;
        let mut observed = snapshot(vec![device]);
        observed.owners = vec![contact];
        observed.members_complete = false;
        observed.owners_complete = false;

        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Ignore, Vec::new())]),
            &[observed],
            &[],
        )
        .unwrap();

        assert!(plan.groups[0].members.is_none());
        assert!(plan.groups[0].owners.is_none());
        assert!(plan.groups[0].warnings.is_empty());
        assert!(plan.groups[0].blockers.is_empty());
        assert_eq!(plan.change_count(), 0);
    }

    #[test]
    fn incomplete_exact_inventory_blocks_changes_instead_of_guessing_removals() {
        let missing = observed(2);
        let mut observed = snapshot(vec![observed(1)]);
        observed.members_complete = false;
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Exact,
                vec![reference(&missing)],
            )]),
            &[observed],
            &[resolution(&missing)],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(plan.change_count(), 0);
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::IncompleteExactInventory {
                relationship: GroupRelationship::Members,
            }]
        );
    }

    #[test]
    fn incomplete_additive_inventory_cannot_establish_a_service_principal_is_missing() {
        let mut service_principal = observed(2);
        service_principal.kind = EntraDirectoryObjectType::ServicePrincipal;
        service_principal.user_principal_name = None;
        let mut observed = snapshot(Vec::new());
        observed.members_complete = false;
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Additive,
                vec![reference(&service_principal)],
            )]),
            &[observed],
            &[resolution(&service_principal)],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(plan.change_count(), 0);
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::ServicePrincipalPresenceUnknown {
                relationship: GroupRelationship::Members,
                principal_id: service_principal.id,
            }]
        );
        assert_eq!(
            plan.groups[0].warnings,
            vec![GroupPlanWarning::IncompleteAdditiveInventory {
                relationship: GroupRelationship::Members,
            }]
        );
    }

    #[test]
    fn stale_upn_annotations_warn_and_keep_the_pinned_id() {
        let current = observed(1);
        let mut request = reference(&current);
        request.user_principal_name = Some("old-name@example.invalid".parse().unwrap());
        let resolution = PrincipalResolution {
            tenant_id: tenant(1),
            reference: request.clone(),
            by_id: Some(current.clone()),
            by_upn: None,
        };
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Additive, vec![request])]),
            &[snapshot(Vec::new())],
            &[resolution],
        )
        .unwrap();
        assert!(!plan.has_blockers());
        assert_eq!(
            plan.groups[0].members.as_ref().unwrap().add,
            vec![current.clone()]
        );
        assert_eq!(
            plan.groups[0].warnings,
            vec![GroupPlanWarning::PrincipalUserPrincipalNameMismatch {
                principal_id: current.id,
                expected: "old-name@example.invalid".parse().unwrap(),
                observed: current.user_principal_name.unwrap(),
            }]
        );
    }

    #[test]
    fn a_upn_reassigned_to_another_id_blocks_the_relationship() {
        let pinned = observed(1);
        let reassigned = observed(2);
        let mut request = reference(&pinned);
        request.user_principal_name = reassigned.user_principal_name.clone();
        let resolution = PrincipalResolution {
            tenant_id: tenant(1),
            reference: request.clone(),
            by_id: Some(pinned.clone()),
            by_upn: Some(reassigned.clone()),
        };
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Exact,
                vec![request.clone()],
            )]),
            &[snapshot(vec![observed(3)])],
            &[resolution],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(plan.change_count(), 0);
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::PrincipalResolutionFailed {
                relationship: GroupRelationship::Members,
                reference: request,
                error: PrincipalResolutionError::UpnIdMismatch {
                    upn: reassigned.user_principal_name.unwrap(),
                    pinned_id: pinned.id,
                    observed_id: reassigned.id,
                },
            }]
        );
    }

    #[test]
    fn a_lookup_result_for_a_different_upn_cannot_satisfy_the_requested_selector() {
        let person = observed(1);
        let request = DesiredPrincipal {
            id: Some(person.id),
            user_principal_name: Some("different@example.invalid".parse().unwrap()),
            display_name: None,
        };
        let lookup = PrincipalResolution {
            tenant_id: tenant(1),
            reference: request.clone(),
            by_id: Some(person.clone()),
            by_upn: Some(person.clone()),
        };
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Additive,
                vec![request.clone()],
            )]),
            &[snapshot(Vec::new())],
            &[lookup],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::PrincipalResolutionFailed {
                relationship: GroupRelationship::Members,
                reference: request.clone(),
                error: PrincipalResolutionError::UpnMismatch {
                    upn: request.user_principal_name.unwrap(),
                    principal_id: person.id,
                    observed: person.user_principal_name,
                },
            }]
        );
        assert_eq!(plan.change_count(), 0);
    }

    #[test]
    fn a_lookup_result_for_a_different_id_cannot_satisfy_a_pinned_reference() {
        let pinned = observed(1);
        let different = observed(2);
        let mut lookup = resolution(&pinned);
        lookup.by_id = Some(different.clone());
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Exact,
                vec![reference(&pinned)],
            )]),
            &[snapshot(vec![observed(3)])],
            &[lookup],
        )
        .unwrap();

        assert_eq!(plan.change_count(), 0);
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::PrincipalResolutionFailed {
                relationship: GroupRelationship::Members,
                reference: reference(&pinned),
                error: PrincipalResolutionError::PinnedIdMismatch {
                    expected: pinned.id,
                    observed: different.id,
                },
            }]
        );
    }

    #[test]
    fn duplicate_upn_lookup_records_block_changes_before_selecting_a_result() {
        let person = observed(1);
        let request = DesiredPrincipal {
            id: None,
            user_principal_name: person.user_principal_name.clone(),
            display_name: None,
        };
        let lookup = PrincipalResolution {
            tenant_id: tenant(1),
            reference: request.clone(),
            by_id: None,
            by_upn: Some(person),
        };
        let mut conflicting = lookup.clone();
        conflicting.by_upn = Some(observed(2));
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Exact,
                vec![request.clone()],
            )]),
            &[snapshot(vec![observed(3)])],
            &[lookup, conflicting],
        )
        .unwrap();

        assert_eq!(plan.change_count(), 0);
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::PrincipalResolutionFailed {
                relationship: GroupRelationship::Members,
                reference: request,
                error: PrincipalResolutionError::DuplicateLookupRecords {
                    tenant_id: tenant(1),
                },
            }]
        );
    }

    #[test]
    fn id_and_upn_aliases_resolving_to_one_person_are_rejected_as_duplicates() {
        let person = observed(1);
        let by_upn = DesiredPrincipal {
            id: None,
            user_principal_name: person.user_principal_name.clone(),
            display_name: None,
        };
        let upn_resolution = PrincipalResolution {
            tenant_id: tenant(1),
            reference: by_upn.clone(),
            by_id: None,
            by_upn: Some(person.clone()),
        };
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Additive,
                vec![reference(&person), by_upn],
            )]),
            &[snapshot(Vec::new())],
            &[resolution(&person), upn_resolution],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::DuplicateDesiredPrincipal {
                relationship: GroupRelationship::Members,
                principal_id: person.id,
            }]
        );
    }

    #[test]
    fn lookup_results_from_another_tenant_never_satisfy_a_reference() {
        let person = observed(1);
        let mut other_tenant = resolution(&person);
        other_tenant.tenant_id = tenant(2);
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Additive,
                vec![reference(&person)],
            )]),
            &[snapshot(Vec::new())],
            &[other_tenant],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::PrincipalResolutionFailed {
                relationship: GroupRelationship::Members,
                reference: reference(&person),
                error: PrincipalResolutionError::LookupNotCompleted {
                    tenant_id: tenant(1),
                },
            }]
        );
        assert_eq!(plan.change_count(), 0);
    }

    #[test]
    fn snapshots_from_another_tenant_do_not_establish_the_groups_existence() {
        let mut other_tenant = snapshot(Vec::new());
        other_tenant.tenant_id = tenant(2);
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(ReconciliationMode::Exact, Vec::new())]),
            &[other_tenant],
            &[],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert!(plan.groups[0].observed_display_name.is_none());
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::MissingSnapshot {
                tenant_id: tenant(1),
                group_id: "20000000-0000-0000-0000-000000000001".parse().unwrap(),
            }]
        );
    }

    #[test]
    fn desired_principals_not_found_do_not_turn_into_an_empty_exact_target() {
        let missing = observed(2);
        let mut lookup = resolution(&missing);
        lookup.by_id = None;
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![desired(
                ReconciliationMode::Exact,
                vec![reference(&missing)],
            )]),
            &[snapshot(vec![observed(1)])],
            &[lookup],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(plan.change_count(), 0);
    }

    #[test]
    fn a_group_can_be_a_member_but_not_an_owner() {
        let mut nested_group = observed(1);
        nested_group.kind = EntraDirectoryObjectType::Group;
        nested_group.user_principal_name = None;
        let mut request = desired(ReconciliationMode::Additive, vec![reference(&nested_group)]);
        request.owners_mode = ReconciliationMode::Additive;
        request.owners = request.members.clone();
        let plan = DesireDocumentAchievementPlan::try_new(
            &document(vec![request]),
            &[snapshot(Vec::new())],
            &[resolution(&nested_group)],
        )
        .unwrap();
        assert!(plan.has_blockers());
        assert_eq!(
            plan.groups[0].members.as_ref().unwrap().add,
            vec![nested_group.clone()]
        );
        assert!(plan.groups[0].owners.as_ref().unwrap().add.is_empty());
        assert_eq!(
            plan.groups[0].blockers,
            vec![GroupPlanBlocker::GroupCannotBeOwner {
                principal_id: nested_group.id,
            }]
        );
    }

    #[test]
    fn requested_devices_and_contacts_block_both_relationships() {
        for kind in [
            EntraDirectoryObjectType::Device,
            EntraDirectoryObjectType::OrganizationalContact,
        ] {
            let mut unsupported = observed(1);
            unsupported.kind = kind;
            unsupported.display_name = None;
            unsupported.user_principal_name = None;
            let mut request = desired(ReconciliationMode::Exact, vec![reference(&unsupported)]);
            request.owners_mode = ReconciliationMode::Additive;
            request.owners = request.members.clone();
            let plan = DesireDocumentAchievementPlan::try_new(
                &document(vec![request]),
                &[snapshot(vec![observed(2)])],
                &[resolution(&unsupported)],
            )
            .unwrap();

            assert_eq!(plan.change_count(), 0);
            assert_eq!(
                plan.groups[0].blockers,
                [GroupRelationship::Members, GroupRelationship::Owners].map(|relationship| {
                    GroupPlanBlocker::UnsupportedPrincipalKind {
                        relationship,
                        principal_id: unsupported.id,
                        actual_kind: kind,
                    }
                })
            );
        }
    }

    #[test]
    fn an_unsupported_baseline_blocks_only_its_relationship_without_hiding_the_object() {
        for kind in [
            EntraDirectoryObjectType::Device,
            EntraDirectoryObjectType::OrganizationalContact,
        ] {
            let mut unsupported = observed(1);
            unsupported.kind = kind;
            unsupported.display_name = None;
            unsupported.user_principal_name = None;
            let member = observed(2);
            let owner = observed(3);
            let mut request = desired(ReconciliationMode::Exact, vec![reference(&member)]);
            request.owners_mode = ReconciliationMode::Additive;
            request.owners = vec![reference(&owner)];
            let plan = DesireDocumentAchievementPlan::try_new(
                &document(vec![request]),
                &[snapshot(vec![unsupported.clone()])],
                &[resolution(&member), resolution(&owner)],
            )
            .unwrap();

            let members = plan.groups[0].members.as_ref().unwrap();
            assert_eq!(members.observed, vec![unsupported.clone()]);
            assert!(members.add.is_empty());
            assert!(members.remove.is_empty());
            assert_eq!(plan.groups[0].owners.as_ref().unwrap().add, vec![owner]);
            assert_eq!(plan.change_count(), 1);
            assert_eq!(
                plan.groups[0].blockers,
                vec![GroupPlanBlocker::UnsupportedPrincipalKind {
                    relationship: GroupRelationship::Members,
                    principal_id: unsupported.id,
                    actual_kind: kind,
                }]
            );
        }
    }

    #[test]
    fn changes_are_sorted_independently_of_desired_and_observed_order() {
        let one = observed(1);
        let two = observed(2);
        let three = observed(3);
        let four = observed(4);
        let requests = [
            document(vec![desired(
                ReconciliationMode::Exact,
                vec![reference(&four), reference(&two)],
            )]),
            document(vec![desired(
                ReconciliationMode::Exact,
                vec![reference(&two), reference(&four)],
            )]),
        ];
        let baselines = [
            snapshot(vec![three.clone(), one.clone()]),
            snapshot(vec![one, three]),
        ];
        let resolutions = [resolution(&two), resolution(&four)];
        let first =
            DesireDocumentAchievementPlan::try_new(&requests[0], &baselines[..1], &resolutions)
                .unwrap();
        let second =
            DesireDocumentAchievementPlan::try_new(&requests[1], &baselines[1..], &resolutions)
                .unwrap();
        assert_eq!(first.groups, second.groups);
    }
}
