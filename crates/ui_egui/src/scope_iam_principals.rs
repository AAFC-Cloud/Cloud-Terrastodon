use crate::app::MyApp;
use crate::scope_iam_snapshot::ScopeIamSnapshot;
use crate::scope_role_assignment_row::ScopeRoleAssignmentRow;
use cloud_terrastodon_azure::EntraPrincipal;
use cloud_terrastodon_azure::EntraPrincipalId;
use cloud_terrastodon_azure::EntraPrincipalLookup;
use cloud_terrastodon_azure::EntraPrincipalRequest;
use cloud_terrastodon_command::CancellationErrorExt;
use cloud_terrastodon_command::app_work::AppboundWorkFinishedMessage;
use cloud_terrastodon_command::app_work::FieldUpdaterWorkSuccessMutator;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_command::app_work::Setter;
use cloud_terrastodon_execution::ConcurrentIteratorExt;
use cloud_terrastodon_execution::ExecutableRequest;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem::Discriminant;
use std::mem::discriminant;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

// The request route matters when the inventory has a missing or incorrect principal type.
type PrincipalLookupKey = (EntraPrincipalId, Discriminant<EntraPrincipalLookup>);
type PrincipalLookupResult = (PrincipalLookupKey, Option<Arc<EntraPrincipal>>);

/// Directory lookups shared across every resource IAM view for the current account.
#[derive(Default)]
pub(crate) struct ScopeIamPrincipals {
    resolved: HashMap<PrincipalLookupKey, Option<Arc<EntraPrincipal>>>,
    in_flight: HashMap<PrincipalLookupKey, u64>,
    revision: u64,
}

impl ScopeIamPrincipals {
    pub fn is_loading(&self) -> bool {
        self.pending_count() != 0
    }

    pub fn pending_count(&self) -> usize {
        self.in_flight.len()
    }

    pub fn pending_for(&self, rows: &[ScopeRoleAssignmentRow]) -> usize {
        if self.in_flight.is_empty() {
            return 0;
        }
        rows.iter()
            .filter_map(|row| ScopeRoleAssignmentRow::principal_lookup(&row.assignment))
            .map(lookup_key)
            .filter(|key| self.in_flight.contains_key(key))
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn apply(&self, snapshot: &mut ScopeIamSnapshot) {
        let Ok(rows) = &mut snapshot.assignments else {
            return;
        };
        for row in rows {
            if let Some(lookup) = ScopeRoleAssignmentRow::principal_lookup(&row.assignment)
                && let Some(Some(principal)) = self.resolved.get(&lookup_key(lookup))
            {
                row.apply_principal(principal);
            }
        }
    }

    fn complete(
        &mut self,
        key: PrincipalLookupKey,
        revision: u64,
        result: Option<Arc<EntraPrincipal>>,
    ) -> bool {
        if self.in_flight.get(&key) != Some(&revision) {
            return false;
        }
        self.in_flight.remove(&key);
        let resolved = result.is_some();
        let previous = self.resolved.insert(key, result);
        resolved || previous.is_some_and(|principal| principal.is_some())
    }
}

/// Populate principal labels in the background after the scope's RBAC rows are visible.
pub(crate) fn load_scope_iam_principals(
    app: &mut MyApp,
    snapshot: &ScopeIamSnapshot,
    refresh: bool,
) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let Ok(rows) = &snapshot.assignments else {
        return;
    };
    let principals = &mut app.azure.scope_iam.principals;
    let lookups: HashMap<_, _> = rows
        .iter()
        .filter_map(|row| ScopeRoleAssignmentRow::principal_lookup(&row.assignment))
        .map(|lookup| (lookup_key(lookup), lookup))
        .filter(|(key, _)| {
            refresh
                || !(principals.resolved.contains_key(key)
                    || principals.in_flight.contains_key(key))
        })
        .collect();
    if lookups.is_empty() {
        return;
    }

    principals.revision = principals.revision.wrapping_add(1);
    let revision = principals.revision;
    let keys: Vec<_> = lookups.keys().copied().collect();
    let generation = app.azure.account_generation;
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let sender = app.tx.clone();
    let started_at = Instant::now();
    let apply_result: Setter<MyApp, PrincipalLookupResult> = Arc::new(move |app, result| {
        if let Some(azure) = app.azure_content_mut(generation)
            && let Loadable::Loaded {
                value: (key, principal),
                ..
            } = result
            && azure
                .scope_iam
                .principals
                .complete(key, revision, principal)
        {
            azure.scope_iam.snapshots.clear();
        }
    });
    let queued = LoadableWorkBuilder::<MyApp, ()>::new()
        .description("Resolve IAM principals")
        .setter(move |app, result| {
            let Some(azure) = app.azure_content_mut(generation) else {
                return;
            };
            let principals = &mut azure.scope_iam.principals;
            match result {
                Loadable::Loading { .. } => {
                    principals.in_flight.extend(keys.iter().map(|&key| (key, revision)));
                }
                Loadable::Failed { .. } => {
                    let mut changed = false;
                    for &key in &keys {
                        changed |= principals.complete(key, revision, None);
                    }
                    if changed {
                        azure.scope_iam.snapshots.clear();
                    }
                }
                Loadable::Loaded { .. } | Loadable::NotLoaded => {}
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            lookups
                .into_iter()
                .map_concurrent(
                    &executor,
                    NonZeroUsize::new(8).unwrap_or(NonZeroUsize::MIN),
                    async |child, (key, lookup)| {
                        let result = EntraPrincipalRequest { lookup, authentication }
                            .execute(&child)
                            .with_invalidation(refresh)
                            .await;
                        let principal = match result {
                            Ok(principal) => Some(principal),
                            Err(error) if error.is_cancellation() => return Err(error),
                            Err(error) => {
                                debug!(principal_id = %lookup.object_id(), error = %error, "Could not resolve IAM principal");
                                None
                            }
                        };
                        let update = AppboundWorkFinishedMessage::update_state(
                            FieldUpdaterWorkSuccessMutator {
                                data: (key, principal),
                                setter: Arc::clone(&apply_result),
                                started_at,
                            },
                        );
                        if sender.send(update).is_err() {
                            debug!("UI receiver closed before IAM principal delivery");
                        }
                        eyre::Ok(())
                    },
                )
                .await?;
            Ok(())
        })
        .build()
        .enqueue(&app.work_tracker.clone(), app.tx.clone(), app, &cancellation);
    if let Err(error) = queued
        && !error.is_cancellation()
    {
        debug!(%error, "Could not schedule IAM principal lookups");
    }
}

fn lookup_key(lookup: EntraPrincipalLookup) -> PrincipalLookupKey {
    (
        EntraPrincipalId::from(lookup.object_id()),
        discriminant(&lookup),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn older_lookup_cannot_complete_a_refreshed_principal() -> eyre::Result<()> {
        let id: EntraPrincipalId = "00000000-0000-0000-0000-000000000001".parse()?;
        let key = lookup_key(id.into());
        let mut cache = ScopeIamPrincipals::default();
        cache.in_flight.insert(key, 2);

        assert!(!cache.complete(key, 1, None));
        assert!(cache.is_loading());
        assert!(!cache.resolved.contains_key(&key));

        assert!(!cache.complete(key, 2, None));
        assert!(!cache.is_loading());
        assert!(matches!(cache.resolved.get(&key), Some(None)));
        Ok(())
    }
}
