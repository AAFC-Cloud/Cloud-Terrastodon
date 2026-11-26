use crate::app::MyApp;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_resource_scope::network_security_group_id;
use crate::scope_tags_editor::ScopeTagsEditor;
use crate::storage_account_browser::load_storage_account;
use cloud_terrastodon_azure::AzureResourceTags;
use cloud_terrastodon_azure::AzureResourceTagsGetRequest;
use cloud_terrastodon_azure::AzureResourceTagsPatchOperation;
use cloud_terrastodon_azure::AzureResourceTagsUpdateRequest;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceTagsId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::ensure;
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

/// Tag editors and the authoritative write responses pending in inventory snapshots.
#[derive(Default)]
pub(crate) struct ScopeTagsBrowser {
    pub entries: HashMap<PortalResourceScope, ScopeTagsEditor>,
    inventory_updates: HashMap<PortalResourceScope, Arc<AzureResourceTags>>,
    overview_updates: HashMap<PortalResourceScope, Arc<AzureResourceTags>>,
}

impl ScopeTagsBrowser {
    pub fn is_loading(&self) -> bool {
        self.entries.values().any(ScopeTagsEditor::is_loading)
    }

    pub fn can_refresh(&self, scope: &PortalResourceScope) -> bool {
        supports_tags(scope)
            && self
                .entries
                .get(scope)
                .is_none_or(ScopeTagsEditor::can_refresh)
    }
}

/// [Azure tagging](https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/tag-resources#tag-usage-and-recommendations)
/// is available on subscriptions, resource groups, and supporting resource types.
/// Azure reports the provider-specific restrictions when a resource is selected.
pub(crate) fn supports_tags(scope: &PortalResourceScope) -> bool {
    match scope {
        PortalResourceScope::ManagementGroup(_) => false,
        PortalResourceScope::Resource(id) => !matches!(
            id.as_ref(),
            ScopeImpl::ManagementGroup(_) | ScopeImpl::ResourceTags(_)
        ),
        _ => true,
    }
}

