use crate::app::MyApp;
use crate::portal_tabs::PortalTabs;
use crate::storage_explorer_container_row::StorageExplorerContainerRow;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainer;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerGetRequest;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerListRequest;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::ensure;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Blob container inventory and metadata shared by views of each open storage account.
#[derive(Default)]
pub(crate) struct StorageExplorer {
    pub inventories: HashMap<AzureStorageAccountId, Loadable<Vec<StorageExplorerContainerRow>>>,
    pub containers: HashMap<
        AzureStorageAccountBlobContainerId,
        Loadable<Arc<AzureStorageAccountBlobContainer>>,
    >,
}

impl StorageExplorer {
    pub fn retain_open(&mut self, tabs: &PortalTabs) {
        // Keep in-flight state until delivery so reopening a view shares its request.
        self.inventories.retain(|id, response| {
            tabs.has_storage_account(id) || matches!(response, Loadable::Loading { .. })
        });
        self.containers.retain(|id, response| {
            tabs.has_storage_account(&id.storage_account_id)
                || matches!(response, Loadable::Loading { .. })
        });
    }

    pub fn is_loading(&self) -> bool {
        self.inventories
            .values()
            .any(|response| matches!(response, Loadable::Loading { .. }))
            || self
                .containers
                .values()
                .any(|response| matches!(response, Loadable::Loading { .. }))
    }

    pub fn can_refresh(&self, id: &AzureStorageAccountId) -> bool {
        !matches!(self.inventories.get(id), Some(Loadable::Loading { .. }))
            && !self.containers.iter().any(|(container, response)| {
                container.storage_account_id == *id && matches!(response, Loadable::Loading { .. })
            })
    }

    pub fn can_refresh_container(&self, id: &AzureStorageAccountBlobContainerId) -> bool {
        !matches!(self.containers.get(id), Some(Loadable::Loading { .. }))
            && !matches!(
                self.inventories.get(&id.storage_account_id),
                Some(Loadable::Loading { .. })
            )
    }
}

pub(crate) fn load_storage_explorer(app: &mut MyApp, id: &AzureStorageAccountId, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if !app.azure.storage_explorer.can_refresh(id) {
        return;
    }
    let inspecting: Vec<_> = app
        .azure
        .storage_explorer
        .containers
        .keys()
        .filter(|container| refresh && container.storage_account_id == *id)
        .cloned()
        .collect();
    let generation = app.azure.account_generation;
    let inventory_id = id.clone();
    let request = AzureStorageAccountBlobContainerListRequest {
        authentication,
        storage_account_id: Cow::Owned(id.clone()),
    };
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let queued = LoadableWorkBuilder::<MyApp, Vec<StorageExplorerContainerRow>>::new()
        .description("Load blob containers")
        .setter(move |app, response| {
            let retain = app.tabs.has_storage_account(&inventory_id);
            let Some(azure) = app.azure_content_mut(generation) else {
                return;
            };
            if !retain {
                azure.storage_explorer.inventories.remove(&inventory_id);
                return;
            }
            if refresh && response.as_loaded().is_some() {
                azure
                    .storage_explorer
                    .containers
                    .retain(|id, _| id.storage_account_id != inventory_id);
            }
            azure
                .storage_explorer
                .inventories
                .insert(inventory_id.clone(), response);
        })
        .work(async move |scope| {
            let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
            let storage_account_id = request.storage_account_id.clone();
            let containers = request
                .execute(&executor)
                .with_invalidation(refresh)
                .await?;
            // Refresh details already inspected for this account, preserving every other account.
            for container_id in &inspecting {
                executor.cancellation().bail_if_cancelled()?;
                AzureStorageAccountBlobContainerGetRequest {
                    authentication,
                    container_id: Cow::Borrowed(container_id),
                }
                .invalidate(&executor)
                .await?;
            }
            let mut rows = Vec::with_capacity(containers.len());
            for container in containers {
                executor.cancellation().bail_if_cancelled()?;
                ensure!(
                    &container.id.storage_account_id == storage_account_id.as_ref(),
                    "Azure returned a blob container from a different storage account"
                );
                rows.push(StorageExplorerContainerRow::new(&container));
            }
            Ok(rows)
        })
        .build()
        .enqueue(
            &app.work_tracker.clone(),
            app.tx.clone(),
            app,
            &cancellation,
        );
    if let Err(error) = queued {
        app.azure.storage_explorer.inventories.insert(
            id.clone(),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}

pub(crate) fn load_storage_explorer_container(
    app: &mut MyApp,
    id: &AzureStorageAccountBlobContainerId,
    refresh: bool,
) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    if !app.azure.storage_explorer.can_refresh_container(id) {
        return;
    }
    let generation = app.azure.account_generation;
    let container_id = id.clone();
    let request = AzureStorageAccountBlobContainerGetRequest {
        authentication,
        container_id: Cow::Owned(id.clone()),
    };
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let queued = LoadableWorkBuilder::<MyApp, Arc<AzureStorageAccountBlobContainer>>::new()
        .description("Load blob container properties")
        .setter(move |app, response| {
            let retain = app
                .tabs
                .has_storage_account(&container_id.storage_account_id);
            let Some(azure) = app.azure_content_mut(generation) else {
                return;
            };
            if retain {
                azure
                    .storage_explorer
                    .containers
                    .insert(container_id.clone(), response);
            } else {
                azure.storage_explorer.containers.remove(&container_id);
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
        app.azure.storage_explorer.containers.insert(
            id.clone(),
            Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_tab_target::PortalTabTarget;

    #[test]
    fn storage_container_tab_retains_its_inventory_without_an_account_tab() -> eyre::Result<()> {
        let id: AzureStorageAccountBlobContainerId = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic/providers/Microsoft.Storage",
            "/storageAccounts/syntheticaccount/blobServices/default/containers/example",
        )
        .parse()?;
        let mut explorer = StorageExplorer::default();
        explorer
            .inventories
            .insert(id.storage_account_id.clone(), Loadable::NotLoaded);

        let mut tabs = PortalTabs::default();
        tabs.open(PortalTabTarget::StorageContainer(id.clone()));
        explorer.retain_open(&tabs);
        assert!(explorer.inventories.contains_key(&id.storage_account_id));

        tabs.close(0);
        explorer.retain_open(&tabs);
        assert!(explorer.inventories.is_empty());
        Ok(())
    }
}
