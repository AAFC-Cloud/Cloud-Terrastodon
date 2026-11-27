use crate::app::MyApp;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_page::PortalPage;
use crate::portal_resource_scope::network_security_group_id;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_iconography::SvgIcon;
use cloud_terrastodon_iconography_azure::AzureAllResourcesIcon;
use cloud_terrastodon_iconography_azure::AzureContainerRegistryIcon;
use cloud_terrastodon_iconography_azure::AzureKeyVaultIcon;
use cloud_terrastodon_iconography_azure::AzureManagementGroupIcon;
use cloud_terrastodon_iconography_azure::AzureResourceGroupIcon;
use cloud_terrastodon_iconography_azure::AzureStorageAccountIcon;
use cloud_terrastodon_iconography_azure::AzureSubscriptionIcon;
use cloud_terrastodon_iconography_azure::AzureVirtualNetworkIcon;
use cloud_terrastodon_iconography_azure::icon_for_resource_type;
use eframe::egui::ImageSource;
use eframe::egui::include_image as i;
use eframe::egui::load::Bytes;
use std::borrow::Cow;

pub const DEVOPS_ICON: ImageSource = i!("../assets/10261-icon-service-Azure-DevOps-4x.png");
pub const DEVOPS_PROJECT_ICON: ImageSource = i!("../assets/devops-projects.svg");
pub const SUBSCRIPTION_ICON: ImageSource = i!("../assets/10002-icon-service-Subscriptions-4x.png");
pub const RESOURCE_GROUP_ICON: ImageSource =
    i!("../assets/10007-icon-service-Resource-Groups-4x.png");
pub const ABOUT_ICON: ImageSource = i!("../assets/about.svg");
pub const TAG_ICON: ImageSource = i!("../assets/tag.svg");
pub const TAGS_ICON: ImageSource = i!("../assets/tags.svg");
pub(crate) const ACCESS_CONTROL_ICON: ImageSource = i!("../assets/access-control.svg");
pub(crate) const ACTIVITY_LOG_ICON: ImageSource = i!("../assets/activity-log.svg");
pub(crate) const STORAGE_EXPLORER_ICON: ImageSource = i!("../assets/work-directories.svg");

/// Adapt the shared embedded SVG catalogue to egui's image cache. Packaged SVG
/// bytes stay borrowed; egui rasterizes each URI at the requested display size.
fn svg_source(icon: &dyn SvgIcon) -> ImageSource<'static> {
    ImageSource::Bytes {
        uri: format!("bytes://ct-icons/{}/{}.svg", icon.category(), icon.name()).into(),
        bytes: match icon.svg() {
            Cow::Borrowed(svg) => Bytes::Static(svg.as_bytes()),
            Cow::Owned(svg) => Bytes::from(svg.into_bytes()),
        },
    }
}

pub fn resource_icon(kind: &ResourceType) -> ImageSource<'static> {
    svg_source(icon_for_resource_type(kind).unwrap_or(&AzureAllResourcesIcon))
}

pub(crate) fn portal_target_icon(app: &MyApp, target: &PortalTabTarget) -> ImageSource<'static> {
    portal_target_icon_in(Some(&app.azure), target)
}

pub(crate) fn portal_target_icon_in(
    azure: Option<&PortalAzureContent>,
    target: &PortalTabTarget,
) -> ImageSource<'static> {
    match target {
        PortalTabTarget::Resource(id) => {
            if let Some(group) = network_security_group_id(id) {
                return resource_icon(&group.resource_type);
            }
            azure
                .and_then(|azure| azure.inventory_resource(id))
                .map_or_else(
                    || page_icon(PortalPage::Resources),
                    |resource| resource_icon(&resource.kind),
                )
        }
        PortalTabTarget::ResourcesOfType(kind) => resource_icon(kind),
        PortalTabTarget::ScopeView {
            view: PortalScopeView::ActivityLog,
            ..
        } => ACTIVITY_LOG_ICON,
        PortalTabTarget::ScopeView {
            view: PortalScopeView::Tags,
            ..
        } => TAG_ICON,
        PortalTabTarget::ScopeView {
            view: PortalScopeView::StorageExplorer,
            ..
        }
        | PortalTabTarget::StorageContainer(_) => STORAGE_EXPLORER_ICON,
        PortalTabTarget::ScopeView {
            view: PortalScopeView::Networking,
            ..
        } => networking_icon(),
        PortalTabTarget::ScopeView { scope, .. } => portal_target_icon_in(azure, &scope.overview()),
        PortalTabTarget::ResourceOutput { resource, .. } => {
            portal_target_icon_in(azure, &resource.overview())
        }
        PortalTabTarget::AzureDevOpsProject { view, .. } => view.icon(),
        PortalTabTarget::AzureDevOpsOrganization {
            view:
                PortalAzureDevOpsOrganizationView::Projects
                | PortalAzureDevOpsOrganizationView::Project(_),
            ..
        } => DEVOPS_PROJECT_ICON,
        PortalTabTarget::EntraGroupSection { section, .. } => section.icon(),
        PortalTabTarget::EntraUserSection { section, .. } => section.icon(),
        target => page_icon(target.page()),
    }
}

pub(crate) fn networking_icon() -> ImageSource<'static> {
    svg_source(&AzureVirtualNetworkIcon)
}

pub fn page_icon(page: PortalPage) -> ImageSource<'static> {
    match page {
        PortalPage::Home => i!("../assets/home.svg"),
        PortalPage::Resources | PortalPage::PimAzureResources => svg_source(&AzureAllResourcesIcon),
        PortalPage::Subscriptions => svg_source(&AzureSubscriptionIcon),
        PortalPage::ResourceGroups => svg_source(&AzureResourceGroupIcon),
        PortalPage::ManagementGroups => svg_source(&AzureManagementGroupIcon),
        PortalPage::StorageAccounts => svg_source(&AzureStorageAccountIcon),
        PortalPage::ContainerRegistries => svg_source(&AzureContainerRegistryIcon),
        PortalPage::KeyVaults => svg_source(&AzureKeyVaultIcon),
        PortalPage::Entra | PortalPage::PimEntraRoles => i!("../assets/entra-directory.svg"),
        PortalPage::EntraUsers | PortalPage::EntraMyUser => i!("../assets/entra-user.svg"),
        PortalPage::EntraGroups => i!("../assets/entra-group.svg"),
        PortalPage::EntraAppRegistrations => i!("../assets/entra-app-registrations.svg"),
        PortalPage::AzureDevOpsOrganizations => DEVOPS_ICON,
        PortalPage::Accounts => i!("../assets/starting-points.svg"),
        PortalPage::AzureTenants => svg_source(&AzureManagementGroupIcon),
        PortalPage::StartingPoints => i!("../assets/starting-points.svg"),
        PortalPage::SelectedItems => i!("../assets/selected-items.svg"),
        PortalPage::WorkDirs => i!("../assets/work-directories.svg"),
        PortalPage::FileDrop => i!("../assets/import-files.svg"),
        PortalPage::Logs => i!("../assets/logs.svg"),
        PortalPage::BackgroundWork => i!("../assets/selected-items.svg"),
        PortalPage::About => ABOUT_ICON,
        PortalPage::History => i!("../assets/history.svg"),
        PortalPage::Tags => TAGS_ICON,
        PortalPage::Auditing => i!("../assets/auditing.svg"),
        PortalPage::AzureDevOpsAudit => DEVOPS_ICON,
        PortalPage::StorageAccountAudit => svg_source(&AzureStorageAccountIcon),
        PortalPage::Pim => i!("../assets/pim.svg"),
    }
}
