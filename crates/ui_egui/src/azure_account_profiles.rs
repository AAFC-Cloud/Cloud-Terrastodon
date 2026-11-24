use crate::app::MyApp;
use crate::azure_account_profile::AzureAccountProfile;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountPrincipal;
use eframe::egui::Context;
use std::collections::HashMap;
use std::collections::VecDeque;
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

/// Retains recent profile cards for this GUI session. Each selection includes
/// the account, source and tenant. Replacing the provider drops this state.
#[derive(Default)]
pub(crate) struct AzureAccountProfiles {
    completed: VecDeque<(AzureBearerAuthenticationSelection, AzureAccountProfile)>,
    pending: HashMap<AzureBearerAuthenticationSelection, oneshot::Receiver<AzureAccountProfile>>,
}

impl AzureAccountProfiles {
    pub fn get(
        &self,
        selection: AzureBearerAuthenticationSelection,
    ) -> Option<&AzureAccountProfile> {
        self.completed
            .iter()
            .find_map(|(item, profile)| (*item == selection).then_some(profile))
    }

    fn finish(
        &mut self,
        selection: AzureBearerAuthenticationSelection,
        profile: AzureAccountProfile,
    ) {
        const MAX_CARDS: usize = 16;
        if self.completed.len() >= MAX_CARDS {
            self.completed.pop_front();
        }
        self.completed.push_back((selection, profile));
    }

    fn poll(&mut self) {
        let mut completed = Vec::new();
        self.pending.retain(|&selection, receiver| {
            let profile = match receiver.try_recv() {
                Ok(profile) => profile,
                Err(TryRecvError::Closed) => AzureAccountProfile::default(),
                Err(TryRecvError::Empty) => return true,
            };
            completed.push((selection, profile));
            false
        });
        for (selection, profile) in completed {
            self.finish(selection, profile);
        }
    }
}

impl MyApp {
    pub(crate) fn update_account_profile(&mut self, ctx: &Context) {
        let selection = self.ready_azure_authentication();
        self.account_profiles.poll();
        let Some(selection) = selection else {
            return;
        };
        if self.account_profiles.pending.contains_key(&selection)
            || self.account_profiles.get(selection).is_some()
        {
            return;
        }
        let Some(CloudTerrastodonAccountPrincipal::EntraUser(expected_user)) = self
            .account_picker
            .catalogue
            .as_ref()
            .ok()
            .and_then(|catalogue| catalogue.by_id(selection.account_id).ok())
            .and_then(|account| account.principal)
        else {
            return;
        };

        let (mut send, receive) = oneshot::channel();
        let ctx = ctx.clone();
        let executor = self.executor.clone();
        let authentication_provider = self.account_authentication.clone();
        let task = self.work_tracker.spawn(
            self.executor.cancellation(),
            &Handle::current(),
            "Load account profile".to_owned(),
            async move |scope| {
                let executor = executor.with_cancellation_scope(scope);
                let (cancellation, load) = executor.cancellable_child_scope(async |executor| {
                    AzureAccountProfile::load(
                        &executor,
                        &authentication_provider,
                        selection,
                        expected_user,
                        &ctx,
                    )
                    .await
                });
                tokio::pin!(load);
                let profile = tokio::select! {
                    biased;
                    () = send.closed() => {
                        cancellation.request_cancel("account profile no longer needed");
                        let _ = load.await;
                        return Ok(());
                    }
                    result = &mut load => result.unwrap_or_default(),
                };
                let _ = send.send(profile);
                ctx.request_repaint();
                Ok(())
            },
        );
        if task.is_ok() {
            self.account_profiles.pending.insert(selection, receive);
        } else {
            self.account_profiles
                .finish(selection, AzureAccountProfile::default());
        }
    }
}
