use crate::autosave_info::AutoSaveBehaviour;
use crate::azure_account_manager::AzureAccountManager;
use crate::azure_account_operation::AzureAccountOperation;
use crate::azure_account_picker::AzureAccountPicker;
use crate::azure_tenant_manager::AzureTenantManager;
use crate::gui_work_cache::GuiWorkCache;
use crate::portal_page::PortalPage;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureResourceGroupMap;
use cloud_terrastodon_azure::Resource;
use cloud_terrastodon_azure::Subscription;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_command::app_work::AppboundWorkFinishedMessage;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_config::EguiConfig;
use cloud_terrastodon_config::WorkDirsConfig;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eframe::App;
use eframe::egui::Align2;
use eframe::egui::Direction;
use eframe::egui::Id;
use eframe::egui::Ui;
use egui_toast::Toasts;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;
use std::time::Duration;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::capture::LogBuffer;
use teamy_tracing::gui::LogView;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;
use tracing::info;

pub struct MyApp {
    /// In standalone mode, this executor supplies the provider and scope while
    /// cache jobs bind their own active owner through `work_cache`.
    pub executor: CloudTerrastodonExecutor,
    pub(crate) work_cache: GuiWorkCache,
    pub azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    pub authentication: Option<AzureBearerAuthenticationSelection>,
    pub account_picker: AzureAccountPicker,
    pub(crate) account_store: CloudTerrastodonAccountStore,
    pub(crate) account_authentication: Arc<CredentialAuthenticationProvider>,
    pub(crate) account_manager: AzureAccountManager,
    pub(crate) account_operation: Option<AzureAccountOperation>,
    pub(crate) tenant_manager: AzureTenantManager,
    /// Shared bounded history owned by the application, not a second GUI collector.
    pub log_capture: LogBuffer,
    pub log_view: LogView,
    /// Results from an earlier selection must not replace the current account's data.
    pub account_generation: u64,
    pub page: PortalPage,
    pub navigation_expanded: bool,
    pub resource_search: String,
    pub toggle_intents: HashSet<Id>,
    pub checkboxes: HashMap<Id, bool>,
    pub subscriptions: Loadable<Rc<Vec<Arc<Subscription>>>>,
    pub azure_devops_projects: Loadable<Arc<Vec<Arc<AzureDevOpsProject>>>>,
    pub resource_groups: Loadable<Rc<AzureResourceGroupMap>>,
    pub resources: Loadable<Arc<Vec<Arc<Resource>>>>,
    pub tx: UnboundedSender<AppboundWorkFinishedMessage<MyApp>>,
    pub rx: UnboundedReceiver<AppboundWorkFinishedMessage<MyApp>>,
    pub egui_config: EguiConfig,
    pub egui_config_auto_save: AutoSaveBehaviour<EguiConfig>,
    pub work_dirs_config: WorkDirsConfig,
    pub work_dirs_config_auto_save: AutoSaveBehaviour<WorkDirsConfig>,
    /// Whether the About window is open
    pub about_open: bool,
    /// Toasts manager for transient notifications
    pub toasts: Toasts,
    pub work_tracker: AppWorkTracker,
    /// Transfers final settings to the owner for saving after earlier jobs drain.
    pub shutdown_settings: SyncSender<(EguiConfig, WorkDirsConfig)>,
    /// Formatted application info (version + revision) shown in About window
    pub app_info: String,
}

impl MyApp {
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit GUI startup dependencies"
    )]
    pub async fn new(
        azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
        authentication: eyre::Result<Option<AzureBearerAuthenticationSelection>>,
        account_store: CloudTerrastodonAccountStore,
        log_capture: LogBuffer,
        work_tracker: AppWorkTracker,
        shutdown_settings: SyncSender<(EguiConfig, WorkDirsConfig)>,
        app_info: String,
        executor: &CloudTerrastodonExecutor,
    ) -> eyre::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AppboundWorkFinishedMessage<MyApp>>();
        let mut account_picker = AzureAccountPicker::new(&account_store);
        let authentication = match authentication {
            Ok(selection) => selection,
            Err(error) => {
                account_picker.selection_error = Some(DiagnosticReport::new(error));
                None
            }
        };
        let account_authentication = Arc::new(CredentialAuthenticationProvider::from_store(
            account_store.clone(),
        ));
        let executor = executor.with_authentication(account_authentication.clone())?;
        Ok(Self {
            executor,
            work_cache: GuiWorkCache::CallerOwned,
            azure_devops_organization,
            authentication,
            account_picker,
            account_store,
            account_authentication,
            account_manager: AzureAccountManager::default(),
            account_operation: None,
            tenant_manager: AzureTenantManager::default(),
            log_capture,
            log_view: LogView::default(),
            account_generation: 0,
            page: PortalPage::Home,
            navigation_expanded: false,
            resource_search: String::new(),
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
            toasts: Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(Direction::BottomUp),
            work_tracker,
            shutdown_settings,
            app_info,
        })
    }
    pub(crate) fn clear_account_inventory(&mut self) {
        self.tenant_manager.account_selection_changed();
        self.account_generation += 1;
        self.resources = Loadable::NotLoaded;
        self.subscriptions = Loadable::NotLoaded;
        self.resource_groups = Loadable::NotLoaded;
        self.azure_devops_projects = Loadable::NotLoaded;
        self.checkboxes.clear();
        self.toggle_intents.clear();
    }

    pub fn navigate(&mut self, page: PortalPage) {
        self.page = page;
        self.resource_search.clear();
    }
    pub fn checkbox_for(&mut self, key: impl Hash + Debug) -> &mut bool {
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
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.executor.cancellation().is_cancelled() {
            ui.ctx()
                .send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
        self.poll_account_operation();
        self.handle_messages();
        self.draw_app(ui);
        self.show_account_management(ui.ctx());
        // Channel delivery does not wake eframe. Continue receiving results even
        // when the active page has no animated spinner or is showing other data.
        if matches!(self.resources, Loadable::Loading { .. })
            || matches!(self.subscriptions, Loadable::Loading { .. })
            || matches!(self.resource_groups, Loadable::Loading { .. })
            || matches!(self.azure_devops_projects, Loadable::Loading { .. })
        {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
        if let Err(error) = self.enqueue_auto_save() {
            let report = DiagnosticReport::new(error);
            error!(
                error = report.as_error(),
                "Could not enqueue configuration autosave"
            );
        }
        if let Err(error) = self.work_tracker.prune() {
            let report = DiagnosticReport::new(error);
            error!(
                error = report.as_error(),
                "Could not inspect background work"
            );
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
