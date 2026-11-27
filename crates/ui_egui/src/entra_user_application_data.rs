use crate::app::MyApp;
use cloud_terrastodon_azure::EntraAppRoleAssignment;
use cloud_terrastodon_azure::EntraAppRoleAssignmentId;
use cloud_terrastodon_azure::EntraOwnedApplicationRegistration;
use cloud_terrastodon_azure::EntraServicePrincipalAppRoleAssignmentListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalAppRoleCatalogue;
use cloud_terrastodon_azure::EntraServicePrincipalApplicationRoleCatalogueGetRequest;
use cloud_terrastodon_azure::EntraServicePrincipalOAuth2PermissionGrantListRequest;
use cloud_terrastodon_azure::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure::EntraUserApplicationAssignmentListRequest;
use cloud_terrastodon_azure::EntraUserApplicationOwnershipListRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::OAuth2PermissionGrant;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::sync::Arc;

/// Application inventories and the selected assignment's supplemental details for one tab account.
#[derive(Default)]
pub(crate) struct EntraUserApplicationData {
    pub ownership: HashMap<EntraUserId, Loadable<Vec<Arc<EntraOwnedApplicationRegistration>>>>,
    pub assignments: HashMap<EntraUserId, Loadable<Vec<Arc<EntraAppRoleAssignment>>>>,
    pub selected_assignments: HashMap<EntraUserId, EntraAppRoleAssignmentId>,
    pub role_catalogues: HashMap<
        EntraServicePrincipalObjectId,
        Loadable<Arc<EntraServicePrincipalAppRoleCatalogue>>,
    >,
    pub application_permissions:
        HashMap<EntraServicePrincipalObjectId, Loadable<Vec<Arc<EntraAppRoleAssignment>>>>,
    pub delegated_permissions:
        HashMap<EntraServicePrincipalObjectId, Loadable<Vec<Arc<OAuth2PermissionGrant>>>>,
}

impl EntraUserApplicationData {
    pub fn is_loading(&self) -> bool {
        self.ownership.values().any(is_loading)
            || self.assignments.values().any(is_loading)
            || self.role_catalogues.values().any(is_loading)
            || self.application_permissions.values().any(is_loading)
            || self.delegated_permissions.values().any(is_loading)
    }

    pub fn can_refresh_ownership(&self, user_id: EntraUserId) -> bool {
        !self.ownership.get(&user_id).is_some_and(is_loading)
    }

    pub fn can_refresh_assignments(&self, user_id: EntraUserId) -> bool {
        !self.assignments.get(&user_id).is_some_and(is_loading)
    }
}

impl MyApp {
    pub(crate) fn load_entra_user_application_ownership(
        &mut self,
        user_id: EntraUserId,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        self.load_entra_data(
            EntraUserApplicationOwnershipListRequest {
                authentication,
                user_id,
            },
            refresh,
            "Load Entra application ownership",
            move |azure| {
                azure
                    .entra_user_applications
                    .ownership
                    .entry(user_id)
                    .or_default()
            },
        );
    }

    pub(crate) fn load_entra_user_application_assignments(
        &mut self,
        user_id: EntraUserId,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        if refresh {
            let data = &self.azure.entra_user_applications;
            let application = data
                .selected_assignments
                .get(&user_id)
                .zip(data.assignments.get(&user_id).and_then(Loadable::as_loaded))
                .and_then(|(selected, assignments)| {
                    assignments
                        .iter()
                        .find(|assignment| assignment.id == *selected)
                })
                .map(|assignment| assignment.resource_id);
            if let Some(resource_id) = application {
                let permissions_loaded = data.application_permissions.contains_key(&resource_id);
                self.load_entra_assignment_role_catalogue(resource_id, true);
                if permissions_loaded {
                    self.load_entra_assignment_permissions(resource_id, true);
                }
            }
        }
        self.load_entra_data(
            EntraUserApplicationAssignmentListRequest {
                authentication,
                user_id,
            },
            refresh,
            "Load Entra application assignments",
            move |azure| {
                azure
                    .entra_user_applications
                    .assignments
                    .entry(user_id)
                    .or_default()
            },
        );
    }

    pub(crate) fn load_entra_assignment_role_catalogue(
        &mut self,
        service_principal_id: EntraServicePrincipalObjectId,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        self.load_entra_data(
            EntraServicePrincipalApplicationRoleCatalogueGetRequest {
                authentication,
                service_principal_id,
            },
            refresh,
            "Load application role definitions",
            move |azure| {
                azure
                    .entra_user_applications
                    .role_catalogues
                    .entry(service_principal_id)
                    .or_default()
            },
        );
    }

    pub(crate) fn load_entra_assignment_permissions(
        &mut self,
        service_principal_id: EntraServicePrincipalObjectId,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        self.load_entra_data(
            EntraServicePrincipalAppRoleAssignmentListRequest {
                authentication,
                service_principal_id,
            },
            refresh,
            "Load application permissions",
            move |azure| {
                azure
                    .entra_user_applications
                    .application_permissions
                    .entry(service_principal_id)
                    .or_default()
            },
        );
        self.load_entra_data(
            EntraServicePrincipalOAuth2PermissionGrantListRequest {
                authentication,
                service_principal_id,
            },
            refresh,
            "Load delegated permissions",
            move |azure| {
                azure
                    .entra_user_applications
                    .delegated_permissions
                    .entry(service_principal_id)
                    .or_default()
            },
        );
    }
}

fn is_loading<T>(result: &Loadable<T>) -> bool {
    matches!(result, Loadable::Loading { .. })
}
