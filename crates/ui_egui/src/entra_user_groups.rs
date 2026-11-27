use crate::app::MyApp;
use cloud_terrastodon_azure::EntraDirectoryObjectGroup;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupListByMemberRequest;
use cloud_terrastodon_azure::EntraGroupListByOwnerRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_command::app_work::Loadable;
use std::collections::HashMap;
use std::sync::Arc;

/// The group relationships loaded for a user with the tab's selected account.
#[derive(Default)]
pub(crate) struct EntraUserGroups {
    pub memberships: HashMap<EntraUserId, Loadable<Vec<Arc<EntraGroup>>>>,
    pub ownerships: HashMap<EntraUserId, Loadable<Vec<Arc<EntraDirectoryObjectGroup>>>>,
}

impl EntraUserGroups {
    pub(crate) fn is_loading(&self) -> bool {
        self.memberships.values().any(is_loading) || self.ownerships.values().any(is_loading)
    }

    pub(crate) fn can_refresh_memberships(&self, user_id: EntraUserId) -> bool {
        !self.memberships.get(&user_id).is_some_and(is_loading)
    }

    pub(crate) fn can_refresh_ownerships(&self, user_id: EntraUserId) -> bool {
        !self.ownerships.get(&user_id).is_some_and(is_loading)
    }
}

impl MyApp {
    pub(crate) fn load_entra_user_group_memberships(
        &mut self,
        user_id: EntraUserId,
        refresh: bool,
    ) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        self.load_entra_data(
            EntraGroupListByMemberRequest {
                principal_id: user_id.into(),
                authentication,
            },
            refresh,
            "Load Entra user group memberships",
            move |azure| {
                azure
                    .entra_user_groups
                    .memberships
                    .entry(user_id)
                    .or_default()
            },
        );
    }

    pub(crate) fn load_entra_user_group_ownerships(&mut self, user_id: EntraUserId, refresh: bool) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        self.load_entra_data(
            EntraGroupListByOwnerRequest {
                user_id,
                authentication,
            },
            refresh,
            "Load Entra user group ownerships",
            move |azure| {
                azure
                    .entra_user_groups
                    .ownerships
                    .entry(user_id)
                    .or_default()
            },
        );
    }
}

fn is_loading<T>(response: &Loadable<T>) -> bool {
    matches!(response, Loadable::Loading { .. })
}
