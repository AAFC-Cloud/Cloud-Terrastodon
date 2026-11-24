use crate::app::MyApp;
use crate::portal_tabs::PortalTabs;
use crate::storage_account_inspection::StorageAccountInspection;
use cloud_terrastodon_azure::AzureStorageAccountGetRequest;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Instant;

/// Storage account snapshots shared by the overview and output tabs.
#[derive(Default)]
pub(crate) struct StorageAccountBrowser {
    pub entries: HashMap<AzureStorageAccountId, StorageAccountInspection>,
}

impl StorageAccountBrowser {
    pub fn retain_open(&mut self, tabs: &PortalTabs) {
        // Keep in-flight state through delivery so reopening the tab does not start
        // a second attempt or let an older attempt overwrite a newer result.
        self.entries.retain(|id, entry| {
            tabs.has_storage_account(id) || matches!(entry.response, Loadable::Loading { .. })
        });
    }

    pub fn is_loading(&self) -> bool {
        self.entries
            .values()
            .any(|entry| matches!(entry.response, Loadable::Loading { .. }))
    }
}

pub(crate) fn load_storage_account(app: &mut MyApp, id: &AzureStorageAccountId, refresh: bool) {
    let Some(authentication) = app.ready_azure_authentication() else {
        return;
    };
    let entries = &mut app.azure.storage_account_browser.entries;
    if entries
        .get(id)
        .is_some_and(|entry| matches!(entry.response, Loadable::Loading { .. }))
    {
        return;
    }
    let request = AzureStorageAccountGetRequest {
        authentication,
        storage_account_id: Cow::Owned(id.clone()),
    };
    entries.insert(
        id.clone(),
        StorageAccountInspection {
            request: request.clone(),
            response: Loadable::NotLoaded,
        },
    );

    let account_generation = app.azure.account_generation;
    let entry_id = id.clone();
    let executor = app.executor.clone();
    let cancellation = executor.cancellation().clone();
    let work_cache = app.work_cache.clone();
    let started_at = Instant::now();
    let enqueue_result = LoadableWorkBuilder::<MyApp, _>::new()
        .description("Load storage account")
        .setter(move |app, response| {
            let retain = app.tabs.has_storage_account(&entry_id);
            let Some(azure) = app.azure_content_mut(account_generation) else {
                return;
            };
            let entries = &mut azure.storage_account_browser.entries;
            if !retain {
                entries.remove(&entry_id);
            } else if let Some(entry) = entries.get_mut(&entry_id) {
                entry.response = response;
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
    if let Err(error) = enqueue_result
        && let Some(entry) = app.azure.storage_account_browser.entries.get_mut(id)
    {
        entry.response = Loadable::Failed {
            error: error.into(),
            started_at,
            finished_at: Instant::now(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_resource_output_target::PortalResourceOutputTarget;
    use crate::portal_tab_target::PortalTabTarget;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
    use eyre::Result;

    #[test]
    fn opening_output_shares_the_inspection_and_in_flight_work_outlives_its_tab() -> Result<()> {
        let id: AzureStorageAccountId =
            "/subscriptions/00000000-0000-0000-0000-000000000003/resourceGroups/synthetic/providers/Microsoft.Storage/storageAccounts/syntheticaccount"
                .parse()?;
        let mut browser = StorageAccountBrowser::default();
        let mut tabs = PortalTabs::default();
        let account_tab = PortalTabTarget::StorageAccount(id.clone());
        let output_tab = PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
            format: OutputFormat::Json,
        };
        tabs.open(account_tab.clone());
        tabs.open_kept(output_tab.clone());
        browser.entries.insert(
            id.clone(),
            StorageAccountInspection {
                request: AzureStorageAccountGetRequest {
                    authentication: AzureBearerAuthenticationSelection {
                        account_id: "00000000-0000-0000-0000-000000000001".parse()?,
                        tenant_id: "00000000-0000-0000-0000-000000000002".parse()?,
                        auth_source: AzureBearerAuthSource::AzureCli,
                    },
                    storage_account_id: Cow::Owned(id.clone()),
                },
                response: Loadable::NotLoaded,
            },
        );

        assert_eq!(tabs.close(0), Some(account_tab));
        browser.retain_open(&tabs);
        assert!(browser.entries.contains_key(&id));

        browser.entries.get_mut(&id).unwrap().response = Loadable::Loading {
            started_at: Instant::now(),
        };
        assert_eq!(tabs.close(0), Some(output_tab));
        browser.retain_open(&tabs);
        assert!(browser.entries.contains_key(&id));
        assert!(browser.is_loading());

        browser.entries.get_mut(&id).unwrap().response = Loadable::NotLoaded;
        browser.retain_open(&tabs);
        assert!(!browser.entries.contains_key(&id));
        assert!(!browser.is_loading());
        Ok(())
    }
}
