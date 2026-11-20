use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_tab::PortalTab;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::AzureStorageAccountId;
use std::path::Path;

/// Owns the open tabs and selection. Closing the last tab opens a fresh Home preview.
pub(crate) struct PortalTabs {
    tabs: Vec<PortalTab>,
    active: usize,
}

impl Default for PortalTabs {
    fn default() -> Self {
        Self {
            tabs: vec![PortalTab::new(
                PortalTabTarget::Page(PortalPage::Home),
                true,
            )],
            active: 0,
        }
    }
}

impl PortalTabs {
    pub fn active(&self) -> &PortalTabTarget {
        self.tabs[self.active].target()
    }

    pub fn history(&self) -> (&[PortalTabTarget], usize) {
        self.tabs[self.active].history()
    }

    pub fn back(&mut self) -> bool {
        self.history()
            .1
            .checked_sub(1)
            .is_some_and(|position| self.go_to(position))
    }

    pub fn forward(&mut self) -> bool {
        self.go_to(self.history().1 + 1)
    }

    /// Move to a visit in this tab, even if another tab has the same destination.
    pub fn go_to(&mut self, position: usize) -> bool {
        self.tabs[self.active].go_to(position)
    }

    /// Record a visit in the current tab, even when the tab is kept open.
    pub fn navigate_current(&mut self, target: PortalTabTarget) -> bool {
        self.tabs[self.active].navigate(target)
    }

    pub fn replace_current(&mut self, target: PortalTabTarget) {
        self.tabs[self.active].replace_current(target);
    }

