use crate::app::MyApp;
use crate::entra_directory_inventory::EntraDirectoryInventory;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::EntraApplicationObjectId;
use cloud_terrastodon_azure::EntraApplicationRegistration;
use cloud_terrastodon_azure::EntraApplicationRegistrationGetRequest;
use cloud_terrastodon_azure::EntraApplicationRegistrationListRequest;
use cloud_terrastodon_azure::EntraApplicationRegistrationSearchRequest;
use cloud_terrastodon_azure::EntraCurrentUserRequest;
use cloud_terrastodon_azure::EntraGroup;
use cloud_terrastodon_azure::EntraGroupGetRequest;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraGroupListRequest;
use cloud_terrastodon_azure::EntraGroupSearchRequest;
use cloud_terrastodon_azure::EntraUser;
use cloud_terrastodon_azure::EntraUserGetRequest;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure::EntraUserListRequest;
use cloud_terrastodon_azure::EntraUserSearchRequest;
use cloud_terrastodon_azure::MicrosoftGraphResponse;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::CacheInvalidatableRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionEvent;
use eframe::egui::Context;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Instant;
use teamy_cancellation::CancellationToken;

/// Graph inventories and detail requests admitted for the current desktop account.
#[derive(Default)]
pub(crate) struct EntraDirectory {
    pub users: EntraDirectoryInventory<EntraUser, EntraUserId>,
    pub groups: EntraDirectoryInventory<EntraGroup, EntraGroupId>,
    pub app_registrations:
        EntraDirectoryInventory<EntraApplicationRegistration, EntraApplicationObjectId>,
    pub my_user: Loadable<Arc<EntraUser>>,
    pub user_details: HashMap<EntraUserId, Loadable<Arc<EntraUser>>>,
    pub group_details: HashMap<EntraGroupId, Loadable<Arc<EntraGroup>>>,
    pub app_registration_details:
        HashMap<EntraApplicationObjectId, Loadable<Arc<EntraApplicationRegistration>>>,
}

impl EntraDirectory {
    pub fn is_loading(&self) -> bool {
        self.users.is_loading()
            || self.groups.is_loading()
            || self.app_registrations.is_loading()
            || is_loading(&self.my_user)
            || self.user_details.values().any(is_loading)
            || self.group_details.values().any(is_loading)
            || self.app_registration_details.values().any(is_loading)
    }

