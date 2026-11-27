use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::reflected_value::ReflectedValue;
use crate::scope_role_assignment_principal_link::ScopeRoleAssignmentPrincipalLink;
use cloud_terrastodon_azure::EntraDirectoryObject;
use eframe::egui::ImageSource;
use eframe::egui::Ui;
use eframe::egui::include_image;
use facet::Facet;

/// A table entry for a direct member or owner, including limited-information Graph objects.
#[derive(Debug, Facet)]
pub(crate) struct EntraGroupDirectoryObjectRow {
    pub name: String,
    pub object_id: String,
    pub kind: String,
    pub principal_link: Option<ScopeRoleAssignmentPrincipalLink>,
}

impl EntraGroupDirectoryObjectRow {
    pub fn new(object: &EntraDirectoryObject) -> Self {
        let (kind, principal_link) = match object {
            EntraDirectoryObject::User(user) => (
                "User",
                Some(ScopeRoleAssignmentPrincipalLink::User(user.id)),
            ),
            EntraDirectoryObject::Group(group) => (
                "Group",
                Some(ScopeRoleAssignmentPrincipalLink::Group(group.id)),
            ),
            EntraDirectoryObject::ServicePrincipal(_) => ("Service principal", None),
            EntraDirectoryObject::Device(_) => ("Device", None),
            EntraDirectoryObject::OrganizationalContact(_) => ("Organizational contact", None),
        };
        let object_id = object.id().to_string();
        let name = match object {
            EntraDirectoryObject::User(user) => user.display_name.as_deref(),
            _ => None,
        }
        .or_else(|| object.display_name())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(&object_id)
        .to_owned();
        Self {
            name,
            object_id,
            kind: kind.to_owned(),
            principal_link,
        }
    }

    pub fn icon(_: &Ui, row: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
        match row.get::<Self>()?.kind.as_str() {
            "User" => Some(page_icon(PortalPage::EntraUsers)),
            "Group" => Some(page_icon(PortalPage::EntraGroups)),
            "Service principal" => Some(include_image!("../assets/iam-service-principal.svg")),
            _ => None,
        }
    }
}
