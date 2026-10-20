use super::DesireDocument;
use crate::domain::DesiredGroup;
use crate::domain::DesiredPrincipal;
use crate::domain::GroupSnapshot;
use crate::domain::ObservedPrincipal;
use crate::domain::PrincipalResolution;
use crate::domain::ReconciliationMode;
use crate::planning::DesireDocumentAchievementPlan;
use crate::planning::GroupPlanBlocker;
use crate::tenant_preflight::preflight_tenants;
use cloud_terrastodon::app::AuthContext;
use cloud_terrastodon::app::CancellationScope;
use cloud_terrastodon::azure::fetch_entra_directory_objects_by_ids;
use cloud_terrastodon::azure::fetch_entra_user;
use cloud_terrastodon::azure::fetch_group;
use cloud_terrastodon::azure::fetch_group_members;
use cloud_terrastodon::azure::fetch_group_owners;
use cloud_terrastodon::azure_types::AzureTenantId;
use cloud_terrastodon::azure_types::EntraGroup;
use cloud_terrastodon::azure_types::EntraUser;
use cloud_terrastodon::azure_types::uuid::Uuid;
use cloud_terrastodon::command::CacheInvalidatableIntoFuture;
use cloud_terrastodon::credentials::AzureRestResource;
use cloud_terrastodon::credentials::AzureTenantAuthContext;
use cloud_terrastodon::rest::RestHttpError;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use std::collections::BTreeMap;

