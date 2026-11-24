use crate::icons::page_icon;
use crate::icons::resource_icon;
use crate::portal_page::PortalPage;
use crate::reflected_value::ReflectedValue;
use crate::scope_iam_ancestry::ScopeIamAncestry;
use crate::scope_iam_thumbnails::scope_iam_thumbnail;
use crate::scope_role_assignment_principal_link::ScopeRoleAssignmentPrincipalLink;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraPrincipal;
use cloud_terrastodon_azure::EntraPrincipalLookup;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::ResourceType;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::RoleAssignmentPrincipalType;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::ScopeImpl;
use cloud_terrastodon_azure::SubscriptionId;
use cloud_terrastodon_azure::UserPrincipalName;
use eframe::egui::ImageSource;
use eframe::egui::Ui;
use eframe::egui::include_image;
use facet::Facet;
use std::sync::Arc;

/// Display fields for a role assignment from the Resource Graph inventory.
#[derive(Facet)]
pub(crate) struct ScopeRoleAssignmentRow {
    pub assignment: Arc<RoleAssignment>,
    pub principal_name: String,
    pub principal_id: String,
    pub principal_detail: String,
    pub principal_email: Option<String>,
    pub principal_type: String,
    pub principal_link: Option<ScopeRoleAssignmentPrincipalLink>,
    pub role: String,
    pub inheritance: String,
    pub scope: ScopeImpl,
    pub scope_label: String,
    pub condition: String,
}

impl ScopeRoleAssignmentRow {
    pub fn new(
        assignment: Arc<RoleAssignment>,
        role: String,
        ancestry: &ScopeIamAncestry,
        scope_name: Option<&str>,
    ) -> Self {
        let scope = assignment.scope.clone();
        let inheritance = if ancestry.is_selected(&scope.expanded_form()) {
            "Direct"
        } else {
            "Inherited"
        };
        let principal_id = assignment.principal_id.to_string();
        let principal_link = match Self::principal_lookup(&assignment) {
            Some(EntraPrincipalLookup::User(id)) => {
                Some(ScopeRoleAssignmentPrincipalLink::User(id))
            }
            Some(EntraPrincipalLookup::Group(id)) => {
                Some(ScopeRoleAssignmentPrincipalLink::Group(id))
            }
            _ => None,
        };
        let principal_type = match &assignment.principal_type {
            Some(RoleAssignmentPrincipalType::ServicePrincipal) => "Service principal".into(),
            Some(RoleAssignmentPrincipalType::ForeignGroup) => "Foreign group".into(),
            Some(RoleAssignmentPrincipalType::AgentUser) => "Agent user".into(),
            Some(RoleAssignmentPrincipalType::AgentServicePrincipal) => {
                "Agent service principal".into()
            }
            Some(kind) => String::from(kind),
            None => "—".into(),
        };
        Self {
            principal_name: principal_id.clone(),
            principal_detail: principal_id.clone(),
            principal_id,
            principal_email: None,
            principal_type,
            principal_link,
            role,
            inheritance: inheritance.into(),
            scope_label: scope_name.map_or_else(|| scope.short_form(), str::to_owned),
            scope,
            condition: assignment.condition.as_deref().unwrap_or("—").into(),
            assignment,
        }
    }

    pub fn principal_lookup(assignment: &RoleAssignment) -> Option<EntraPrincipalLookup> {
        let id = *assignment.principal_id.as_ref();
        Some(match &assignment.principal_type {
            Some(RoleAssignmentPrincipalType::User | RoleAssignmentPrincipalType::AgentUser) => {
                EntraPrincipalLookup::User(EntraUserId::new(id))
            }
            Some(RoleAssignmentPrincipalType::Group) => {
                EntraPrincipalLookup::Group(EntraGroupId::new(id))
            }
            Some(
                RoleAssignmentPrincipalType::ServicePrincipal
                | RoleAssignmentPrincipalType::AgentServicePrincipal,
            ) => EntraPrincipalLookup::ServicePrincipal(EntraServicePrincipalObjectId::new(id)),
            Some(RoleAssignmentPrincipalType::Unknown(_)) | None => assignment.principal_id.into(),
            _ => return None,
        })
    }

