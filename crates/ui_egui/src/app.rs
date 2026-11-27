use crate::autosave_info::AutoSaveBehaviour;
use crate::azure_account_manager::AzureAccountManager;
use crate::azure_account_operation::AzureAccountOperation;
use crate::azure_account_picker::AzureAccountPicker;
use crate::azure_account_profiles::AzureAccountProfiles;
use crate::azure_devops_organization_picker::AzureDevOpsOrganizationPicker;
use crate::azure_tenant_manager::AzureTenantManager;
use crate::capture_gui_failure::capture_gui_failure;
use crate::gui_work_cache::GuiWorkCache;
use crate::local_path_preview::LocalPathPreview;
use crate::portal_address::PortalAddress;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_history::PortalHistory;
use crate::portal_log_toast::PortalLogToast;
use crate::portal_page::PortalPage;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_target::PortalTabTarget;
use crate::portal_tabs::PortalTabs;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_command::app_work::AppboundWorkFinishedMessage;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_config::EguiConfig;
use cloud_terrastodon_config::WorkDirsConfig;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eframe::App;
use eframe::Storage;
use eframe::egui::Id;
use eframe::egui::Ui;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
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
    pub(crate) azure_devops_organization_picker: AzureDevOpsOrganizationPicker,
    pub(crate) azure_devops_license_updates: HashSet<AzureDevOpsOrganizationUrl>,
    pub(crate) azure: PortalAzureContent,
    // The displayed content is attached while invoking its existing page widgets.
    azure_tab: PortalTabId,
    azure_tabs: HashMap<PortalTabId, PortalAzureContent>,
    next_account_generation: u64,
    pub account_picker: AzureAccountPicker,
    pub(crate) account_profiles: AzureAccountProfiles,
    pub(crate) account_store: CloudTerrastodonAccountStore,
    pub(crate) account_authentication: Arc<CredentialAuthenticationProvider>,
    pub(crate) account_manager: AzureAccountManager,
    pub(crate) account_operation: Option<AzureAccountOperation>,
    pub(crate) account_sign_in_failure: Option<DiagnosticReport>,
    pub(crate) tenant_manager: AzureTenantManager,
    /// Shared bounded history owned by the application, not a second GUI collector.
    pub log_capture: LogBuffer,
    pub log_view: LogView,
    pub(crate) log_toast_cursor: usize,
    pub(crate) log_toasts: Vec<PortalLogToast>,
    pub(crate) pending_log_jumps: HashMap<PortalTabId, usize>,
    pub(crate) tabs: PortalTabs,
    pub(crate) portal_address: Option<PortalAddress>,
    pub(crate) portal_history: PortalHistory,
    pub navigation_expanded: bool,
    pub(crate) resource_navigation_expanded: bool,
    pub(crate) local_path_previews: HashMap<PathBuf, Loadable<LocalPathPreview>>,
    pub tx: UnboundedSender<AppboundWorkFinishedMessage<MyApp>>,
    pub rx: UnboundedReceiver<AppboundWorkFinishedMessage<MyApp>>,
    pub egui_config: EguiConfig,
    pub egui_config_auto_save: AutoSaveBehaviour<EguiConfig>,
    pub work_dirs_config: WorkDirsConfig,
    pub work_dirs_config_auto_save: AutoSaveBehaviour<WorkDirsConfig>,
    pub work_tracker: AppWorkTracker,
    /// Transfers final settings to the owner for saving after earlier jobs drain.
    pub shutdown_settings: SyncSender<(EguiConfig, WorkDirsConfig)>,
    /// Formatted application version and revision for the About page.
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
                account_picker.set_selection_error(
                    PortalTabId(0),
                    Some(capture_gui_failure(
                        error,
                        "Could not select the initial Azure account",
                    )),
                );
                None
            }
        };
        let account_authentication = Arc::new(CredentialAuthenticationProvider::from_store(
            account_store.clone(),
        ));
        let executor = executor.with_authentication(account_authentication.clone())?;
        let egui_config = EguiConfig::load().await?;
        let mut tabs = PortalTabs::default();
        let mut open_dirs: Vec<_> = egui_config.open_dirs.keys().cloned().collect();
        open_dirs.sort();
        for path in open_dirs {
            tabs.open_kept(PortalTabTarget::LocalPath(path));
        }
        tabs.select(0);
        let mut log_toast_cursor = 0;
        log_capture.records_since(&mut log_toast_cursor);
        Ok(Self {
            executor,
            work_cache: GuiWorkCache::CallerOwned,
            azure_devops_organization_picker: AzureDevOpsOrganizationPicker::new(
                azure_devops_organization.as_ref(),
            ),
            azure_devops_license_updates: HashSet::new(),
            azure: PortalAzureContent {
                azure_devops_organization,
                ..PortalAzureContent::new(authentication, 0)
            },
            azure_tab: tabs.active_id(),
            azure_tabs: HashMap::new(),
            next_account_generation: 1,
            account_picker,
            account_profiles: Default::default(),
            account_store,
            account_authentication,
            account_manager: AzureAccountManager::default(),
            account_operation: None,
            account_sign_in_failure: None,
            tenant_manager: AzureTenantManager::default(),
            log_capture,
            log_view: LogView::default(),
            log_toast_cursor,
            log_toasts: Vec::new(),
            pending_log_jumps: HashMap::new(),
            tabs,
            portal_address: None,
            portal_history: PortalHistory::default(),
            navigation_expanded: false,
            resource_navigation_expanded: true,
            local_path_previews: Default::default(),
            tx,
            rx,
            egui_config,
            egui_config_auto_save: Default::default(),
            work_dirs_config: WorkDirsConfig::load().await?,
            work_dirs_config_auto_save: Default::default(),
            work_tracker,
            shutdown_settings,
            app_info,
        })
    }
    pub(crate) fn sync_azure_content(&mut self) {
        let tab = self.tabs.active_id();
        if self.azure_tab == tab {
            return;
        }
        let content = self.azure_tabs.remove(&tab).unwrap_or_else(|| {
            let content = self.azure.fork(self.next_account_generation);
            self.next_account_generation += 1;
            content
        });
        let previous = std::mem::replace(&mut self.azure, content);
        let previous_tab = std::mem::replace(&mut self.azure_tab, tab);
        self.azure_tabs.insert(previous_tab, previous);
    }

    pub(crate) fn seed_azure_content(&mut self, tab: PortalTabId, source: PortalTabId) {
        if self.azure_tab != tab && !self.azure_tabs.contains_key(&tab) {
            let content = self
                .azure_content_for_tab(source)
                .unwrap_or(&self.azure)
                .fork(self.next_account_generation);
            self.next_account_generation += 1;
            self.azure_tabs.insert(tab, content);
        }
    }

    pub(crate) fn azure_content_for_tab(&self, tab: PortalTabId) -> Option<&PortalAzureContent> {
        if self.azure_tab == tab {
            Some(&self.azure)
        } else {
            self.azure_tabs.get(&tab)
        }
    }

    pub(crate) fn azure_content_for_tab_mut(
        &mut self,
        tab: PortalTabId,
    ) -> Option<&mut PortalAzureContent> {
        self.tabs.tab(tab)?;
        if self.azure_tab == tab {
            Some(&mut self.azure)
        } else {
            self.azure_tabs.get_mut(&tab)
        }
    }

    pub(crate) fn azure_content_mut(&mut self, generation: u64) -> Option<&mut PortalAzureContent> {
        if self.azure.account_generation == generation {
            return Some(&mut self.azure);
        }
        self.azure_tabs
            .values_mut()
            .find(|content| content.account_generation == generation)
    }

    pub(crate) fn reset_azure_content_for_tab(&mut self, tab: PortalTabId) {
        let generation = self.next_account_generation;
        if let Some(content) = self.azure_content_for_tab_mut(tab) {
            content.reset_inventory(generation);
            self.next_account_generation += 1;
        }
    }

    pub(crate) fn invalidate_azure_accounts(
        &mut self,
        changed: impl Fn(&PortalAzureContent) -> bool,
    ) {
        self.tenant_manager.account_selection_changed();
        for content in std::iter::once(&mut self.azure).chain(self.azure_tabs.values_mut()) {
            if changed(content) {
                content.reset_inventory(self.next_account_generation);
                self.next_account_generation += 1;
            }
        }
    }

    pub fn navigate(&mut self, page: PortalPage) {
        self.navigate_tab(PortalTabTarget::Page(page));
    }

    pub fn navigate_tab(&mut self, target: PortalTabTarget) {
        self.sync_azure_content();
        let previous = self.tabs.active_id();
        let from_devops_list = matches!(
            self.tabs.active(),
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations)
        );
        if !self.tabs.open(target) {
            return;
        }
        self.sync_azure_content();
        if previous == self.tabs.active_id() {
            self.azure.resource_search.clear();
        }
        if from_devops_list
            && let PortalTabTarget::AzureDevOpsOrganization { organization, .. } =
                self.tabs.active()
        {
            let organization = organization.as_ref().clone();
            self.seed_azure_devops_organization_account(self.tabs.active_id(), &organization);
        }
        self.record_portal_visit();
        self.retain_open_tab_data();
    }

    pub(crate) fn navigate_current_tab(&mut self, target: PortalTabTarget) {
        if self.tabs.navigate_current(target) {
            self.record_portal_visit();
            self.azure.resource_search.clear();
            self.retain_open_tab_data();
        }
    }

    pub(crate) fn open_background_tab(&mut self, target: PortalTabTarget) -> PortalTabId {
        let source = self.tabs.active_id();
        let tab = self.tabs.open_background(target.clone());
        self.seed_azure_content(tab, source);
        if matches!(
            self.tabs.active(),
            PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations)
        ) && let PortalTabTarget::AzureDevOpsOrganization { organization, .. } = &target
        {
            self.seed_azure_devops_organization_account(tab, organization);
        }
        self.record_portal_visit_to(target);
        tab
    }

    pub fn current_page(&self) -> PortalPage {
        self.tabs.active().page()
    }

    pub(crate) fn retain_open_tab_data(&mut self) {
        self.sync_azure_content();
        self.azure_tabs.retain(|id, _| self.tabs.retains_tab(*id));
        self.account_picker
            .retain_tabs(|id| self.tabs.tab(id).is_some());
        self.pending_log_jumps
            .retain(|id, _| self.tabs.tab(*id).is_some());
        self.local_path_previews
            .retain(|path, _| self.tabs.has_local_path(path));
        for content in std::iter::once(&mut self.azure).chain(self.azure_tabs.values_mut()) {
            content.storage_account_browser.retain_open(&self.tabs);
            content.storage_explorer.retain_open(&self.tabs);
        }
    }

    pub(crate) fn open_azure_contents_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut PortalAzureContent> {
        let tabs = &self.tabs;
        std::iter::once(&mut self.azure).chain(
            self.azure_tabs
                .iter_mut()
                .filter_map(move |(id, content)| tabs.tab(*id).map(|_| content)),
        )
    }

    pub(crate) fn pending_principal_lookups(&self) -> usize {
        std::iter::once(&self.azure)
            .chain(self.azure_tabs.values())
            .map(|content| content.scope_iam.principals.pending_count())
            .sum()
    }
    pub fn checkbox_for(&mut self, key: impl Hash + Debug) -> &mut bool {
        self.azure.checkboxes.entry(Id::new(key)).or_default()
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
    fn save(&mut self, storage: &mut dyn Storage) {
        self.portal_history.save(storage);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.sync_azure_content();
        if self.executor.cancellation().is_cancelled() {
            ui.ctx()
                .send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
        self.poll_account_operation();
        self.tenant_manager.poll();
        self.handle_messages();
        if let Err(error) = self.work_tracker.prune() {
            let report = DiagnosticReport::new(error);
            error!(
                error = report.as_error(),
                "Could not inspect background work"
            );
        }
        self.draw_app(ui);
        self.show_account_operation(ui.ctx());
        // Channel delivery does not wake eframe. Continue receiving results even
        // when the active page has no animated spinner or is showing other data.
        if self.azure.is_loading()
            || self.azure_tabs.values().any(PortalAzureContent::is_loading)
            || self
                .azure_devops_organization_picker
                .is_loading_tracked_organizations()
            || self
                .local_path_previews
                .values()
                .any(|preview| matches!(preview, Loadable::Loading { .. }))
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