    pub fn contains(&self, target: &PortalTabTarget) -> bool {
        self.tabs.iter().any(|tab| tab.target() == target)
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &PortalTab, bool)> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| (index, tab, index == self.active))
    }

    pub fn has_local_path(&self, path: &Path) -> bool {
        self.tabs
            .iter()
            .any(|tab| matches!(tab.target(), PortalTabTarget::LocalPath(other) if other == path))
    }

    pub fn has_storage_account(&self, id: &AzureStorageAccountId) -> bool {
        self.tabs.iter().any(|tab| {
            matches!(
                tab.target(),
                PortalTabTarget::StorageAccount(other)
                    | PortalTabTarget::ResourceOutput {
                        resource: PortalResourceOutputTarget::StorageAccount(other),
                        ..
                    }
                    if other == id
            )
        })
    }

    /// Start a separate Home preview, even if a Home tab is already open.
    pub fn open_new_tab(&mut self) {
        self.push(PortalTab::new(
            PortalTabTarget::Page(PortalPage::Home),
            true,
        ));
    }

    /// Focus an open destination or navigate using the active tab's preview policy.
    pub fn open(&mut self, target: PortalTabTarget) {
        if let Some(index) = self.find_target(&target) {
            self.active = index;
        } else if self.tabs[self.active].preview {
            self.tabs[self.active].navigate(target);
        } else {
            let mut tab = PortalTab::new(self.active().clone(), true);
            tab.navigate(target);
            self.push(tab);
        }
    }

    pub fn open_kept(&mut self, target: PortalTabTarget) {
        if let Some(index) = self.find_target(&target) {
            self.tabs[index].preview = false;
            self.active = index;
        } else {
            self.push(PortalTab::new(target, false));
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
        }
    }

    pub fn keep_open(&mut self, index: usize) {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.preview = false;
        }
    }

    pub fn clear_azure_details(&mut self) {
        let previous_active = self.active;
        let mut previous_index = 0;
        let mut retained = 0;
        self.active = 0;
        self.tabs.retain_mut(|tab| {
            let keep = tab.retain_after_account_change();
            if keep && previous_index == previous_active {
                self.active = retained;
            }
            retained += usize::from(keep);
            previous_index += 1;
            keep
        });
        if self.tabs.is_empty() {
            *self = Self::default();
        }
    }

    pub fn close(&mut self, index: usize) -> Option<PortalTabTarget> {
        if index >= self.tabs.len() {
            return None;
        }
        let removed = self.tabs.remove(index);
        if self.tabs.is_empty() {
            *self = Self::default();
        } else if self.active > index {
            self.active -= 1;
        } else if self.active == index {
            self.active = index.min(self.tabs.len() - 1);
        }
        Some(removed.into_target())
    }

    fn find_target(&self, target: &PortalTabTarget) -> Option<usize> {
        if self.active() == target {
            Some(self.active)
        } else {
            self.tabs.iter().position(|tab| tab.target() == target)
        }
    }

    fn push(&mut self, tab: PortalTab) {
        self.active = self.tabs.len();
        self.tabs.push(tab);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_resource_scope::PortalResourceScope;
    use crate::portal_scope_view::PortalScopeView;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure::AzureResourceGroupId;

    #[test]
    fn opening_an_existing_resource_switches_tabs_and_preserves_history() -> eyre::Result<()> {
        let mut tabs = PortalTabs::default();
        let group = PortalTabTarget::ResourceGroup(AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?);
        let home = PortalTabTarget::Page(PortalPage::Home);
        let inventory = PortalTabTarget::Page(PortalPage::Resources);
        tabs.open(group.clone());
        tabs.keep_open(0);
        tabs.open_kept(inventory.clone());
        tabs.open(group.clone());

        assert_eq!(tabs.tabs.len(), 2);
        assert_eq!(tabs.active, 0);
        assert_eq!(tabs.active(), &group);
        assert_eq!(tabs.history(), (&[home, group][..], 1));
        tabs.select(1);
        assert_eq!(tabs.history(), (std::slice::from_ref(&inventory), 0));
        Ok(())
    }

    #[test]
    fn navigation_replaces_preview_and_leaves_kept_tabs_open() {
        use PortalPage::ResourceGroups;
        use PortalPage::Resources;
        use PortalPage::Subscriptions;
        use PortalTabTarget::Page;

        let mut tabs = PortalTabs::default();
        tabs.open(Page(Resources));
        tabs.open(Page(Subscriptions));
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active(), &Page(Subscriptions));

        tabs.keep_open(0);
        tabs.open(Page(ResourceGroups));
        assert_eq!(tabs.tabs.len(), 2);
        assert!(tabs.tabs[1].preview);

        tabs.open(Page(Subscriptions));
        assert_eq!(tabs.active(), &Page(Subscriptions));
        assert_eq!(
            tabs.history(),
            (
                &[Page(PortalPage::Home), Page(Resources), Page(Subscriptions)][..],
                2
            )
        );
        tabs.open(Page(Resources));
        assert_eq!(tabs.tabs.len(), 3);
        assert_eq!(tabs.active(), &Page(Resources));
        assert_eq!(tabs.close(0), Some(Page(Subscriptions)));
        assert_eq!(tabs.active(), &Page(Resources));
        assert_eq!(tabs.close(0), Some(Page(ResourceGroups)));
        assert_eq!(tabs.active(), &Page(Resources));
        assert_eq!(tabs.close(0), Some(Page(Resources)));
        assert_eq!(tabs.active(), &Page(PortalPage::Home));
    }

    #[test]
    fn home_can_close_and_is_recreated_when_the_last_tab_closes() {
        let mut tabs = PortalTabs::default();
        let home = PortalTabTarget::Page(PortalPage::Home);
        let logs = PortalTabTarget::Page(PortalPage::Logs);
        assert!(tabs.tabs[0].preview);

        tabs.open_kept(logs.clone());
        assert_eq!(tabs.close(0), Some(home.clone()));
        assert_eq!(tabs.active(), &logs);
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.close(0), Some(logs));
        assert!(tabs.tabs[0].preview);
        assert_eq!(tabs.history(), (std::slice::from_ref(&home), 0));
        assert_eq!(tabs.close(0), Some(home));
        assert!(!tabs.back());
    }

    #[test]
    fn new_tab_opens_home_with_its_own_history() {
        let mut tabs = PortalTabs::default();
        let home = PortalTabTarget::Page(PortalPage::Home);
        let resources = PortalTabTarget::Page(PortalPage::Resources);
        tabs.open(resources.clone());
        tabs.open_new_tab();
        tabs.open_new_tab();

        assert_eq!(tabs.tabs.len(), 3);
        assert!(tabs.tabs[2].preview);
        assert_eq!(tabs.history(), (std::slice::from_ref(&home), 0));
        tabs.select(0);
        assert_eq!(tabs.history(), (&[home, resources][..], 1));
    }

    #[test]
    fn history_replays_in_the_current_tab_and_new_navigation_drops_forward_visits() {
        use PortalPage::Logs;
        use PortalPage::ResourceGroups;
        use PortalPage::Resources;
        use PortalPage::StorageAccounts;
        use PortalTabTarget::Page;

        let mut tabs = PortalTabs::default();
        tabs.open_kept(Page(Resources));
        tabs.open(Page(StorageAccounts));
        tabs.open(Page(ResourceGroups));
        assert_eq!(
            tabs.history(),
            (
                &[Page(Resources), Page(StorageAccounts), Page(ResourceGroups)][..],
                2
            )
        );

        assert!(tabs.back());
        assert!(tabs.back());
        assert!(!tabs.back());
        assert_eq!(tabs.active, 2);
        assert_eq!(tabs.active(), &Page(Resources));
        assert!(tabs.go_to(2));
        assert_eq!(tabs.active(), &Page(ResourceGroups));
        assert!(tabs.back());
        assert!(tabs.forward());
        assert!(tabs.back());

        tabs.open(Page(Logs));
        assert_eq!(
            tabs.history(),
            (&[Page(Resources), Page(StorageAccounts), Page(Logs)][..], 2)
        );
        assert!(!tabs.forward());
        assert!(!tabs.go_to(4));
    }

    #[test]
    fn switching_scope_blades_replaces_only_the_displayed_visit() -> eyre::Result<()> {
        let scope = PortalResourceScope::ResourceGroup(AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?);
        let overview = scope.overview();
        let activity = PortalTabTarget::ScopeView {
            scope: scope.clone(),
            view: PortalScopeView::ActivityLog,
        };
        let access = PortalTabTarget::ScopeView {
            scope,
            view: PortalScopeView::AccessControl,
        };
        let inventory = PortalTabTarget::Page(PortalPage::ResourceGroups);
        let home = PortalTabTarget::Page(PortalPage::Home);
        let mut tabs = PortalTabs::default();
        tabs.open(inventory.clone());
        tabs.open(overview.clone());
        tabs.keep_open(0);

        tabs.replace_current(activity);
        tabs.replace_current(access.clone());
        assert!(tabs.back());
        assert_eq!(tabs.active(), &inventory);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &access);

        let next = PortalTabTarget::Page(PortalPage::Logs);
        tabs.navigate_current(next.clone());
        assert!(tabs.back());
        tabs.replace_current(overview.clone());
        assert_eq!(
            tabs.history(),
            (&[home, inventory, overview, next.clone()][..], 2)
        );
        assert_eq!(tabs.tabs.len(), 1);
        assert!(!tabs.tabs[0].preview);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &next);
        Ok(())
    }

    #[test]
    fn changing_accounts_clears_only_azure_detail_tabs() -> eyre::Result<()> {
        let mut tabs = PortalTabs::default();
        let group_id = cloud_terrastodon_azure::AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let storage_id = AzureStorageAccountId::try_new(group_id.clone(), "syntheticstore")?;
        let detail = PortalTabTarget::ResourceGroup(group_id);
        let output = PortalTabTarget::ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(storage_id.clone()),
            format: OutputFormat::Json,
        };
        let folder = PortalTabTarget::LocalPath("example".into());
        let accounts = PortalTabTarget::Page(PortalPage::Accounts);
        let tenants = PortalTabTarget::Page(PortalPage::AzureTenants);
        let inventory = PortalTabTarget::Page(PortalPage::ResourceGroups);
        let logs = PortalTabTarget::Page(PortalPage::Logs);
        tabs.open_kept(folder.clone());
        tabs.open_kept(inventory.clone());
        assert!(tabs.navigate_current(detail.clone()));
        assert!(tabs.navigate_current(logs.clone()));
        assert!(tabs.navigate_current(output.clone()));
        assert!(tabs.back());
        tabs.open_kept(accounts.clone());
        tabs.open_kept(tenants.clone());
        tabs.open_kept(detail);
        tabs.open_kept(PortalTabTarget::StorageAccount(storage_id));
        tabs.open_kept(output);
        tabs.clear_azure_details();
        assert_eq!(tabs.tabs.len(), 5);
        assert_eq!(tabs.active(), &PortalTabTarget::Page(PortalPage::Home));
        assert!(tabs.contains(&folder));
        assert!(tabs.contains(&accounts));
        assert!(tabs.contains(&tenants));
        tabs.open_kept(logs.clone());
        assert_eq!(tabs.history(), (&[inventory, logs][..], 1));
        assert!(!tabs.forward());
        Ok(())
    }

    #[test]
    fn changing_accounts_restores_home_if_every_tab_was_scoped() -> eyre::Result<()> {
        let id = cloud_terrastodon_azure::AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?;
        let mut tabs = PortalTabs::default();
        tabs.open(PortalTabTarget::ResourceGroup(id));
        tabs.clear_azure_details();
        assert!(tabs.tabs[0].preview);
        assert_eq!(tabs.active(), &PortalTabTarget::Page(PortalPage::Home));
        assert!(!tabs.back());
        Ok(())
    }

    #[test]
    fn storage_account_output_uses_preview_navigation() -> eyre::Result<()> {
        use PortalTabTarget::Page;
        use PortalTabTarget::ResourceOutput;
        use PortalTabTarget::StorageAccount;

        let id = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic-group",
            "/providers/Microsoft.Storage/storageAccounts/syntheticstore",
        )
        .parse::<AzureStorageAccountId>()?;
        let mut tabs = PortalTabs::default();
        let detail = StorageAccount(id.clone());
        let output = ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
            format: OutputFormat::Json,
        };

        tabs.open(Page(PortalPage::StorageAccounts));
        tabs.open(detail.clone());
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active(), &detail);
        assert!(tabs.tabs[0].preview);

        tabs.open(output.clone());
        assert_eq!(tabs.active(), &output);
        assert_eq!(tabs.tabs.len(), 1);
        assert!(tabs.tabs[0].preview);
        assert!(tabs.back());
        assert_eq!(tabs.active(), &detail);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &output);

        // The overview breadcrumb replaces the output preview in the same tab.
        tabs.open(detail.clone());
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active(), &detail);

        tabs.keep_open(0);
        tabs.open(output.clone());
        assert_eq!(tabs.tabs.len(), 2);
        assert!(tabs.tabs[1].preview);
        assert_eq!(tabs.history(), (&[detail.clone(), output.clone()][..], 1));
        tabs.open(detail.clone());
        assert_eq!(tabs.active, 0);
        tabs.open(output.clone());
        assert_eq!(tabs.active, 1);
        assert_eq!(tabs.tabs.len(), 2);

        let text_output = ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
            format: OutputFormat::Text,
        };
        assert!(tabs.navigate_current(text_output.clone()));
        assert!(!tabs.navigate_current(text_output.clone()));
        assert!(tabs.back());
        assert_eq!(tabs.active(), &output);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &text_output);
        assert!(tabs.back());

        assert_eq!(tabs.close(0), Some(detail));
        assert!(tabs.has_storage_account(&id));
        assert_eq!(tabs.close(0), Some(output));
        assert!(!tabs.has_storage_account(&id));
        Ok(())
    }
}
