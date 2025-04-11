use std::collections::HashMap;
use std::collections::HashSet;

use crate::app_message::AppMessage;
use crate::loadable::Loadable;
use cloud_terrastodon_core_azure::prelude::ResourceGroup;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use eframe::egui::Id;
use eframe::App;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tracing::error;

#[derive(Debug)]
pub struct MyApp {
    pub toggle_intents: HashSet<Id>,
    pub checkboxes: HashMap<Id, bool>,
    pub subscriptions: Loadable<Vec<(bool, Subscription)>, eyre::ErrReport>,
    pub resource_groups: Loadable<HashMap<SubscriptionId, Vec<(bool, ResourceGroup)>>, eyre::ErrReport>,
    pub tx: UnboundedSender<AppMessage>,
    pub rx: UnboundedReceiver<AppMessage>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();
        Self {
            toggle_intents: Default::default(),
            checkboxes: Default::default(),
            subscriptions: Default::default(),
            resource_groups: Default::default(),
            tx,
            rx,
        }
    }
    pub fn try_thing<F, T>(&mut self, future: F) -> JoinHandle<F::Output>
    where
        F: Future<Output = eyre::Result<T>> + Send + 'static,
        F::Output: Send + 'static,
        T: Send + 'static,
    {
        let handle = tokio::runtime::Handle::current().spawn(async move {
            let result = future.await;
            if let Err(e) = &result {
                error!("Error in message thread: {:#?}", e)
            }
            result
        });
        handle
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::StateChange(state_mutator) => {
                    state_mutator.mutate_state(self);
                }
            }
        }
        self.draw_app(ctx);
    }
}
