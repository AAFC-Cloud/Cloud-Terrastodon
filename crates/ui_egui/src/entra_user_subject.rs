use crate::entra_directory::EntraDirectory;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::EntraUserId;
use facet::Facet;

/// The selected account's signed-in user or an explicit directory object.
#[derive(Clone, Copy, Debug, Eq, Facet, PartialEq)]
#[repr(C)]
pub enum EntraUserSubject {
    SignedIn,
    Directory(EntraUserId),
}

impl EntraUserSubject {
    pub(crate) fn overview(self) -> PortalTabTarget {
        match self {
            Self::SignedIn => PortalTabTarget::Page(PortalPage::EntraMyUser),
            Self::Directory(id) => PortalTabTarget::EntraUser(id),
        }
    }

    pub(crate) fn page(self) -> PortalPage {
        match self {
            Self::SignedIn => PortalPage::EntraMyUser,
            Self::Directory(_) => PortalPage::EntraUsers,
        }
    }

    pub(crate) fn user_id(self, directory: &EntraDirectory) -> Option<EntraUserId> {
        match self {
            Self::Directory(id) => Some(id),
            Self::SignedIn => directory.my_user.as_loaded().map(|user| user.id),
        }
    }
}
