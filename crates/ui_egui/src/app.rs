use crate::autosave_info::AutoSaveBehaviour;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::ResourceGroupMap;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_command::app_work::AppboundWorkFinishedMessage;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_config::EguiConfig;
use cloud_terrastodon_config::WorkDirsConfig;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
// use cloud_terrastodon_tracing::event_collector; // TODO(EGUI-TRACING)
use eframe::App;
use eframe::egui::Align2;
use eframe::egui::Direction;
use eframe::egui::Id;
use egui_toast::Toasts;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;
use tracing::info;

pub struct MyApp {
    pub executor: CloudTerrastodonExecutor,
    pub tenant_id: AzureTenantId,
    pub auth_context: AuthContext,
    pub toggle_intents: HashSet<Id>,
    pub checkboxes: HashMap<Id, bool>,
    pub subscriptions: Loadable<Rc<Vec<Arc<Subscription>>>, eyre::ErrReport>,
    pub azure_devops_projects: Loadable<Arc<Vec<Arc<AzureDevOpsProject>>>, eyre::ErrReport>,
    pub resource_groups: Loadable<Rc<ResourceGroupMap>, eyre::ErrReport>,
    pub resources: Loadable<Arc<Vec<Arc<Resource>>>, eyre::ErrReport>,
    pub tx: UnboundedSender<AppboundWorkFinishedMessage<MyApp>>,
    pub rx: UnboundedReceiver<AppboundWorkFinishedMessage<MyApp>>,
    pub egui_config: EguiConfig,
    pub egui_config_auto_save: AutoSaveBehaviour<EguiConfig>,
    pub work_dirs_config: WorkDirsConfig,
    pub work_dirs_config_auto_save: AutoSaveBehaviour<WorkDirsConfig>,
    /// Whether the About window is open
    pub about_open: bool,
    /// Whether the Logs window is visible
    pub logs_visible: bool,
    /// Toasts manager for transient notifications
    pub toasts: Toasts,
    /// Number of events we've already processed for toasts
    pub last_seen_event_count: usize,
    pub work_tracker: AppWorkTracker,
    /// Transfers final settings to the owner for saving after earlier jobs drain.
    pub shutdown_settings: SyncSender<(EguiConfig, WorkDirsConfig)>,
    /// Formatted application info (version + revision) shown in About window
    pub app_info: String,

    pub tiles: egui_tiles::Tree<crate::tiles::Pane>,

    pub tiles_behavior: crate::tiles::TreeBehavior,
}

impl MyApp {
    pub async fn new(
        tenant_id: AzureTenantId,
        auth_context: AuthContext,
        work_tracker: AppWorkTracker,
        shutdown_settings: SyncSender<(EguiConfig, WorkDirsConfig)>,
        app_info: String,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppboundWorkFinishedMessage<MyApp>>();
        Ok(Self {
            executor: executor.clone(),
            tenant_id,
            auth_context,
            toggle_intents: Default::default(),
            checkboxes: Default::default(),
            subscriptions: Default::default(),
            resource_groups: Default::default(),
            resources: Default::default(),
            azure_devops_projects: Default::default(),
            tx,
            rx,
            egui_config: EguiConfig::load().await?,
            egui_config_auto_save: Default::default(),
            work_dirs_config: WorkDirsConfig::load().await?,
            work_dirs_config_auto_save: Default::default(),
            about_open: false,
            logs_visible: false,
            toasts: Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(Direction::BottomUp),
            last_seen_event_count: 0, //event_collector().events().len(), // TODO(EGUI-TRACING)
            work_tracker,
            shutdown_settings,
            app_info,
            tiles: crate::tiles::create_tree(),
            tiles_behavior: Default::default(),
        })
    }
    pub fn checkbox_for(&mut self, key: impl Hash) -> &mut bool {
        self.checkboxes.entry(Id::new(key)).or_default()
    }
    pub fn enqueue_auto_save(&mut self) -> eyre::Result<()> {
        self.egui_config_auto_save.apply(
            &self.egui_config,
            &self.work_tracker,
            self.executor.cancellation(),
        )?;
        self.work_dirs_config_auto_save.apply(
            &self.work_dirs_config,
            &self.work_tracker,
            self.executor.cancellation(),
        )?;
        Ok(())
    }
    pub fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppboundWorkFinishedMessage::StateChange(state_mutator) => {
                    state_mutator.mutate_state(self);
                }
            }
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.executor.cancellation().is_cancelled() {
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
        self.handle_messages();
        self.draw_app(ctx);
        if let Err(error) = self.enqueue_auto_save() {
            error!(?error, "Could not enqueue configuration autosave");
        }
        if let Err(error) = self.work_tracker.prune() {
            error!(?error, "Could not inspect background work");
        }
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("Gracefully exiting");
        if let Err(error) = self
            .shutdown_settings
            .send((self.egui_config.clone(), self.work_dirs_config.clone()))
        {
            error!(?error, "Could not hand off final configuration for saving");
        }
        self.executor
            .cancellation()
            .request_cancel("GUI window closed");
    }
}
