use crate::portal_page::PortalPage;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::storage_account_output::format_label;
use cloud_terrastodon_app::OutputFormat;
use cloud_terrastodon_azure::AzureResourceGroupId;
use cloud_terrastodon_azure::AzureStorageAccountId;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::SubscriptionId;
use std::borrow::Cow;
use std::path::PathBuf;

/// The identity of a view that can be opened in the portal.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    StorageAccountOutput {
        id: AzureStorageAccountId,
        format: OutputFormat,
    },
    LocalPath(PathBuf),
}

impl PortalTabTarget {
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
            Self::StorageAccountOutput { id, format } => Cow::Owned(format!(
                "{} · {}",
                id.storage_account_name,
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
            Self::StorageAccount(_) | Self::StorageAccountOutput { .. } => {
                PortalPage::StorageAccounts
            }
            Self::LocalPath(_) => PortalPage::WorkDirs,
        }
    }
}
