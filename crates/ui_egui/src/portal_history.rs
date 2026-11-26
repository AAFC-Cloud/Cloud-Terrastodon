use crate::portal_history_entry::PortalHistoryEntry;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use eframe::Storage;
use std::cmp::Reverse;
use std::collections::VecDeque;

const STORAGE_KEY: &str = "cloud_terrastodon.portal_history.v1";
const MAX_VISITS: usize = 1024;

/// The window's recent navigation across all tabs, including closed tabs.
#[derive(Default)]
pub(crate) struct PortalHistory {
    entries: VecDeque<PortalHistoryEntry>,
}

impl PortalHistory {
    pub fn restore(storage: Option<&dyn Storage>) -> Self {
        let Some(data) = storage.and_then(|storage| storage.get_string(STORAGE_KEY)) else {
            return Self::default();
        };
        match facet_json::from_str::<Vec<PortalHistoryEntry>>(&data) {
            Ok(entries) => Self {
                entries: entries
                    .into_iter()
                    .take(MAX_VISITS)
                    .map(|entry| PortalHistoryEntry {
                        target: entry.target.into_current(),
                        ..entry
                    })
                    .collect(),
            },
            Err(error) => {
                tracing::warn!(%error, "Could not restore portal history");
                Self::default()
            }
        }
    }

    pub fn save(&self, storage: &mut dyn Storage) {
        let entries: Vec<_> = self.entries.iter().collect();
        match facet_json::to_string(&entries) {
            Ok(data) => storage.set_string(STORAGE_KEY, data),
            Err(error) => tracing::warn!(%error, "Could not save portal history"),
        }
    }

    pub fn record(&mut self, entry: PortalHistoryEntry) {
        self.entries.push_front(entry);
        self.entries.truncate(MAX_VISITS);
    }

    pub fn visible(
        &self,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> Vec<&PortalHistoryEntry> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry.is_visible(authentication))
            .collect();
        entries.sort_by_key(|entry| Reverse(entry.visited_at));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_page::PortalPage;
    use crate::portal_tab_target::PortalTabTarget;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use jiff::Timestamp;
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestStorage(HashMap<String, String>);

    impl Storage for TestStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.to_owned(), value);
        }

        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }

        fn flush(&mut self) {}
    }

    #[test]
    fn former_global_devops_projects_history_restores_to_organizations() -> eyre::Result<()> {
        let target = PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations);
        let mut history = PortalHistory::default();
        history.record(PortalHistoryEntry {
            target: target.clone(),
            title: "DevOps projects".to_owned(),
            visited_at: Timestamp::UNIX_EPOCH,
            resource_kind: None,
            authentication: None,
        });
        let mut storage = TestStorage::default();
        history.save(&mut storage);
        let serialized = storage.get_string(STORAGE_KEY).unwrap();
        assert!(serialized.contains("AzureDevOpsOrganizations"));
        storage.set_string(
            STORAGE_KEY,
            serialized.replace("AzureDevOpsOrganizations", "AzureDevOpsProjects"),
        );
        assert_eq!(
            PortalHistory::restore(Some(&storage)).visible(None)[0].target,
            target
        );
        Ok(())
    }

    #[test]
    fn former_project_history_restores_to_the_project_overview() -> eyre::Result<()> {
        use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
        use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;

        let organization = "synthetic".parse()?;
        let project = "00000000-0000-0000-0000-000000000001".parse()?;
        let mut history = PortalHistory::default();
        history.record(PortalHistoryEntry {
            target: PortalAzureDevOpsOrganizationView::Project(project).target(&organization),
            title: "Synthetic project".to_owned(),
            visited_at: Timestamp::UNIX_EPOCH,
            resource_kind: None,
            authentication: None,
        });
        let mut storage = TestStorage::default();
        history.save(&mut storage);
        let history = PortalHistory::restore(Some(&storage));
        let entries = history.visible(None);
        assert_eq!(entries[0].title, "Synthetic project");
        assert_eq!(
            entries[0].target,
            PortalAzureDevOpsProjectView::Overview.target(&organization, project)
        );
        Ok(())
    }

    #[test]
    fn restored_visits_keep_their_dates_and_respect_the_active_account() -> eyre::Result<()> {
        let first_account = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000001".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000002".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
        let second_account = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000003".parse()?,
            ..first_account
        };
        let subscription =
            PortalTabTarget::Subscription("00000000-0000-0000-0000-000000000004".parse()?);
        let about = PortalTabTarget::Page(PortalPage::About);
        let path = PortalTabTarget::LocalPath("project/main.tf".into());
        let mut history = PortalHistory::default();
        for (seconds, target) in [(172800, &about), (86400, &subscription), (0, &path)] {
            history.record(PortalHistoryEntry {
                target: target.clone(),
                title: target.title().into_owned(),
                visited_at: Timestamp::from_second(seconds).unwrap(),
                resource_kind: None,
                authentication: Some(first_account),
            });
        }
        let mut storage = TestStorage::default();
        history.save(&mut storage);
        let history = PortalHistory::restore(Some(&storage));
        let matching = history.visible(Some(first_account));
        assert_eq!(
            matching
                .iter()
                .map(|entry| &entry.target)
                .collect::<Vec<_>>(),
            [&about, &subscription, &path]
        );
        assert_eq!(
            matching
                .iter()
                .map(|entry| entry.visited_at.as_second())
                .collect::<Vec<_>>(),
            [172800, 86400, 0]
        );
        let other = history.visible(Some(second_account));
        assert_eq!(
            other.iter().map(|entry| &entry.target).collect::<Vec<_>>(),
            [&about, &path]
        );
        Ok(())
    }
}