impl DesireDocument {
    /// Read fresh state and produce a review artifact without changing the directory.
    ///
    /// This consumes the validated document to apply any requested tenant selection.
    /// File loading and local validation belong to the caller.
    ///
    /// The entire selected tenant set is admitted before directory reads. Graph v1.0
    /// relationship inventories have documented service-principal omissions, so this
    /// adapter never marks them complete for exact reconciliation.
    pub async fn try_plan(
        self,
        only_tenant: Option<AzureTenantId>,
        auth: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<DesireDocumentAchievementPlan> {
        let document_tenant_ids = {
            let mut tenants: Vec<_> = self.groups().iter().map(|group| group.tenant_id).collect();
            tenants.sort_by_key(ToString::to_string);
            tenants.dedup();
            tenants
        };
        let desired = match only_tenant {
            Some(tenant) => self.select_tenant(tenant)?,
            None => self,
        };
        let bindings = preflight_tenants(desired.groups(), auth)?;

        // A successful local binding is not proof that a credential can authenticate
        // to this tenant/resource. Complete these exchanges for every selected tenant
        // before reading the first group. No access token enters the plan artifact.
        for binding in bindings.values() {
            cancellation.bail_if_cancelled()?;
            binding
                .fetch_bearer_access_token(AzureRestResource::MicrosoftGraph, cancellation)
                .await
                .wrap_err_with(|| {
                    format!(
                        "Authenticating to Microsoft Graph in tenant {}",
                        binding.tenant_id
                    )
                })?;
        }

        let resolutions = resolve_principals(desired.groups(), &bindings, cancellation).await?;
        let mut snapshots = Vec::with_capacity(desired.groups().len());
        let mut eligibility = BTreeMap::new();
        for group in desired.groups() {
            cancellation.bail_if_cancelled()?;
            let binding = bindings.get(&group.tenant_id).ok_or_else(|| {
                eyre::eyre!(
                    "Tenant {} was not admitted by authentication preflight",
                    group.tenant_id
                )
            })?;
            let observed = fetch_group(group.group_id, binding, cancellation)
                .with_invalidation(true)
                .await
                .wrap_err_with(|| {
                    format!(
                        "Reading group {} in tenant {}",
                        group.group_id, group.tenant_id
                    )
                })?;
            ensure!(
                observed.id == group.group_id,
                "Directory response identified a different group"
            );
            eligibility.insert(
                (group.tenant_id.to_string(), group.group_id.to_string()),
                eligibility_blockers(&observed),
            );
            let members = if group.members_mode != ReconciliationMode::Ignore {
                fetch_group_members(group.group_id, binding, cancellation)
                    .with_invalidation(true)
                    .await?
                    .into_iter()
                    .map(ObservedPrincipal::from)
                    .collect()
            } else {
                Vec::new()
            };
            let owners = if group.owners_mode != ReconciliationMode::Ignore {
                fetch_group_owners(group.group_id, binding, cancellation)
                    .with_invalidation(true)
                    .await?
                    .into_iter()
                    .map(ObservedPrincipal::from)
                    .collect()
            } else {
                Vec::new()
            };
            snapshots.push(GroupSnapshot {
                tenant_id: group.tenant_id,
                group_id: group.group_id,
                display_name: observed.display_name,
                members,
                owners,
                members_complete: false,
                owners_complete: false,
            });
        }

        cancellation.bail_if_cancelled()?;
        let mut plan = DesireDocumentAchievementPlan::try_new(&desired, &snapshots, &resolutions)?;
        plan.selected_tenant_id = only_tenant;
        plan.excluded_tenant_ids = document_tenant_ids
            .iter()
            .copied()
            .filter(|tenant| !bindings.contains_key(tenant))
            .collect();
        plan.document_tenant_ids = document_tenant_ids;
        for group in &mut plan.groups {
            if let Some(blockers) =
                eligibility.remove(&(group.tenant_id.to_string(), group.group_id.to_string()))
                && !blockers.is_empty()
            {
                group.blockers.extend(blockers);
                for relationship in [&mut group.members, &mut group.owners]
                    .into_iter()
                    .flatten()
                {
                    relationship.add.clear();
                    relationship.remove.clear();
                }
                group.normalize_diagnostics();
            }
        }
        Ok(plan)
    }
}

async fn resolve_principals(
    desired: &[DesiredGroup],
    bindings: &BTreeMap<AzureTenantId, AzureTenantAuthContext>,
    cancellation: &CancellationScope,
) -> Result<Vec<PrincipalResolution>> {
    let mut resolutions = Vec::new();
    for binding in bindings.values() {
        cancellation.bail_if_cancelled()?;
        let mut references = BTreeMap::new();
        for group in desired
            .iter()
            .filter(|group| group.tenant_id == binding.tenant_id)
        {
            for reference in group.members.iter().chain(&group.owners) {
                references
                    .entry(reference_key(reference))
                    .or_insert(reference);
            }
        }
        let ids: Vec<Uuid> = references
            .values()
            .filter_map(|reference| reference.id.map(|id| *id.as_ref()))
            .collect();
        let by_id: BTreeMap<_, _> = if ids.is_empty() {
            BTreeMap::new()
        } else {
            fetch_entra_directory_objects_by_ids(ids, binding, cancellation)
                .with_invalidation(true)
                .await?
                .into_iter()
                .map(ObservedPrincipal::from)
                .map(|principal| (principal.id.to_string(), principal))
                .collect()
        };
        let mut by_upn = BTreeMap::new();
        for reference in references.into_values() {
            cancellation.bail_if_cancelled()?;
            let user = if let Some(upn) = &reference.user_principal_name {
                let key = upn.comparison_key();
                if !by_upn.contains_key(&key) {
                    let user = user_lookup_observation(
                        fetch_entra_user(upn, binding, cancellation)
                            .with_invalidation(true)
                            .await,
                    )?;
                    by_upn.insert(key.clone(), user);
                }
                by_upn.get(&key).cloned().flatten()
            } else {
                None
            };
            resolutions.push(PrincipalResolution {
                tenant_id: binding.tenant_id,
                reference: reference.clone(),
                by_id: reference
                    .id
                    .and_then(|id| by_id.get(&id.to_string()).cloned()),
                by_upn: user,
            });
        }
    }
    Ok(resolutions)
}

/// Only an explicit Graph not-found response establishes an absent user.
fn user_lookup_observation(result: Result<EntraUser>) -> Result<Option<ObservedPrincipal>> {
    match result {
        Ok(user) => Ok(Some(user.into())),
        Err(error)
            if error
                .downcast_ref::<RestHttpError>()
                .is_some_and(|http| http.status.as_u16() == 404) =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn reference_key(reference: &DesiredPrincipal) -> (Option<String>, Option<String>) {
    (
        reference.id.map(|id| id.to_string()),
        reference
            .user_principal_name
            .as_ref()
            .map(|upn| upn.comparison_key()),
    )
}

fn eligibility_blockers(group: &EntraGroup) -> Vec<GroupPlanBlocker> {
    let mut blockers = Vec::new();
    if !group.security_enabled || group.mail_enabled != Some(false) {
        blockers.push(GroupPlanBlocker::UnsupportedGroupConfiguration {
            security_enabled: group.security_enabled,
            mail_enabled: group.mail_enabled,
        });
    }
    if group
        .group_types
        .iter()
        .any(|kind| kind == "DynamicMembership")
    {
        blockers.push(GroupPlanBlocker::DynamicMembership);
    }
    if group.on_premises_sync_enabled == Some(true) {
        blockers.push(GroupPlanBlocker::OnPremisesSynchronized);
    }
    match group.is_assignable_to_role {
        Some(true) => blockers.push(GroupPlanBlocker::RoleAssignableGroup),
        None => blockers.push(GroupPlanBlocker::UnknownRoleAssignability),
        Some(false) => {}
    }
    blockers
}

#[cfg(test)]
mod tests {
    use super::user_lookup_observation;
    use cloud_terrastodon::rest::RestHttpError;
    use eyre::Report;
    use std::io;
    use std::io::ErrorKind;

    fn http_failure(status: u16) -> Report {
        Report::new(RestHttpError {
            status: status.try_into().unwrap(),
            reason_phrase: Some("Synthetic lookup response".into()),
        })
        .wrap_err("Reading a user")
    }

    #[test]
    fn only_a_typed_not_found_response_establishes_an_absent_user() {
        assert!(
            user_lookup_observation(Err(http_failure(404)))
                .unwrap()
                .is_none()
        );

        for status in [403, 429] {
            let error = user_lookup_observation(Err(http_failure(status))).unwrap_err();
            assert_eq!(
                error
                    .downcast_ref::<RestHttpError>()
                    .unwrap()
                    .status
                    .as_u16(),
                status
            );
        }

        let transport = Report::new(io::Error::new(
            ErrorKind::ConnectionReset,
            "Synthetic transport failure mentioning 404",
        ))
        .wrap_err("Reading a user");
        let error = user_lookup_observation(Err(transport)).unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            ErrorKind::ConnectionReset
        );

        let decode = facet_json::from_str::<String>("{").unwrap_err();
        let error =
            user_lookup_observation(Err(Report::new(decode).wrap_err("Decoding a user response")))
                .unwrap_err();
        assert!(
            error
                .downcast_ref::<facet_json::DeserializeError>()
                .is_some()
        );
    }
}
