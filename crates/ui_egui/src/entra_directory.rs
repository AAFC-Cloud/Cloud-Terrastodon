use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::EntraApplicationObjectId;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use cloud_terrastodon_azure::EntraApplicationRegistrationGetRequest;
use cloud_terrastodon_azure::EntraApplicationRegistrationListRequest;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupGetRequest;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupListRequest;
use cloud_terrastodon_azure::EntraUser;
use cloud_terrastodon_azure::EntraUserGetRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::ExecutableRequest;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

/// Graph inventories and detail requests admitted for the current desktop account.
#[derive(Default)]
pub(crate) struct EntraDirectory {
    pub users: Loadable<Arc<Vec<Arc<EntraUser>>>>,
    pub groups: Loadable<Arc<Vec<Arc<EntraGroup>>>>,
    pub app_registrations: Loadable<Vec<Arc<EntraApplicationRegistration>>>,
    pub user_details: HashMap<EntraUserId, Loadable<Arc<EntraUser>>>,
    pub group_details: HashMap<EntraGroupId, Loadable<Arc<EntraGroup>>>,
    pub app_registration_details:
        HashMap<EntraApplicationObjectId, Loadable<Arc<EntraApplicationRegistration>>>,
}

impl EntraDirectory {
    pub fn is_loading(&self) -> bool {
        matches!(self.users, Loadable::Loading { .. })
            || matches!(self.groups, Loadable::Loading { .. })
            || is_loading(&self.app_registrations)
            || self.user_details.values().any(is_loading)
            || self.group_details.values().any(is_loading)
            || self.app_registration_details.values().any(is_loading)
    }

    pub fn can_refresh(&self, target: &PortalTabTarget) -> bool {
        match target {
            PortalTabTarget::Page(PortalPage::EntraUsers) => !is_loading(&self.users),
            PortalTabTarget::Page(PortalPage::EntraGroups) => !is_loading(&self.groups),
            PortalTabTarget::Page(PortalPage::EntraAppRegistrations) => {
                !is_loading(&self.app_registrations)
            }
            PortalTabTarget::EntraUser(id) => !self.user_details.get(id).is_some_and(is_loading),
            PortalTabTarget::EntraGroup(id) => !self.group_details.get(id).is_some_and(is_loading),
            PortalTabTarget::EntraAppRegistration(id) => !self
                .app_registration_details
                .get(id)
                .is_some_and(is_loading),
            _ => false,
        }
    }

    pub fn user_name(&self, id: EntraUserId) -> Option<&str> {
        self.user_details
            .get(&id)
            .and_then(Loadable::as_loaded)
            .or_else(|| self.users.as_loaded()?.iter().find(|user| user.id == id))
            .map(|user| user.display_name.as_str())
    }

    pub fn group_name(&self, id: EntraGroupId) -> Option<&str> {
        self.group_details
            .get(&id)
            .and_then(Loadable::as_loaded)
            .or_else(|| self.groups.as_loaded()?.iter().find(|group| group.id == id))
            .map(|group| group.display_name.as_str())
    }

    pub fn app_registration_name(&self, id: EntraApplicationObjectId) -> Option<&str> {
        self.app_registration_details
            .get(&id)
            .and_then(Loadable::as_loaded)
            .or_else(|| {
                self.app_registrations
                    .as_loaded()?
                    .iter()
                    .find(|registration| registration.id == id)
            })
            .map(|registration| registration.display_name.as_str())
    }
}

impl MyApp {
    pub(crate) fn load_entra_directory(&mut self, target: &PortalTabTarget, refresh: bool) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        match *target {
            PortalTabTarget::Page(PortalPage::EntraUsers) => self.load_entra_data(
                EntraUserListRequest { authentication },
                refresh,
                "Load Entra users",
                |directory| &mut directory.users,
            ),
            PortalTabTarget::Page(PortalPage::EntraGroups) => self.load_entra_data(
                EntraGroupListRequest { authentication },
                refresh,
                "Load Entra groups",
                |directory| &mut directory.groups,
            ),
            PortalTabTarget::Page(PortalPage::EntraAppRegistrations) => self.load_entra_data(
                EntraApplicationRegistrationListRequest { authentication },
                refresh,
                "Load Entra app registrations",
                |directory| &mut directory.app_registrations,
            ),
            PortalTabTarget::EntraUser(id) => self.load_entra_data(
                EntraUserGetRequest {
                    lookup: id.into(),
                    authentication,
                },
                refresh,
                "Load Entra user",
                move |directory| directory.user_details.entry(id).or_default(),
            ),
            PortalTabTarget::EntraGroup(id) => self.load_entra_data(
                EntraGroupGetRequest {
                    group_id: id,
                    authentication,
                },
                refresh,
                "Load Entra group",
                move |directory| directory.group_details.entry(id).or_default(),
            ),
            PortalTabTarget::EntraAppRegistration(id) => self.load_entra_data(
                EntraApplicationRegistrationGetRequest {
                    application_object_id: id,
                    authentication,
                },
                refresh,
                "Load Entra app registration",
                move |directory| directory.app_registration_details.entry(id).or_default(),
            ),
            _ => {}
        }
    }

    fn load_entra_data<R>(
        &mut self,
        request: R,
        refresh: bool,
        description: &'static str,
        field: impl Fn(&mut EntraDirectory) -> &mut Loadable<R::Output> + Copy + Send + Sync + 'static,
    ) where
        R: CacheInvalidatableRequest + ExecutableRequest<Error = eyre::Report> + 'static,
        R::Output: Debug + 'static,
    {
        match field(&mut self.azure.entra_directory) {
            Loadable::Loading { .. } => return,
            Loadable::NotLoaded => {}
            _ if !refresh => return,
            _ => {}
        }
        let generation = self.azure.account_generation;
        let executor = self.executor.clone();
        let cancellation = executor.cancellation().clone();
        let work_cache = self.work_cache.clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, R::Output>::new()
            .description(description)
            .setter(move |app, load| {
                if let Some(azure) = app.azure_content_mut(generation) {
                    *field(&mut azure.entra_directory) = load;
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                request.execute(&executor).with_invalidation(refresh).await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Err(error) = queued {
            *field(&mut self.azure.entra_directory) = Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            };
        }
    }
}

fn is_loading<T>(result: &Loadable<T>) -> bool {
    matches!(result, Loadable::Loading { .. })
}
