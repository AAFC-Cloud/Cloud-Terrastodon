use crate::portal_page::PortalPage;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_iconography::SvgIcon;
use cloud_terrastodon_iconography_azure::AzureAllResourcesIcon;
use cloud_terrastodon_iconography_azure::AzureContainerRegistryIcon;
use cloud_terrastodon_iconography_azure::AzureKeyVaultIcon;
use cloud_terrastodon_iconography_azure::AzureResourceGroupIcon;
use cloud_terrastodon_iconography_azure::AzureStorageAccountIcon;
use cloud_terrastodon_iconography_azure::AzureSubscriptionIcon;
use cloud_terrastodon_iconography_azure::icon_for_resource_type;
use eframe::egui::ImageSource;
use eframe::egui::include_image as i;
use eframe::egui::load::Bytes;
use std::borrow::Cow;

pub const DEVOPS_ICON: ImageSource = i!("../assets/10261-icon-service-Azure-DevOps-4x.png");
pub const SUBSCRIPTION_ICON: ImageSource = i!("../assets/10002-icon-service-Subscriptions-4x.png");
pub const RESOURCE_GROUP_ICON: ImageSource =
    i!("../assets/10007-icon-service-Resource-Groups-4x.png");
pub const ABOUT_ICON: ImageSource = i!("../assets/about.svg");

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

pub fn page_icon(page: PortalPage) -> ImageSource<'static> {
    match page {
        PortalPage::Home => i!("../assets/home.svg"),
        PortalPage::Resources => svg_source(&AzureAllResourcesIcon),
        PortalPage::Subscriptions => svg_source(&AzureSubscriptionIcon),
        PortalPage::ResourceGroups => svg_source(&AzureResourceGroupIcon),
        PortalPage::StorageAccounts => svg_source(&AzureStorageAccountIcon),
        PortalPage::ContainerRegistries => svg_source(&AzureContainerRegistryIcon),
        PortalPage::KeyVaults => svg_source(&AzureKeyVaultIcon),
        PortalPage::StartingPoints => i!("../assets/starting-points.svg"),
        PortalPage::SelectedItems => i!("../assets/selected-items.svg"),
        PortalPage::WorkDirs => i!("../assets/work-directories.svg"),
        PortalPage::FileDrop => i!("../assets/import-files.svg"),
        PortalPage::Logs => i!("../assets/logs.svg"),
    }
}
