use crate::app::MyApp;
use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::entra_group_section::EntraGroupSection;
use crate::entra_user_section::EntraUserSection;
use crate::entra_user_subject::EntraUserSubject;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_resource_scope::network_security_group_id;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_snapshot::load_network_security_group;
use crate::scope_activity::load_scope_activity;
use crate::scope_iam::load_scope_iam;
use crate::scope_tags_browser::load_scope_tags;
use crate::storage_account_browser::load_storage_account;
use crate::storage_explorer::load_storage_explorer;
use crate::storage_explorer::load_storage_explorer_container;
use crate::windows::dir_window::load_local_path;
use crate::windows::management_groups_window::load_management_groups;
use crate::windows::resources_window::load_resources;
use crate::workers::load_resource_groups::load_resource_groups;
use crate::workers::load_subscriptions::load_subscriptions_with_invalidation;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Ui;

impl MyApp {
    pub(crate) fn can_refresh_portal_view(&self, target: &PortalTabTarget) -> bool {
        self.can_refresh_portal_view_in(target, &self.azure)
    }

    pub(crate) fn can_refresh_portal_view_in(
        &self,
        target: &PortalTabTarget,
        content: &PortalAzureContent,
    ) -> bool {
        let has_azure_account = self
            .ready_azure_authentication_for(content.authentication)
            .is_some();
        match target {
            PortalTabTarget::Page(PortalPage::Entra) => false,
            PortalTabTarget::Page(PortalPage::EntraMyUser) => {
                has_azure_account
                    && content.authentication.is_some_and(|authentication| {
                        authentication.auth_source != AzureBearerAuthSource::WorkloadIdentity
                    })
                    && content.entra_directory.can_refresh(target)
            }
            PortalTabTarget::EntraGroupSection { group_id, section } => {
                has_azure_account
                    && match section {
                        EntraGroupSection::Overview | EntraGroupSection::Properties => content
                            .entra_directory
                            .can_refresh(&PortalTabTarget::EntraGroup(*group_id)),
                        EntraGroupSection::AzureDevOpsPermissions => !matches!(
                            content.entra_group_devops_permissions.get(group_id),
                            Some(Loadable::Loading { .. })
                        ),
                        _ => content.entra_group_data.can_refresh(*group_id, *section),
                    }
            }
            PortalTabTarget::EntraUserSection { user, section } => {
                has_azure_account
                    && (*user != EntraUserSubject::SignedIn
                        || content.authentication.is_some_and(|authentication| {
                            authentication.auth_source != AzureBearerAuthSource::WorkloadIdentity
                        }))
                    && match (section, user.user_id(&content.entra_directory)) {
                        (EntraUserSection::Overview, _) | (_, None) => {
                            content.entra_directory.can_refresh(&user.overview())
                        }
                        (EntraUserSection::GroupMemberships, Some(id)) => {
                            content.entra_user_groups.can_refresh_memberships(id)
                        }
                        (EntraUserSection::GroupOwnerships, Some(id)) => {
                            content.entra_user_groups.can_refresh_ownerships(id)
                        }
                        (EntraUserSection::ApplicationOwnership, Some(id)) => {
                            content.entra_user_applications.can_refresh_ownership(id)
                        }
                        (EntraUserSection::ApplicationAssignments, Some(id)) => {
                            content.entra_user_applications.can_refresh_assignments(id)
                        }
                        (EntraUserSection::AzureRoleAssignments, Some(id)) => {
                            content.entra_user_permissions.can_refresh_azure_roles(id)
                        }
                        (EntraUserSection::AzureDevOpsPermissions, Some(id)) => content
                            .entra_user_permissions
                            .can_refresh_devops_permissions(id),
                    }
            }
            PortalTabTarget::Page(PortalPage::Pim) => false,
            PortalTabTarget::Page(
                page @ (PortalPage::PimEntraRoles | PortalPage::PimAzureResources),
            ) => {
                has_azure_account
                    && content.authentication.is_some_and(|authentication| {
                        authentication.auth_source != AzureBearerAuthSource::WorkloadIdentity
                    })
                    && content.pim_roles.can_refresh(*page)
                    && !content
                        .pim_activation
                        .as_ref()
                        .is_some_and(|activation| activation.is_loading())
            }
            PortalTabTarget::Page(
                PortalPage::EntraUsers
                | PortalPage::EntraGroups
                | PortalPage::EntraAppRegistrations,
            )
            | PortalTabTarget::EntraUser(_)
            | PortalTabTarget::EntraGroup(_)
            | PortalTabTarget::EntraAppRegistration(_) => {
                has_azure_account && content.entra_directory.can_refresh(target)
            }
            PortalTabTarget::Page(
                PortalPage::Home
                | PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts
                | PortalPage::ContainerRegistries
                | PortalPage::KeyVaults
                | PortalPage::Tags,
            )
            | PortalTabTarget::ResourcesOfType(_) => {
                has_azure_account && !matches!(content.resources, Loadable::Loading { .. })
            }
            PortalTabTarget::Resource(scope) => {
                has_azure_account && can_refresh_resource(content, scope)
            }
            PortalTabTarget::Page(PortalPage::ResourceGroups) => {
                has_azure_account && !matches!(content.resource_groups, Loadable::Loading { .. })
            }
            PortalTabTarget::Page(PortalPage::ManagementGroups) => {
                has_azure_account && !matches!(content.management_groups, Loadable::Loading { .. })
            }
            PortalTabTarget::ManagementGroup(_) => {
                has_azure_account
                    && !matches!(content.management_groups, Loadable::Loading { .. })
                    && !matches!(content.subscriptions, Loadable::Loading { .. })
            }
            PortalTabTarget::ResourceGroup(_) => {
                has_azure_account
                    && !matches!(content.resource_groups, Loadable::Loading { .. })
                    && !matches!(content.resources, Loadable::Loading { .. })
            }
            PortalTabTarget::Subscription(_) => {
                has_azure_account
                    && !matches!(content.subscriptions, Loadable::Loading { .. })
                    && !matches!(content.resources, Loadable::Loading { .. })
            }
            PortalTabTarget::ScopeView { scope, view } => {
                has_azure_account
                    && match view {
                        PortalScopeView::ActivityLog => !content
                            .scope_activity
                            .get(scope)
                            .is_some_and(|activity| activity.is_loading()),
                        PortalScopeView::AccessControl => {
                            content.scope_iam.can_refresh(&scope.arm_scope())
                        }
                        PortalScopeView::Tags => content.scope_tags.can_refresh(scope),
                        PortalScopeView::StorageExplorer => matches!(
                            scope,
                            PortalResourceScope::StorageAccount(id) if content.storage_explorer.can_refresh(id)
                        ),
                        PortalScopeView::Networking => matches!(
                            scope,
                            PortalResourceScope::StorageAccount(id) if !content.storage_account_browser.entries.get(id)
                                .is_some_and(|entry| matches!(entry.response, Loadable::Loading { .. }))
                        ),
                    }
            }
            PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::Scope(scope),
                ..
            } => {
                has_azure_account
                    && match scope {
                        PortalResourceScope::ResourceGroup(_) => {
                            !matches!(content.resource_groups, Loadable::Loading { .. })
                        }
                        PortalResourceScope::Subscription(_) => {
                            !matches!(content.subscriptions, Loadable::Loading { .. })
                        }
                        PortalResourceScope::ManagementGroup(_) => {
                            !matches!(content.management_groups, Loadable::Loading { .. })
                        }
                        PortalResourceScope::Resource(scope) => {
                            can_refresh_resource(content, scope)
                        }
                        PortalResourceScope::StorageAccount(id) => !content
                            .storage_account_browser
                            .entries
                            .get(id)
                            .is_some_and(|entry| {
                                matches!(entry.response, Loadable::Loading { .. })
                            }),
                    }
            }
            PortalTabTarget::StorageAccount(id)
            | PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::StorageAccount(id),
                ..
            } => {
                has_azure_account
                    && !content
                        .storage_account_browser
                        .entries
                        .get(id)
                        .is_some_and(|entry| matches!(entry.response, Loadable::Loading { .. }))
            }
            PortalTabTarget::StorageContainer(id) => {
                has_azure_account && content.storage_explorer.can_refresh_container(id)
            }
            PortalTabTarget::AzureDevOpsOrganization {
                organization,
                view:
                    PortalAzureDevOpsOrganizationView::Projects
                    | PortalAzureDevOpsOrganizationView::Project(_),
            } => self.can_refresh_azure_devops_projects_in(content, organization),
            PortalTabTarget::AzureDevOpsOrganization {
                organization,
                view: PortalAzureDevOpsOrganizationView::Users,
            } => self.can_refresh_azure_devops_users_in(content, organization),
            PortalTabTarget::AzureDevOpsProject {
                organization,
                project,
                view,
            } => self.can_refresh_azure_devops_project_in(content, organization, *project, *view),
            PortalTabTarget::Page(PortalPage::Accounts) => self.account_operation.is_none(),
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations)
            | PortalTabTarget::AzureDevOpsOrganization {
                view: PortalAzureDevOpsOrganizationView::Overview,
                ..
            } => {
                self.account_operation.is_none()
                    && !self
                        .azure_devops_organization_picker
                        .is_loading_tracked_organizations()
            }
            PortalTabTarget::Page(PortalPage::AzureTenants) | PortalTabTarget::AzureTenant(_) => {
                self.account_operation.is_none() && self.tenant_manager.can_reload_catalogue()
            }
            PortalTabTarget::LocalPath(path) => !matches!(
                self.local_path_previews.get(path),
                Some(Loadable::Loading { .. })
            ),
            PortalTabTarget::Page(
                PortalPage::StartingPoints
                | PortalPage::SelectedItems
                | PortalPage::WorkDirs
                | PortalPage::FileDrop
                | PortalPage::Logs
                | PortalPage::BackgroundWork
                | PortalPage::About
                | PortalPage::Auditing
                | PortalPage::AzureDevOpsAudit
                | PortalPage::StorageAccountAudit
                | PortalPage::History,
            ) => false,
        }
    }

    pub(crate) fn refresh_portal_view(&mut self, ui: &mut Ui, target: &PortalTabTarget) {
        if !self.can_refresh_portal_view(target) {
            return;
        }
        match target {
            PortalTabTarget::Page(PortalPage::Entra) => {}
            PortalTabTarget::Page(PortalPage::EntraMyUser) => {
                self.load_entra_directory(target, true)
            }
            PortalTabTarget::EntraGroupSection { group_id, section } => match section {
                EntraGroupSection::Overview | EntraGroupSection::Properties => {
                    self.load_entra_directory(&PortalTabTarget::EntraGroup(*group_id), true);
                }
                EntraGroupSection::AzureDevOpsPermissions => {
                    self.load_entra_group_devops_permissions(*group_id, true);
                }
                _ => self.load_entra_group_section(*group_id, *section, true),
            },
            PortalTabTarget::EntraUserSection { user, section } => {
                match (section, user.user_id(&self.azure.entra_directory)) {
                    (EntraUserSection::Overview, _) | (_, None) => {
                        self.load_entra_directory(&user.overview(), true);
                    }
                    (EntraUserSection::GroupMemberships, Some(id)) => {
                        self.load_entra_user_group_memberships(id, true);
                    }
                    (EntraUserSection::GroupOwnerships, Some(id)) => {
                        self.load_entra_user_group_ownerships(id, true);
                    }
                    (EntraUserSection::ApplicationOwnership, Some(id)) => {
                        self.load_entra_user_application_ownership(id, true);
                    }
                    (EntraUserSection::ApplicationAssignments, Some(id)) => {
                        self.load_entra_user_application_assignments(id, true);
                    }
                    (EntraUserSection::AzureRoleAssignments, Some(id)) => {
                        self.load_entra_user_azure_roles(id, true);
                    }
                    (EntraUserSection::AzureDevOpsPermissions, Some(id)) => {
                        self.load_entra_user_devops_permissions(id, true);
                    }
                }
            }
            PortalTabTarget::Page(PortalPage::Pim) => {}
            PortalTabTarget::Page(
                page @ (PortalPage::PimEntraRoles | PortalPage::PimAzureResources),
            ) => {
                self.load_pim_roles(*page, true);
            }
            PortalTabTarget::Page(
                PortalPage::EntraUsers
                | PortalPage::EntraGroups
                | PortalPage::EntraAppRegistrations,
            )
            | PortalTabTarget::EntraUser(_)
            | PortalTabTarget::EntraGroup(_)
            | PortalTabTarget::EntraAppRegistration(_) => self.load_entra_directory(target, true),
            PortalTabTarget::Page(
                PortalPage::Home
                | PortalPage::Resources
                | PortalPage::Subscriptions
                | PortalPage::StorageAccounts
                | PortalPage::ContainerRegistries
                | PortalPage::KeyVaults
                | PortalPage::Tags,
            )
            | PortalTabTarget::ResourcesOfType(_) => load_resources(self, true),
            PortalTabTarget::Resource(scope) => refresh_resource(self, scope),
            PortalTabTarget::Page(PortalPage::ResourceGroups) => {
                load_resource_groups(self, true);
            }
            PortalTabTarget::Page(PortalPage::ManagementGroups) => {
                load_management_groups(self, true)
            }
            PortalTabTarget::ManagementGroup(_) => {
                load_management_groups(self, true);
                load_subscriptions_with_invalidation(self, true);
            }
            PortalTabTarget::ResourceGroup(_) => {
                load_resource_groups(self, true);
                load_resources(self, true);
            }
            PortalTabTarget::Subscription(_) => {
                load_subscriptions_with_invalidation(self, true);
                load_resources(self, true);
            }
            PortalTabTarget::ScopeView { scope, view } => match view {
                PortalScopeView::ActivityLog => load_scope_activity(self, scope, true),
                PortalScopeView::AccessControl => load_scope_iam(self, &scope.arm_scope(), true),
                PortalScopeView::Tags => load_scope_tags(self, scope, true),
                PortalScopeView::StorageExplorer => {
                    if let PortalResourceScope::StorageAccount(id) = scope {
                        load_storage_explorer(self, id, true);
                    }
                }
                PortalScopeView::Networking => {
                    if let PortalResourceScope::StorageAccount(id) = scope {
                        load_storage_account(self, id, true);
                    }
                }
            },
            PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::Scope(scope),
                ..
            } => match scope {
                PortalResourceScope::ResourceGroup(_) => load_resource_groups(self, true),
                PortalResourceScope::Subscription(_) => {
                    load_subscriptions_with_invalidation(self, true)
                }
                PortalResourceScope::ManagementGroup(_) => load_management_groups(self, true),
                PortalResourceScope::Resource(scope) => refresh_resource(self, scope),
                PortalResourceScope::StorageAccount(id) => load_storage_account(self, id, true),
            },
            PortalTabTarget::StorageAccount(id)
            | PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::StorageAccount(id),
                ..
            } => {
                load_storage_account(self, id, true);
            }
            PortalTabTarget::StorageContainer(id) => {
                load_storage_explorer_container(self, id, true);
            }
            PortalTabTarget::AzureDevOpsOrganization {
                organization,
                view:
                    PortalAzureDevOpsOrganizationView::Projects
                    | PortalAzureDevOpsOrganizationView::Project(_),
            } => {
                self.refresh_azure_devops_projects_for(organization);
            }
            PortalTabTarget::AzureDevOpsOrganization {
                organization,
                view: PortalAzureDevOpsOrganizationView::Users,
            } => self.load_azure_devops_users(organization, true),
            PortalTabTarget::AzureDevOpsProject {
                organization,
                project,
                view,
            } => {
                self.load_azure_devops_project_section(organization, *project, *view, true);
            }
            PortalTabTarget::Page(PortalPage::Accounts) => {
                self.handle_account_picker_action(AzureAccountPickerAction::Reload, ui.ctx());
            }
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations)
            | PortalTabTarget::AzureDevOpsOrganization {
                view: PortalAzureDevOpsOrganizationView::Overview,
                ..
            } => {
                self.reload_tracked_azure_devops_organizations();
                self.handle_account_picker_action(AzureAccountPickerAction::Reload, ui.ctx());
            }
            PortalTabTarget::Page(PortalPage::AzureTenants) | PortalTabTarget::AzureTenant(_) => {
                self.tenant_manager.reload_catalogue(
                    &self.work_tracker,
                    &self.executor,
                    &self.work_cache,
                    ui.ctx(),
                )
            }
            PortalTabTarget::LocalPath(path) => load_local_path(self, path),
            PortalTabTarget::Page(
                PortalPage::StartingPoints
                | PortalPage::SelectedItems
                | PortalPage::WorkDirs
                | PortalPage::FileDrop
                | PortalPage::Logs
                | PortalPage::BackgroundWork
                | PortalPage::About
                | PortalPage::Auditing
                | PortalPage::AzureDevOpsAudit
                | PortalPage::StorageAccountAudit
                | PortalPage::History,
            ) => {}
        }
    }
}

fn can_refresh_resource(content: &PortalAzureContent, scope: &ScopeImpl) -> bool {
    if let Some(id) = network_security_group_id(scope) {
        !matches!(
            content.network_security_groups.get(id),
            Some(Loadable::Loading { .. })
        )
    } else {
        !matches!(content.resources, Loadable::Loading { .. })
    }
}

fn refresh_resource(app: &mut MyApp, scope: &ScopeImpl) {
    if let Some(id) = network_security_group_id(scope) {
        load_network_security_group(app, id, true);
    } else {
        load_resources(app, true);
    }
}