    pub fn can_refresh(&self, target: &PortalTabTarget) -> bool {
        match target {
            PortalTabTarget::Page(PortalPage::EntraUsers) => !is_loading(&self.users.inventory),
            PortalTabTarget::Page(PortalPage::EntraGroups) => !is_loading(&self.groups.inventory),
            PortalTabTarget::Page(PortalPage::EntraAppRegistrations) => {
                !is_loading(&self.app_registrations.inventory)
            }
            PortalTabTarget::Page(PortalPage::EntraMyUser) => !is_loading(&self.my_user),
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
            .map(Arc::as_ref)
            .or_else(|| self.users.find(id, |user| user.id))
            .map(|user| user.display_name.as_str())
    }

    pub fn group_name(&self, id: EntraGroupId) -> Option<&str> {
        self.group_details
            .get(&id)
            .and_then(Loadable::as_loaded)
            .map(Arc::as_ref)
            .or_else(|| self.groups.find(id, |group| group.id))
            .map(|group| group.display_name.as_str())
    }

    pub fn app_registration_name(&self, id: EntraApplicationObjectId) -> Option<&str> {
        self.app_registration_details
            .get(&id)
            .and_then(Loadable::as_loaded)
            .map(Arc::as_ref)
            .or_else(|| {
                self.app_registrations
                    .find(id, |registration| registration.id)
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
            PortalTabTarget::Page(PortalPage::EntraUsers) => self.load_entra_inventory(
                EntraUserListRequest { authentication },
                refresh,
                "Load Entra users",
                |directory| &mut directory.users,
                std::convert::identity,
            ),
            PortalTabTarget::Page(PortalPage::EntraGroups) => self.load_entra_inventory(
                EntraGroupListRequest { authentication },
                refresh,
                "Load Entra groups",
                |directory| &mut directory.groups,
                std::convert::identity,
            ),
            PortalTabTarget::Page(PortalPage::EntraAppRegistrations) => self.load_entra_inventory(
                EntraApplicationRegistrationListRequest { authentication },
                refresh,
                "Load Entra app registrations",
                |directory| &mut directory.app_registrations,
                Arc::new,
            ),
            PortalTabTarget::Page(PortalPage::EntraMyUser) => self.load_entra_data(
                EntraCurrentUserRequest { authentication },
                refresh,
                "Load current Entra user",
                |azure| &mut azure.entra_directory.my_user,
            ),
            PortalTabTarget::EntraUser(id) => self.load_entra_data(
                EntraUserGetRequest {
                    lookup: id.into(),
                    authentication,
                },
                refresh,
                "Load Entra user",
                move |azure| azure.entra_directory.user_details.entry(id).or_default(),
            ),
            PortalTabTarget::EntraGroup(id) => self.load_entra_data(
                EntraGroupGetRequest {
                    group_id: id,
                    authentication,
                },
                refresh,
                "Load Entra group",
                move |azure| azure.entra_directory.group_details.entry(id).or_default(),
            ),
            PortalTabTarget::EntraAppRegistration(id) => self.load_entra_data(
                EntraApplicationRegistrationGetRequest {
                    application_object_id: id,
                    authentication,
                },
                refresh,
                "Load Entra app registration",
                move |azure| {
                    azure
                        .entra_directory
                        .app_registration_details
                        .entry(id)
                        .or_default()
                },
            ),
            _ => {}
        }
    }

    pub(crate) fn search_entra_directory(&mut self, page: PortalPage, term: &str, ctx: &Context) {
        let Some(authentication) = self.ready_azure_authentication() else {
            return;
        };
        match page {
            PortalPage::EntraUsers => self.search_entra_inventory(
                term,
                ctx,
                "Search Entra users",
                |directory| &mut directory.users,
                |user| user.id,
                |search_term| EntraUserSearchRequest {
                    authentication,
                    search_term,
                    exact_display_name: true,
                },
            ),
            PortalPage::EntraGroups => self.search_entra_inventory(
                term,
                ctx,
                "Search Entra groups",
                |directory| &mut directory.groups,
                |group| group.id,
                |search_term| EntraGroupSearchRequest {
                    authentication,
                    search_term,
                },
            ),
            PortalPage::EntraAppRegistrations => self.search_entra_inventory(
                term,
                ctx,
                "Search Entra app registrations",
                |directory| &mut directory.app_registrations,
                |application| application.id,
                |search_term| EntraApplicationRegistrationSearchRequest {
                    authentication,
                    search_term,
                    exact_display_name: false,
                },
            ),
            _ => {}
        }
    }

    fn load_entra_inventory<R, T, I>(
        &mut self,
        request: R,
        refresh: bool,
        description: &'static str,
        field: impl Fn(&mut EntraDirectory) -> &mut EntraDirectoryInventory<T, I>
        + Copy
        + Send
        + Sync
        + 'static,
        project: impl FnOnce(R::Output) -> Arc<Vec<Arc<T>>> + Send + 'static,
    ) where
        R: CacheInvalidatableRequest + ExecutableRequest<Error = eyre::Report> + 'static,
        T: Debug + Send + Sync + 'static,
        I: Copy + Eq + Hash,
    {
        let inventory = field(&mut self.azure.entra_directory);
        if !should_load(&inventory.inventory, refresh) {
            return;
        }
        if refresh {
            inventory.refresh_lookup();
        }
        let (send, pages) = tokio::sync::mpsc::unbounded_channel();
        inventory.pages = Some(pages);
        let _ = self.queue_entra_work(
            description,
            move |azure, load| field(&mut azure.entra_directory).set_inventory(load),
            async move |executor| {
                request
                    .execute(&executor)
                    .with_invalidation(refresh)
                    .observe(move |event| {
                        if let ExecutionEvent::CollectionPage { index, payload } = event
                            && let Ok(page) = payload.downcast::<MicrosoftGraphResponse<Arc<T>>>()
                        {
                            // The receiver is scoped to the tab's account selection.
                            let _ = send.send((index, page));
                        }
                    })
                    .await
                    .map(project)
            },
        );
    }

    fn search_entra_inventory<R, T, I>(
        &mut self,
        term: &str,
        ctx: &Context,
        description: &'static str,
        field: impl Fn(&mut EntraDirectory) -> &mut EntraDirectoryInventory<T, I>
        + Copy
        + Send
        + Sync
        + 'static,
        id: impl Fn(&T) -> I + Copy + Send + Sync + 'static,
        request: impl FnOnce(String) -> R,
    ) where
        R: CacheInvalidatableRequest
            + ExecutableRequest<Error = eyre::Report, Output = Vec<Arc<T>>>
            + 'static,
        T: Debug + Send + Sync + 'static,
        I: Copy + Eq + Hash,
    {
        let Some((lookup_generation, term, refresh)) =
            field(&mut self.azure.entra_directory).lookup_due(term, Instant::now(), ctx)
        else {
            return;
        };
        let request = request(term);
        let cancellation = self.queue_entra_work(
            description,
            move |azure, load| {
                field(&mut azure.entra_directory).set_lookup(lookup_generation, load, id)
            },
            async move |executor| request.execute(&executor).with_invalidation(refresh).await,
        );
        field(&mut self.azure.entra_directory).track_lookup(cancellation);
    }

    pub(crate) fn load_entra_data<R>(
        &mut self,
        request: R,
        refresh: bool,
        description: &'static str,
        field: impl Fn(&mut PortalAzureContent) -> &mut Loadable<R::Output>
        + Copy
        + Send
        + Sync
        + 'static,
    ) where
        R: CacheInvalidatableRequest + ExecutableRequest<Error = eyre::Report> + 'static,
        R::Output: Debug + 'static,
    {
        self.load_entra_data_map(request, refresh, description, field, std::convert::identity);
    }

    pub(crate) fn load_entra_data_map<R, T>(
        &mut self,
        request: R,
        refresh: bool,
        description: &'static str,
        field: impl Fn(&mut PortalAzureContent) -> &mut Loadable<T> + Copy + Send + Sync + 'static,
        project: impl FnOnce(R::Output) -> T + Send + 'static,
    ) where
        R: CacheInvalidatableRequest + ExecutableRequest<Error = eyre::Report> + 'static,
        T: Debug + Send + 'static,
    {
        if should_load(field(&mut self.azure), refresh) {
            let _ = self.queue_entra_work(
                description,
                move |azure, load| *field(azure) = load,
                async move |executor| {
                    request
                        .execute(&executor)
                        .with_invalidation(refresh)
                        .await
                        .map(project)
                },
            );
        }
    }

    pub(crate) fn queue_entra_work<T, F, Fut>(
        &mut self,
        description: &'static str,
        setter: impl Fn(&mut PortalAzureContent, Loadable<T>) + Copy + Send + Sync + 'static,
        work: F,
    ) -> Option<CancellationToken>
    where
        T: Debug + Send + 'static,
        F: FnOnce(CloudTerrastodonExecutor) -> Fut + Send + 'static,
        Fut: Future<Output = eyre::Result<T>> + Send + 'static,
    {
        let generation = self.azure.account_generation;
        let executor = self.executor.clone();
        let cancellation = executor.cancellation().clone();
        let work_cache = self.work_cache.clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<MyApp, T>::new()
            .description(description)
            .setter(move |app, load| {
                if let Some(azure) = app.azure_content_mut(generation) {
                    setter(azure, load);
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                work(executor).await
            })
            .build()
            .enqueue_cancellable(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        match queued {
            Ok(cancellation) => Some(cancellation),
            Err(error) => {
                setter(
                    &mut self.azure,
                    Loadable::Failed {
                        error: error.into(),
                        started_at,
                        finished_at: Instant::now(),
                    },
                );
                None
            }
        }
    }
}

fn is_loading<T>(result: &Loadable<T>) -> bool {
    matches!(result, Loadable::Loading { .. })
}

fn should_load<T>(result: &Loadable<T>, refresh: bool) -> bool {
    matches!(result, Loadable::NotLoaded) || (refresh && !is_loading(result))
}
