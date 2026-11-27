use crate::entra_group_section::EntraGroupSection;
use crate::entra_user_section::EntraUserSection;
use crate::entra_user_subject::EntraUserSubject;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::resource_output::format_label;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::AzureStorageAccountBlobContainerId;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraApplicationObjectId;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;

/// The identity of a view that can be opened in the portal.
#[derive(Clone, Debug, Eq, Facet, PartialEq)]
#[repr(C)]
pub enum PortalTabTarget {
    Page(PortalPage),
    ResourceGroup(AzureResourceGroupId),
    Subscription(SubscriptionId),
    ScopeView {
        scope: PortalResourceScope,
        view: PortalScopeView,
    },
    ResourcesOfType(ResourceType),
    ManagementGroup(ManagementGroupId),
    StorageAccount(AzureStorageAccountId),
    Resource(Box<ScopeImpl>),
    EntraUser(EntraUserId),
    EntraGroup(EntraGroupId),
    ResourceOutput {
        resource: PortalResourceOutputTarget,
        format: OutputFormat,
    },
    LocalPath(PathBuf),
    EntraAppRegistration(EntraApplicationObjectId),
    StorageContainer(AzureStorageAccountBlobContainerId),
    AzureDevOpsOrganization {
        organization: Box<AzureDevOpsOrganizationUrl>,
        view: PortalAzureDevOpsOrganizationView,
    },
    AzureTenant(AzureTenantId),
    AzureDevOpsProject {
        organization: Box<AzureDevOpsOrganizationUrl>,
        project: AzureDevOpsProjectId,
        view: PortalAzureDevOpsProjectView,
    },
    HelixServer {
        server: BmcHelixServerName,
        view: PortalHelixServerView,
    },
    EntraGroupSection {
        group_id: EntraGroupId,
        section: EntraGroupSection,
    },
    EntraUserSection {
        user: EntraUserSubject,
        section: EntraUserSection,
    },
}

impl PortalTabTarget {
    /// Lift a project visit recorded before project sections had their own destination.
    pub(crate) fn into_current(self) -> Self {
        match self {
            Self::AzureDevOpsOrganization {
                organization,
                view: PortalAzureDevOpsOrganizationView::Project(project),
            } => Self::AzureDevOpsProject {
                organization,
                project,
                view: PortalAzureDevOpsProjectView::Overview,
            },
            target => target,
        }
    }

    /// Open the known overview for a typed ARM scope or the general resource view.
    pub(crate) fn from_arm_scope(scope: &ScopeImpl) -> Self {
        match scope {
            ScopeImpl::ResourceGroup(id) => Self::ResourceGroup(id.clone()),
            ScopeImpl::Subscription(id) => Self::Subscription(*id),
            ScopeImpl::ManagementGroup(id) => Self::ManagementGroup(id.clone()),
            ScopeImpl::StorageAccount(id) => Self::StorageAccount(id.clone()),
            id => Self::Resource(Box::new(id.clone())),
        }
    }

    pub fn title(&self) -> Cow<'_, str> {
        match self {
            Self::Page(page) => Cow::Borrowed(page.title()),
            Self::ResourceGroup(id) => Cow::Owned(id.resource_group_name.to_string()),
            Self::Subscription(id) => Cow::Owned(id.to_string()),
            Self::ScopeView { scope, view } => {
                Cow::Owned(format!("{} · {}", scope.overview().title(), view.label()))
            }
            Self::ResourcesOfType(kind) => Cow::Borrowed(kind.as_ref()),
            Self::ManagementGroup(id) => Cow::Borrowed(id.name()),
            Self::StorageAccount(id) => Cow::Borrowed(id.storage_account_name.as_str()),
            Self::StorageContainer(id) => Cow::Borrowed(id.container_name.as_str()),
            Self::AzureDevOpsOrganization { organization, view } => match view {
                PortalAzureDevOpsOrganizationView::Overview => Cow::Borrowed(organization.name()),
                PortalAzureDevOpsOrganizationView::Projects => {
                    Cow::Owned(format!("{} · Projects", organization.name()))
                }
                PortalAzureDevOpsOrganizationView::Users => {
                    Cow::Owned(format!("{} · Users", organization.name()))
                }
                PortalAzureDevOpsOrganizationView::Project(id) => {
                    Cow::Owned(format!("Project {id}"))
                }
            },
            Self::AzureDevOpsProject { project, view, .. } => {
                if *view == PortalAzureDevOpsProjectView::Overview {
                    Cow::Owned(format!("Project {project}"))
                } else {
                    Cow::Owned(format!("Project {project} · {}", view.label()))
                }
            }
            Self::HelixServer { server, view } => match view {
                PortalHelixServerView::Overview => Cow::Owned(server.to_string()),
                PortalHelixServerView::WorkOrders => Cow::Owned(format!("{server} · Work orders")),
                PortalHelixServerView::Tickets => Cow::Owned(format!("{server} · Tickets")),
                PortalHelixServerView::WorkOrder(id) => Cow::Owned(id.to_string()),
                PortalHelixServerView::Ticket(id) => Cow::Borrowed(id.as_str()),
            },
            Self::EntraUser(id) => Cow::Owned(format!("User {id}")),
            Self::EntraUserSection { user, section } => {
                Cow::Owned(format!("{} · {}", user.overview().title(), section.title()))
            }
            Self::AzureTenant(id) => Cow::Owned(id.to_string()),
            Self::EntraGroup(id) => Cow::Owned(format!("Group {id}")),
            Self::EntraGroupSection { group_id, section } => {
                Cow::Owned(format!("Group {group_id} · {}", section.title()))
            }
            Self::EntraAppRegistration(id) => Cow::Owned(format!("App registration {id}")),
            Self::Resource(id) => {
                let full = id.expanded_form();
                Cow::Owned(
                    full.rsplit('/')
                        .find(|part| !part.is_empty())
                        .unwrap_or(&full)
                        .to_owned(),
                )
            }
            Self::ResourceOutput { resource, format } => Cow::Owned(format!(
                "{} · {}",
                resource.overview().title(),
                format_label(*format)
            )),
            Self::LocalPath(path) => path
                .file_name()
                .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy()),
        }
    }

    pub fn page(&self) -> PortalPage {
        match self {
            Self::Page(page) => *page,
            Self::ResourceGroup(_) => PortalPage::ResourceGroups,
            Self::Subscription(_) => PortalPage::Subscriptions,
            Self::ScopeView { scope, .. } => scope.inventory(),
            Self::ResourcesOfType(_) => PortalPage::Resources,
            Self::ManagementGroup(_) => PortalPage::ManagementGroups,
            Self::StorageAccount(_) | Self::StorageContainer(_) => PortalPage::StorageAccounts,
            Self::Resource(_) => PortalPage::Resources,
            Self::EntraUser(_) => PortalPage::EntraUsers,
            Self::EntraUserSection { user, .. } => user.page(),
            Self::AzureTenant(_) => PortalPage::AzureTenants,
            Self::EntraGroup(_) | Self::EntraGroupSection { .. } => PortalPage::EntraGroups,
            Self::EntraAppRegistration(_) => PortalPage::EntraAppRegistrations,
            Self::AzureDevOpsOrganization { .. } | Self::AzureDevOpsProject { .. } => {
                PortalPage::AzureDevOpsOrganizations
            }
            Self::HelixServer { .. } => PortalPage::HelixServers,
            Self::ResourceOutput { resource, .. } => resource.inventory(),
            Self::LocalPath(_) => PortalPage::WorkDirs,
        }
    }
}