    pub fn apply_principal(&mut self, principal: &EntraPrincipal) {
        let (name, kind): (&str, &str) = match principal {
            EntraPrincipal::User(user) => {
                self.principal_link = Some(ScopeRoleAssignmentPrincipalLink::User(user.id));
                self.principal_email = user.mail.clone().filter(|mail| !mail.trim().is_empty());
                (&user.display_name, "User")
            }
            EntraPrincipal::Group(group) => {
                self.principal_link = Some(ScopeRoleAssignmentPrincipalLink::Group(group.id));
                (group.display_name.as_str(), "Group")
            }
            EntraPrincipal::ServicePrincipal(principal)
                if principal.service_principal_type == "ManagedIdentity" =>
            {
                // Graph's alternativeNames can include the complete ARM ID of the
                // user-assigned identity or the resource hosting a system identity.
                // https://learn.microsoft.com/graph/api/resources/serviceprincipal?view=graph-rest-1.0#properties
                self.principal_link =
                    managed_identity_resource(&principal.alternative_names).map(|id| {
                        self.principal_detail = id.expanded_form();
                        ScopeRoleAssignmentPrincipalLink::ManagedResource(id)
                    });
                (&principal.display_name, "Managed identity")
            }
            EntraPrincipal::ServicePrincipal(principal) => {
                (&principal.display_name, "Service principal")
            }
        };
        // Graph display names can be populated from email addresses. The name
        // column remains displayable in demo mode, so use the object ID there.
        if !name.trim().is_empty()
            && !name
                .split_whitespace()
                .any(|part| part.parse::<UserPrincipalName>().is_ok())
        {
            name.clone_into(&mut self.principal_name);
        }
        if !matches!(
            self.principal_type.as_str(),
            "Agent user" | "Agent service principal"
        ) {
            self.principal_type = kind.into();
        }
    }

    pub fn icon(ui: &Ui, row: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
        let row = row.get::<Self>()?;
        match &row.principal_link {
            Some(ScopeRoleAssignmentPrincipalLink::User(id)) => {
                return scope_iam_thumbnail(ui, *id)
                    .or_else(|| Some(page_icon(PortalPage::EntraUsers)));
            }
            Some(ScopeRoleAssignmentPrincipalLink::Group(_)) => {
                return Some(page_icon(PortalPage::EntraGroups));
            }
            _ => {}
        }
        match row.principal_type.as_str() {
            "Service principal" | "Agent service principal" => {
                Some(include_image!("../assets/iam-service-principal.svg"))
            }
            "Managed identity" => Some(resource_icon(
                &ResourceType::MICROSOFT_DOT_MANAGEDIDENTITY_SLASH_USERASSIGNEDIDENTITIES,
            )),
            _ => None,
        }
    }

    pub fn matches(&self, filter: &str, demo_mode: bool) -> bool {
        self.principal_name.to_lowercase().contains(filter)
            || self.principal_id.to_lowercase().contains(filter)
            || self.principal_type.to_lowercase().contains(filter)
            || (!demo_mode
                && self
                    .principal_email
                    .as_ref()
                    .is_some_and(|email| email.to_lowercase().contains(filter)))
            || self.role.to_lowercase().contains(filter)
            || self.inheritance.to_lowercase().contains(filter)
            || self.scope_label.to_lowercase().contains(filter)
            || self.scope.expanded_form().to_lowercase().contains(filter)
    }
}

fn managed_identity_resource(alternative_names: &[String]) -> Option<ScopeImpl> {
    let mut known_id: Option<&str> = None;
    'names: for name in alternative_names {
        let mut path = name.split('/');
        let (
            Some(""),
            Some(subscriptions),
            Some(subscription),
            Some(resource_groups),
            Some(group),
            Some(providers),
            Some(namespace),
            Some(kind),
            Some(resource),
        ) = (
            path.next(),
            path.next(),
            path.next(),
            path.next(),
            path.next(),
            path.next(),
            path.next(),
            path.next(),
            path.next(),
        )
        else {
            continue;
        };
        if !subscriptions.eq_ignore_ascii_case("subscriptions")
            || subscription.parse::<SubscriptionId>().is_err()
            || !resource_groups.eq_ignore_ascii_case("resourceGroups")
            || !providers.eq_ignore_ascii_case("providers")
            || [group, namespace, kind, resource]
                .iter()
                .any(|part| ["", ".", ".."].contains(part))
        {
            continue;
        }
        let mut remaining = path;
        while let Some(kind) = remaining.next() {
            if ["", ".", ".."].contains(&kind)
                || remaining
                    .next()
                    .is_none_or(|part| ["", ".", ".."].contains(&part))
            {
                continue 'names;
            }
        }
        if known_id.is_some_and(|known| !known.eq_ignore_ascii_case(name)) {
            return None;
        }
        known_id = Some(name);
    }
    known_id.and_then(|id| id.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_identity_retains_its_complete_resource_path() {
        let prefix = "/subscriptions/00000000-0000-0000-0000-000000000001/resourceGroups/synthetic/providers";
        let id =
            format!("{prefix}/Microsoft.Compute/virtualMachineScaleSets/sample/virtualMachines/1");
        let incomplete =
            format!("{prefix}/Microsoft.Compute/virtualMachineScaleSets/sample/virtualMachines");
        let actual =
            managed_identity_resource(&["isExplicit=False".into(), incomplete, id.clone()]);
        assert_eq!(actual.map(|scope| scope.expanded_form()), Some(id));
    }

    #[test]
    fn managed_identity_with_multiple_resource_ids_does_not_guess() {
        let prefix = "/subscriptions/00000000-0000-0000-0000-000000000001/resourceGroups/synthetic/providers/Microsoft.ManagedIdentity/userAssignedIdentities";
        let names = [format!("{prefix}/first"), format!("{prefix}/second")];
        assert!(managed_identity_resource(&names).is_none());
    }
}
