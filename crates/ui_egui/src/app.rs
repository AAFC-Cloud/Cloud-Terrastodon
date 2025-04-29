use crate::app_message::AppMessage;
use crate::autosave_info::AutoSaveBehaviour;
use crate::loadable::Loadable;
use crate::work_tracker::WorkTracker;
use cloud_terrastodon_core_azure::prelude::ResourceGroupMap;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevOpsProject;
use cloud_terrastodon_core_config::egui_config::EguiConfig;
use cloud_terrastodon_core_config::iconfig::IConfig;
use cloud_terrastodon_core_config::work_dirs_config::WorkDirsConfig;
use eframe::App;
use eframe::egui::Id;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

#[derive(Debug)]
pub struct MyApp {
    pub toggle_intents: HashSet<Id>,
    pub checkboxes: HashMap<Id, bool>,
    pub subscriptions: Loadable<Rc<Vec<Subscription>>, eyre::ErrReport>,
    pub azure_devops_projects: Loadable<Rc<Vec<AzureDevOpsProject>>, eyre::ErrReport>,
    pub resource_groups: Loadable<Rc<ResourceGroupMap>, eyre::ErrReport>,
    pub tx: UnboundedSender<AppMessage>,
    pub rx: UnboundedReceiver<AppMessage>,
    pub egui_config: EguiConfig,
    pub egui_config_auto_save: AutoSaveBehaviour<EguiConfig>,
    pub work_dirs_config: WorkDirsConfig,
    pub work_dirs_config_auto_save: AutoSaveBehaviour<WorkDirsConfig>,
    pub work_tracker: Rc<WorkTracker>,
}

impl MyApp {
    pub async fn new(
        _cc: &eframe::CreationContext<'_>,
        work_tracker: Rc<WorkTracker>,
    ) -> eyre::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();
        Ok(Self {
            toggle_intents: Default::default(),
            checkboxes: Default::default(),
            subscriptions: Default::default(),
            resource_groups: Default::default(),
            azure_devops_projects: Default::default(),
            tx,
            rx,
            egui_config: EguiConfig::load().await?,
            egui_config_auto_save: Default::default(),
            work_dirs_config: WorkDirsConfig::load().await?,
            work_dirs_config_auto_save: Default::default(),
            work_tracker,
        })
    }
    pub fn checkbox_for(&mut self, key: impl Hash) -> &mut bool {
        self.checkboxes.entry(Id::new(key)).or_default()
    }
    pub fn enqueue_auto_save(&mut self) {
        self.egui_config_auto_save
            .apply(&self.egui_config, self.work_tracker.clone());
        self.work_dirs_config_auto_save
            .apply(&self.work_dirs_config, self.work_tracker.clone());
    }
    pub fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::StateChange(state_mutator) => {
                    state_mutator.mutate_state(self);
                }
            }
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();
        self.draw_app(ctx);
        self.enqueue_auto_save();
        self.work_tracker.prune();
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("Gracefully exiting");
        self.enqueue_auto_save();
    }
}
