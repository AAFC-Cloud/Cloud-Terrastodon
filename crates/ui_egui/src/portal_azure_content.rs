use crate::azure_devops_project_inventory::AzureDevOpsProjectInventory;
use crate::azure_devops_user_inventory::AzureDevOpsUserInventory;
use crate::entra_devops_permissions::EntraDevOpsPermissionOrganization;
use crate::entra_directory::EntraDirectory;
use crate::entra_group_data::EntraGroupData;
use crate::entra_user_application_data::EntraUserApplicationData;
use crate::entra_user_groups::EntraUserGroups;
use crate::entra_user_permissions::EntraUserPermissions;
use crate::pim_activation::PimActivation;
use crate::pim_role_inventory::PimRoleInventory;
use crate::portal_pane_id::PortalPaneId;
use crate::portal_resource_scope::PortalResourceScope;
use crate::resource_group_browser::ResourceGroupBrowser;
use crate::resource_inventory::ResourceInventory;
use crate::resource_output::ResourceOutput;
use crate::scope_activity_log::ScopeActivityLog;
use crate::scope_iam_browser::ScopeIamBrowser;
use crate::scope_tags_browser::ScopeTagsBrowser;
use crate::storage_account_browser::StorageAccountBrowser;
use crate::storage_explorer::StorageExplorer;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_azure::AzureNetworkSecurityGroup;
use cloud_terrastodon_azure::AzureResourceGroupMap;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::ManagementGroup;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceId;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Id;
use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

/// Account-bound data for Azure and Azure DevOps browser content. The tab layout
/// is independent of this state and can host pages with other credential choices.
#[derive(Default)]
pub(crate) struct PortalAzureContent {
    pub authentication: Option<AzureBearerAuthenticationSelection>,
    /// Unique within the window so late replies can find their issuing view.
    pub account_generation: u64,
    pub resource_search: String,
    pub checkboxes: HashMap<Id, bool>,
    pub toggle_intents: HashSet<Id>,
    pub subscriptions: Loadable<Rc<Vec<Arc<Subscription>>>>,
    pub azure_devops_projects: AzureDevOpsProjectInventory,
    pub azure_devops_users: AzureDevOpsUserInventory,
    pub azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    pub devops_pat_accounts: HashMap<AzureDevOpsOrganizationUrl, CloudTerrastodonAccountId>,
    pub devops_organization_generation: u64,
    pub resource_groups: Loadable<Rc<AzureResourceGroupMap>>,
    pub resource_group_browser: ResourceGroupBrowser,
    pub storage_account_browser: StorageAccountBrowser,
    pub storage_explorer: StorageExplorer,
    pub resource_output: ResourceOutput,
    pub management_groups: Loadable<Vec<Arc<ManagementGroup>>>,
    pub resources: Loadable<Arc<Vec<Arc<Resource>>>>,
    pub network_security_groups: HashMap<ResourceId, Loadable<Arc<AzureNetworkSecurityGroup>>>,
    pub resource_inventories: HashMap<PortalPaneId, Option<ResourceInventory>>,
    pub scope_activity: HashMap<PortalResourceScope, ScopeActivityLog>,
    pub scope_iam: ScopeIamBrowser,
    pub scope_tags: ScopeTagsBrowser,
    pub entra_directory: EntraDirectory,
    pub entra_user_groups: EntraUserGroups,
    pub entra_user_applications: EntraUserApplicationData,
    pub entra_user_permissions: EntraUserPermissions,
    pub entra_group_data: EntraGroupData,
    pub entra_group_devops_permissions:
        HashMap<EntraGroupId, Loadable<Vec<EntraDevOpsPermissionOrganization>>>,
    pub pim_roles: PimRoleInventory,
    pub pim_activation: Option<PimActivation>,
    pub audit_results: HashMap<Id, Arc<dyn Any + Send + Sync>>,
}

impl PortalAzureContent {
    pub fn new(
        authentication: Option<AzureBearerAuthenticationSelection>,
        account_generation: u64,
    ) -> Self {
        Self {
            authentication,
            account_generation,
            ..Self::default()
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.resources, Loadable::Loading { .. })
            || self
                .network_security_groups
                .values()
                .any(|group| matches!(group, Loadable::Loading { .. }))
            || self
                .scope_activity
                .values()
                .any(ScopeActivityLog::is_loading)
            || self.scope_iam.is_loading()
            || self.scope_tags.is_loading()
            || matches!(self.subscriptions, Loadable::Loading { .. })
            || matches!(self.resource_groups, Loadable::Loading { .. })
            || matches!(self.management_groups, Loadable::Loading { .. })
            || self.azure_devops_projects.is_loading()
            || self.azure_devops_users.is_loading()
            || self.storage_account_browser.is_loading()
            || self.storage_explorer.is_loading()
            || self.entra_directory.is_loading()
            || self.entra_user_groups.is_loading()
            || self.entra_user_applications.is_loading()
            || self.entra_user_permissions.is_loading()
            || self.entra_group_data.is_loading()
            || self
                .entra_group_devops_permissions
                .values()
                .any(|result| matches!(result, Loadable::Loading { .. }))
            || self.pim_roles.is_loading()
            || self
                .pim_activation
                .as_ref()
                .is_some_and(PimActivation::is_loading)
    }

    pub fn reset_inventory(&mut self, generation: u64) {
        *self = Self {
            azure_devops_organization: self.azure_devops_organization.take(),
            devops_pat_accounts: std::mem::take(&mut self.devops_pat_accounts),
            devops_organization_generation: self.devops_organization_generation,
            ..Self::new(self.authentication, generation)
        };
    }

    /// Share initial choices with a newly opened page while giving it independent data.
    pub fn fork(&self, generation: u64) -> Self {
        Self {
            azure_devops_organization: self.azure_devops_organization.clone(),
            devops_pat_accounts: self.devops_pat_accounts.clone(),
            ..Self::new(self.authentication, generation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_page::PortalPage;
    use crate::portal_tab_target::PortalTabTarget;
    use crate::portal_tabs::PortalTabs;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use std::time::Instant;

    #[test]
    fn duplicate_resource_group_views_can_change_accounts_independently() -> eyre::Result<()> {
        let original_selection = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000001".parse()?,
            tenant_id: "00000000-0000-0000-0000-000000000002".parse()?,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
        let other_selection = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000003".parse()?,
            ..original_selection
        };
        let target = PortalTabTarget::Page(PortalPage::ResourceGroups);
        let mut tabs = PortalTabs::default();
        tabs.navigate_current(target.clone());
        tabs.duplicate(tabs.active_id());
        let mut original = PortalAzureContent::new(Some(original_selection), 0);
        let started_at = Instant::now();
        original.resource_groups = Loadable::Loading { started_at };
        let mut duplicate = original.fork(1);
        assert_eq!(duplicate.authentication, Some(original_selection));

        duplicate.authentication = Some(other_selection);
        duplicate.reset_inventory(2);
        assert_eq!(original.authentication, Some(original_selection));
        assert_eq!(duplicate.authentication, Some(other_selection));
        assert!(matches!(original.resource_groups, Loadable::Loading { .. }));
        assert!(matches!(duplicate.resource_groups, Loadable::NotLoaded));
        assert_eq!(
            tabs.iter()
                .filter(|(_, tab, _)| tab.target() == &target)
                .count(),
            2
        );
        Ok(())
    }
}
