use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_pane_id::PortalPaneId;
use crate::portal_pane_layout::PortalPaneLayout;
use crate::portal_split::PortalSplit;
use crate::portal_split_handle::show_split_junctions;
use crate::portal_split_side::PortalSplitSide;
use crate::portal_tab_drag::PortalTabDrag;
use crate::portal_tab_target::PortalTabTarget;
use crate::scope_tags_browser::refresh_tagged_inventories;
use eframe::egui::Align;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::DragAndDrop;
use eframe::egui::Frame;
use eframe::egui::Layout;
use eframe::egui::Margin;
use eframe::egui::PointerButton;
use eframe::egui::Rect;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Ui;
use eframe::egui::UiBuilder;

impl MyApp {
    pub(crate) fn show_portal_workspace(&mut self, ui: &mut Ui, split_rails: bool) {
        let bounds = ui.available_rect_before_wrap();
        ui.set_min_size(bounds.size());
        self.show_workspace_node(ui, &self.tabs.layout().clone(), bounds, split_rails);
        let mut handles = Vec::new();
        self.tabs.layout().split_handles(bounds, &mut handles);
        show_split_junctions(ui, &handles, |marker, bounds, center| {
            self.tabs.resize_split(marker, bounds, center);
        });
    }

    fn show_workspace_node(
        &mut self,
        ui: &mut Ui,
        layout: &PortalPaneLayout,
        bounds: Rect,
        split_rails: bool,
    ) {
        if !bounds.is_positive() {
            return;
        }
        match layout {
            PortalPaneLayout::Pane(id) => self.show_workspace_pane(ui, *id, bounds, split_rails),
            PortalPaneLayout::Split {
                marker,
                divider,
                children,
            } => {
                let mut division = *divider;
                let (_, handle, _) = division.layout(bounds);
                let response = ui
                    .interact(
                        handle,
                        ui.id().with(("split-divider", marker)),
                        Sense::drag(),
                    )
                    .on_hover_and_drag_cursor(if division.horizontal() {
                        CursorIcon::ResizeHorizontal
                    } else {
                        CursorIcon::ResizeVertical
                    });
                if response.dragged()
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    self.tabs.resize_split(*marker, bounds, pointer);
                    division.resize(bounds, pointer);
                }
                let (first, handle, second) = division.layout(bounds);
                ui.painter().rect_filled(
                    handle,
                    0.0,
                    if response.hovered() || response.dragged() {
                        Color32::from_rgb(0, 120, 212)
                    } else {
                        Color32::from_gray(224)
                    },
                );
                self.show_workspace_node(ui, &children[0], first, split_rails);
                self.show_workspace_node(ui, &children[1], second, split_rails);
            }
        }
    }

    fn show_workspace_pane(
        &mut self,
        ui: &mut Ui,
        id: PortalPaneId,
        bounds: Rect,
        split_rails: bool,
    ) {
        if self.tabs.pane_active(id).is_none() {
            return;
        }
        let mut pane = ui.new_child(
            UiBuilder::new()
                .id_salt(("portal-pane", id))
                .max_rect(bounds)
                .layout(Layout::top_down(Align::Min)),
        );
        pane.shrink_clip_rect(bounds);
        if split_rails {
            self.portal_tabs_bar(&mut pane, id);
        }
        let Some(tab) = self.tabs.pane_active(id) else {
            return;
        };
        let Some(target) = self.tabs.tab(tab).map(|tab| tab.target().clone()) else {
            return;
        };
        let content_bounds = pane.available_rect_before_wrap();
        if !content_bounds.is_positive() {
            return;
        }
        let pressed = pane.input(|input| {
            input.pointer.any_pressed() && !input.pointer.button_pressed(PointerButton::Middle)
        });
        if pressed
            && self.tabs.active_id() != tab
            && pane.rect_contains_pointer(content_bounds)
            && let Some(index) = self.tabs.index_of(tab)
        {
            self.tabs.select(index);
            pane.ctx().request_repaint();
        }

        let previous = self.tabs.active_id();
        self.tabs.set_active_for_render(tab);
        self.sync_azure_content();
        self.azure
            .resource_inventories
            .retain(|&pane, _| pane == id);
        refresh_tagged_inventories(self);
        let margin = match &target {
            PortalTabTarget::Page(page)
                if *page != PortalPage::Home
                    && !page.is_entra()
                    && !page.is_about_section()
                    && !page.is_audit_section()
                    && !page.is_pim_section() =>
            {
                Margin::symmetric(16, 12)
            }
            PortalTabTarget::ResourcesOfType(_) | PortalTabTarget::LocalPath(_) => {
                Margin::symmetric(16, 12)
            }
            _ => Margin::ZERO,
        };
        let mut content = pane.new_child(
            UiBuilder::new()
                .id_salt(("portal-tab-content", tab))
                .max_rect(content_bounds)
                .layout(Layout::top_down(Align::Min)),
        );
        content.shrink_clip_rect(content_bounds);
        Frame::NONE.inner_margin(margin).show(&mut content, |ui| {
            self.show_portal_tab(ui, target.clone());
        });
        // Rendering an inactive split must not switch the toolbar's content state.
        if self.tabs.active_id() == tab && self.tabs.active() == &target {
            self.tabs.set_active_for_render(previous);
        }
        self.sync_azure_content();
        self.show_workspace_drop_target(&pane, id, bounds, content_bounds);
    }

    fn show_workspace_drop_target(
        &mut self,
        ui: &Ui,
        destination: PortalPaneId,
        bounds: Rect,
        content: Rect,
    ) {
        let Some(drag) = DragAndDrop::payload::<PortalTabDrag>(ui.ctx()) else {
            return;
        };
        let Some(pointer) = ui.input(|input| input.pointer.interact_pos()) else {
            return;
        };
        if !content.contains(pointer) || !ui.rect_contains_pointer(content) {
            return;
        }
        let Some(source) = drag.source_pane(&self.tabs) else {
            return;
        };
        let tabs = drag.tabs(&self.tabs);
        if tabs.is_empty() {
            return;
        }
        // Drop in the center to join an existing pane, or at an edge to split it.
        let join = self.tabs.is_split()
            && content.shrink2(content.size() * 0.22).contains(pointer)
            && tabs
                .iter()
                .any(|&id| self.tabs.tab(id).is_some_and(|tab| tab.pane != destination));
        let side = PortalSplitSide::nearest(content, pointer);
        let preview = if join {
            bounds
        } else {
            let (first, _, second) = PortalSplit::new(side).layout(bounds);
            match side {
                PortalSplitSide::Left | PortalSplitSide::Top => first,
                PortalSplitSide::Right | PortalSplitSide::Bottom => second,
            }
        };
        // Use the workspace painter so the split preview also covers the tab rail.
        ui.painter().rect_filled(
            preview,
            0.0,
            Color32::from_rgba_unmultiplied(0, 120, 212, 38),
        );
        ui.painter().rect_stroke(
            preview,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(0, 120, 212)),
            StrokeKind::Inside,
        );
        if ui.input(|input| input.pointer.button_released(PointerButton::Primary)) {
            DragAndDrop::take_payload::<PortalTabDrag>(ui.ctx());
            if join {
                self.tabs.move_tabs_to_pane(&tabs, destination);
            } else {
                self.tabs.split_tabs(&tabs, source, destination, side);
            }
            self.sync_azure_content();
            ui.ctx().request_repaint();
        }
    }
}
