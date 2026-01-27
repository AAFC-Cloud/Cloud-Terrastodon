use crate::app::MyApp;
use eframe::egui;
use eframe::egui::Ui;
use egui_tiles::Behavior;
use egui_tiles::SimplificationOptions;
use egui_tiles::Tabs;
use egui_tiles::TileId;
use egui_tiles::Tiles;
use egui_tiles::Tree;
use egui_tiles::UiResponse;
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pane {
    Home,
    Subscriptions,
    ResourceGroups,
    Resources,
    Pim,
    StartingPoints,
    SelectedItems,
    WorkDirs,
    FileDrag,
}

pub struct TreeBehavior {
    app_ptr: Option<*mut MyApp>,
    simplification_options: SimplificationOptions,
    tab_bar_height: f32,
    gap_width: f32,

    /// How many home tabs the top-bar "+" has requested to create this frame.
    pending_new_home_tabs: usize,

    /// If set, we should open a resources tab after the tree UI pass completes.
    pending_open_resources_tab: bool,

    /// If the last tab was closed we set this and the outer code will ensure a Home tab exists.
    ensure_home_tab: bool,
}

impl Default for TreeBehavior {
    fn default() -> Self {
        Self {
            app_ptr: None,
            simplification_options: SimplificationOptions {
                prune_empty_tabs: true,
                prune_empty_containers: true,
                prune_single_child_tabs: false,
                prune_single_child_containers: true,
                all_panes_must_have_tabs: true,
                join_nested_linear_containers: true,
            },
            tab_bar_height: 26.0,
            gap_width: 2.0,
            pending_new_home_tabs: 0,
            pending_open_resources_tab: false,
            ensure_home_tab: false,
        }
    }
}

impl TreeBehavior {
    /// Set the raw pointer to the app without creating extra borrows.
    pub fn set_app(&mut self, app_ptr: *mut MyApp) {
        self.app_ptr = Some(app_ptr);
    }
    pub fn clear_app(&mut self) {
        self.app_ptr = None;
    }

    fn app(&mut self) -> &mut MyApp {
        unsafe {
            &mut *self
                .app_ptr
                .expect("TreeBehavior: app not set before UI render")
        }
    }

    // Accessors used by outer UI to take/clear the pending flags in a single
    // non-borrowing operation.
    pub fn take_pending_new_home_tabs(&mut self) -> usize {
        let c = self.pending_new_home_tabs;
        self.pending_new_home_tabs = 0;
        c
    }

    pub fn take_pending_open_resources_tab(&mut self) -> bool {
        let b = self.pending_open_resources_tab;
        self.pending_open_resources_tab = false;
        b
    }

    pub fn take_ensure_home_tab(&mut self) -> bool {
        let b = self.ensure_home_tab;
        self.ensure_home_tab = false;
        b
    }
}

impl Behavior<Pane> for TreeBehavior {
    fn pane_ui(&mut self, ui: &mut Ui, _tile_id: TileId, view: &mut Pane) -> UiResponse {
        match view {
            Pane::Home => {
                ui.vertical(|ui| {
                    ui.heading("Get started:");
                    ui.add_space(6.0);
                    if ui.button("Subscriptions").clicked() {
                        // request to open a subscriptions tab
                        self.pending_new_home_tabs = self.pending_new_home_tabs.saturating_add(1);
                    }
                    if ui.button("Resource Groups").clicked() {
                        // request to open a resource groups tab
                        self.pending_new_home_tabs = self.pending_new_home_tabs.saturating_add(1);
                    }
                    if ui.button("Resources").clicked() {
                        // request opening a resources tab after tree UI pass
                        self.pending_open_resources_tab = true;
                    }
                    if ui.button("PIM").clicked() {
                        // request to open a PIM tab
                        self.pending_new_home_tabs = self.pending_new_home_tabs.saturating_add(1);
                    }
                });
                UiResponse::None
            }
            Pane::Resources => {
                crate::windows::resources_window::resources_ui(self.app(), ui);
                UiResponse::None
            }
            Pane::Subscriptions | Pane::ResourceGroups | Pane::Pim => {
                ui.label("(Not yet implemented)");
                UiResponse::None
            }
            Pane::StartingPoints => {
                crate::windows::starting_points_window::starting_points_ui(self.app(), ui);
                UiResponse::None
            }
            Pane::SelectedItems => {
                crate::windows::selected_items_window::selected_items_ui(self.app(), ui);
                UiResponse::None
            }
            Pane::WorkDirs => {
                let mut dnd_resp = None;
                crate::windows::work_dirs_window::work_dirs_ui(self.app(), ui, &mut dnd_resp);
                if let Some(crate::windows::work_dirs_window::DNDResponse {
                    source_index,
                    destination_index,
                }) = dnd_resp
                {
                    let item = self
                        .app()
                        .work_dirs_config
                        .work_dirs
                        .remove_index(source_index)
                        .unwrap();
                    self.app()
                        .work_dirs_config
                        .work_dirs
                        .shift_insert(destination_index, item);
                }
                UiResponse::None
            }
            Pane::FileDrag => {
                crate::file_drag_and_drop::file_drag_ui(self.app(), ui);
                UiResponse::None
            }
        }
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::Home => "Home".into(),
            Pane::Subscriptions => "Subscriptions".into(),
            Pane::ResourceGroups => "Resource Groups".into(),
            Pane::Resources => "Resources".into(),
            Pane::Pim => "PIM".into(),
            Pane::StartingPoints => "Starting Points".into(),
            Pane::SelectedItems => "Selected Items".into(),
            Pane::WorkDirs => "Work Dirs".into(),
            Pane::FileDrag => "File Drop".into(),
        }
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &Tiles<Pane>,
        ui: &mut Ui,
        _tile_id: TileId,
        _tabs: &Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            // Request that the outer code adds a Home tab after the UI pass.
            self.pending_new_home_tabs = self.pending_new_home_tabs.saturating_add(1);
        }
    }



    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        self.tab_bar_height
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        self.gap_width
    }

    fn simplification_options(&self) -> SimplificationOptions {
        self.simplification_options
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }

    fn on_tab_close(&mut self, tiles: &mut Tiles<Pane>, tile_id: TileId) -> bool {
        // If closing this tile would leave no tiles, request the outer code to ensure a Home tab exists.
        if tiles.len() <= 1 {
            self.ensure_home_tab = true;
        }
        true
    }
}

pub fn create_tree() -> Tree<Pane> {
    // Start with a single Home tab
    let mut tiles = Tiles::default();

    let home = tiles.insert_pane(Pane::Home);

    let children = vec![home];
    let root = tiles.insert_tab_tile(children);

    Tree::new("main_tiles", root, tiles)
}