pub(crate) fn load_scope_tags(app: &mut MyApp, scope: &PortalResourceScope, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if !app.azure.scope_tags.can_refresh(scope) {
        return;
    }
    let request = AzureResourceTagsGetRequest {
        authentication,
        id: Cow::Owned(ResourceTagsId::from_scope(&scope.arm_scope())),
    };
    let generation = app.azure.account_generation;
    let editor_scope = scope.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let queued = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load resource tags")
        .setter(move |app, source| {
            let Some(azure) = app.azure_content_mut(generation) else {
                return;
            };
            let editor = azure
                .scope_tags
                .entries
                .entry(editor_scope.clone())
                .or_default();
            editor.source = source;
            if editor.source.as_loaded().is_some() {
                editor.reset_draft();
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            request.execute(&executor).with_invalidation(refresh).await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = queued {
        app.azure
            .scope_tags
            .entries
            .entry(scope.clone())
            .or_default()
            .source = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

pub(crate) fn apply_scope_tags(app: &mut MyApp, scope: &PortalResourceScope) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let Some(editor) = app.azure.scope_tags.entries.get(scope) else {
        return;
    };
    let Some(source) = editor.source.as_loaded().map(Arc::clone) else {
        return;
    };
    if editor.is_loading() || !editor.has_changes() || editor.draft_error().is_some() {
        return;
    }
    let id = ResourceTagsId::from_scope(&scope.arm_scope());
    let tags = editor.draft_tags();
    let generation = app.azure.account_generation;
    let editor_scope = scope.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let queued = LoadableWorkBuilder::<MyApp, AzureResourceTags>::new()
        .description("Update resource tags")
        .setter(move |app, write| {
            let Some(azure) = app.azure_content_mut(generation) else {
                return;
            };
            let Some(editor) = azure.scope_tags.entries.get_mut(&editor_scope) else {
                return;
            };
            if let Loadable::Loaded { value, started_at, finished_at } = write {
                let tags = Arc::new(value);
                editor.source = Loadable::Loaded {
                    value: Arc::clone(&tags),
                    started_at,
                    finished_at,
                };
                editor.reset_draft();
                editor.write = Loadable::Loaded { value: (), started_at, finished_at };
                azure.scope_tags.inventory_updates.insert(editor_scope.clone(), Arc::clone(&tags));
                if match &editor_scope {
                    PortalResourceScope::Resource(scope) => network_security_group_id(scope).is_some(),
                    _ => true,
                } {
                    azure.scope_tags.overview_updates.insert(editor_scope.clone(), tags);
                }
            } else {
                editor.write = write.map(|_| ());
            }
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            // Replace is one ARM operation. Refresh the baseline before submitting
            // it so a draft cannot overwrite changes seen since the editor loaded.
            let current = AzureResourceTagsGetRequest { authentication, id: Cow::Borrowed(&id) }
                .execute(&executor)
                .with_invalidation(true)
                .await?;
            ensure!(
                current.properties.tags == source.properties.tags,
                "Azure tags changed since this editor loaded them. Discard the draft and refresh before applying another change."
            );
            AzureResourceTagsUpdateRequest {
                authentication,
                id: Cow::Owned(id),
                operation: AzureResourceTagsPatchOperation::Replace,
                tags,
            }
            .execute(&executor)
            .await
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = queued
        && let Some(editor) = app.azure.scope_tags.entries.get_mut(scope)
    {
        editor.write = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

/// Publish confirmed tags after in-flight inventory reads finish. Azure Resource
/// Graph can lag the direct ARM tag write, so preserve its returned tag values.
pub(crate) fn refresh_tagged_inventories(app: &mut MyApp) {
    if let Some(resources) = app.azure.resources.as_loaded_mut() {
        for (scope, updated) in app.azure.scope_tags.inventory_updates.drain() {
            let id = scope.arm_scope();
            let position = resources
                .iter()
                .position(|resource| resource.id == id)
                .or_else(|| {
                    let expanded = id.expanded_form();
                    resources.iter().position(|resource| {
                        resource.id.expanded_form().eq_ignore_ascii_case(&expanded)
                    })
                });
            if let Some(index) = position {
                let resource = &resources[index];
                let replacement = Resource {
                    id: resource.id.clone(),
                    kind: resource.kind.clone(),
                    name: resource.name.clone(),
                    location: resource.location.clone(),
                    properties: resource.properties.clone(),
                    tags: updated.properties.tags.clone(),
                };
                Arc::make_mut(resources)[index] = Arc::new(replacement);
            }
        }
    }

    let mut overviews = std::mem::take(&mut app.azure.scope_tags.overview_updates);
    overviews.retain(|scope, updated| match scope {
        PortalResourceScope::ResourceGroup(id) => {
            let Some(groups) = app.azure.resource_groups.as_loaded_mut() else {
                return true;
            };
            if groups.contains_key(id) {
                // Rebuild the map only when another owner still holds the view.
                if Rc::get_mut(groups).is_none() {
                    *groups = Rc::new(groups.values().cloned().collect::<Vec<_>>().into());
                }
                if let Some(group) = Rc::get_mut(groups).and_then(|groups| groups.get_mut(id)) {
                    Arc::make_mut(group)
                        .tags
                        .clone_from(&updated.properties.tags);
                }
            }
            false
        }
        PortalResourceScope::Subscription(id) => {
            let Some(subscriptions) = app.azure.subscriptions.as_loaded_mut() else {
                return true;
            };
            if let Some(subscription) = Rc::make_mut(subscriptions)
                .iter_mut()
                .find(|item| item.id == *id)
            {
                Arc::make_mut(subscription)
                    .tags
                    .clone_from(&updated.properties.tags);
            }
            false
        }
        PortalResourceScope::StorageAccount(id) => {
            if let Some(inspection) = app.azure.storage_account_browser.entries.get(id) {
                if matches!(inspection.response, Loadable::Loading { .. }) {
                    return true;
                }
                load_storage_account(app, id, false);
            }
            false
        }
        PortalResourceScope::Resource(scope) => {
            let Some(id) = network_security_group_id(scope) else {
                return false;
            };
            match app.azure.network_security_groups.get_mut(id) {
                Some(Loadable::Loading { .. }) => true,
                Some(Loadable::Loaded { value, .. }) => {
                    Arc::make_mut(value)
                        .tags
                        .clone_from(&updated.properties.tags);
                    false
                }
                _ => false,
            }
        }
        _ => false,
    });
    app.azure.scope_tags.overview_updates = overviews;
}
