use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use eframe::egui::Align2;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Panel;
use eframe::egui::Rect;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

const NAVIGATION_PAGES: [(PortalPage, &str); 18] = [
    (PortalPage::Home, "home"),
    (PortalPage::Resources, "resources"),
    (PortalPage::Subscriptions, "subscriptions"),
    (PortalPage::ResourceGroups, "resource-groups"),
    (PortalPage::Tags, "tags"),
    (PortalPage::ManagementGroups, "management-groups"),
    (PortalPage::StorageAccounts, "storage-accounts"),
    (PortalPage::ContainerRegistries, "container-registries"),
    (PortalPage::KeyVaults, "key-vaults"),
    (PortalPage::Entra, "entra"),
    (PortalPage::Pim, "pim"),
    (PortalPage::AzureDevOpsOrganizations, "devops-organizations"),
    (PortalPage::HelixServers, "helix"),
    (PortalPage::Auditing, "auditing"),
    (PortalPage::StartingPoints, "starting-points"),
    (PortalPage::SelectedItems, "selected-items"),
    (PortalPage::WorkDirs, "work-directories"),
    (PortalPage::FileDrop, "import-files"),
];

impl MyApp {
    /// A compact rail can expand to show destination labels without taking content space by default.
    pub fn portal_navigation(&mut self, ui: &mut Ui) {
        Panel::left("portal_navigation")
            .exact_size(if self.navigation_expanded {
                200.0
            } else {
                44.0
            })
            .resizable(false)
            .frame(Frame::new().fill(Color32::from_rgb(250, 250, 250)))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.spacing_mut().button_padding = Vec2::new(12.0, 8.0);
                let expanded = self.navigation_expanded;
                sidebar_expansion_toggle(ui, &mut self.navigation_expanded, "main navigation");
                ScrollArea::vertical().show(ui, |ui| {
                    let id = Id::new("portal_navigation_order");
                    let saved: Vec<String> = ui
                        .ctx()
                        .data_mut(|data| data.get_persisted(id))
                        .unwrap_or_default();
                    let mut items = NAVIGATION_PAGES.to_vec();
                    items.sort_by_key(|(_, key)| {
                        saved
                            .iter()
                            .position(|item| item.as_str() == *key)
                            .unwrap_or(usize::MAX)
                    });
                    let mut moved = None;
                    for (page, key) in &items {
                        if let Some((source, before)) = ui
                            .push_id(key, |ui| self.portal_navigation_item(ui, *page, expanded))
                            .inner
                        {
                            moved = Some((source, *page, before));
                        }
                    }
                    if let Some((source, target, before)) = moved
                        && let Some(source) = items.iter().position(|(page, _)| *page == source)
                    {
                        let item = items.remove(source);
                        if let Some(target) = items.iter().position(|(page, _)| *page == target) {
                            items.insert(target + usize::from(!before), item);
                            let saved: Vec<_> =
                                items.iter().map(|(_, key)| (*key).to_owned()).collect();
                            ui.ctx().data_mut(|data| data.insert_persisted(id, saved));
                        }
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    let page = PortalPage::About;
                    let icon = Image::new(page_icon(page)).fit_to_exact_size(Vec2::splat(20.0));
                    let response = ui
                        .add_sized(
                            [ui.available_width(), 40.0],
                            if expanded {
                                Button::image_and_text(icon, page.title())
                            } else {
                                Button::image(icon)
                            }
                            .frame(false)
                            .selected(self.current_page().is_about_section()),
                        )
                        .on_hover_text("About Cloud Terrastodon");
                    if let Some(action) =
                        PortalTabOpen::from_response(&response, || PortalTabTarget::Page(page))
                    {
                        action.apply(self);
                    }
                });
            });
    }

    fn portal_navigation_item(
        &mut self,
        ui: &mut Ui,
        page: PortalPage,
        expanded: bool,
    ) -> Option<(PortalPage, bool)> {
        let (_, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 40.0),
            Sense::click_and_drag(),
        );
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), page.title()));
        let current = self.current_page();
        let active = current == page
            || (page == PortalPage::Entra && current.is_entra())
            || (page == PortalPage::Auditing && current.is_audit_section())
            || (page == PortalPage::Pim && current.is_pim_section());
        if active || response.hovered() || response.has_focus() {
            ui.painter().rect_filled(
                response.rect,
                0.0,
                if active {
                    Color32::from_rgb(229, 241, 251)
                } else {
                    Color32::from_rgb(237, 235, 233)
                },
            );
        }
        let icon_rect = Rect::from_center_size(
            response.rect.left_center() + Vec2::new(22.0, 0.0),
            Vec2::splat(20.0),
        );
        Image::new(page_icon(page)).paint_at(ui, icon_rect);
        if expanded {
            ui.painter().text(
                response.rect.left_center() + Vec2::new(44.0, 0.0),
                Align2::LEFT_CENTER,
                if page == PortalPage::Pim {
                    "PIM"
                } else {
                    page.title()
                },
                FontId::proportional(13.0),
                Color32::from_rgb(50, 49, 48),
            );
        }
        if active {
            ui.painter().rect_filled(
                Rect::from_min_size(response.rect.min, Vec2::new(3.0, response.rect.height())),
                0.0,
                Color32::from_rgb(0, 120, 212),
            );
        }
        response.dnd_set_drag_payload(page);
        let mut moved = None;
        if let Some(source) = response.dnd_hover_payload::<PortalPage>()
            && *source != page
            && let Some(pointer) = ui.input(|input| input.pointer.interact_pos())
        {
            let before = pointer.y < response.rect.center().y;
            ui.painter().hline(
                response.rect.x_range(),
                if before {
                    response.rect.top()
                } else {
                    response.rect.bottom()
                },
                Stroke::new(2.0, Color32::from_rgb(0, 120, 212)),
            );
            if response.dnd_release_payload::<PortalPage>().is_some() {
                moved = Some((*source, before));
            }
        }
        let response = response.on_hover_text(page.title());
        if let Some(action) =
            PortalTabOpen::from_response(&response, || PortalTabTarget::Page(page))
        {
            action.apply(self);
        }
        moved
    }
}
