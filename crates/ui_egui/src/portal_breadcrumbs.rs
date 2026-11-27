use crate::app::MyApp;
use crate::icons::page_icon;
use crate::icons::portal_target_icon;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Button;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::Margin;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::style::ScrollStyle;
use std::borrow::Cow;

impl MyApp {
    /// Show the visits up to the current position, leaving forward visits in the tab's history.
    pub(crate) fn show_portal_visit_breadcrumbs(&mut self, ui: &mut Ui) -> bool {
        let (visits, current) = self.tabs.history();
        let destination = Frame::new()
            .inner_margin(Margin {
                left: 12,
                top: 4,
                ..Margin::ZERO
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        Image::new(page_icon(PortalPage::History))
                            .fit_to_exact_size(Vec2::splat(14.0)),
                    )
                    .on_hover_text("Tab history");
                    scroll_breadcrumbs(ui, "portal-visit-breadcrumbs", |ui| {
                        let mut destination = None;
                        for (position, visit) in visits[..=current].iter().enumerate() {
                            if position > 0 {
                                ui.label("›");
                            }
                            let title = self.portal_tab_title(visit);
                            if let Some(open) = self.portal_breadcrumb(
                                ui,
                                visit,
                                title.as_ref(),
                                position == current,
                            ) {
                                destination = Some((position, open));
                            }
                        }
                        destination
                    })
                })
                .inner
            })
            .inner;
        match destination {
            Some((position, PortalTabOpen::Current(_))) if self.tabs.go_to(position) => {
                self.after_history_navigation();
                ui.ctx().request_repaint();
                true
            }
            Some((_, PortalTabOpen::Background(target))) => {
                self.open_background_tab(target);
                false
            }
            _ => false,
        }
    }

    /// Returns whether a breadcrumb moved to a different destination.
    pub(crate) fn show_portal_breadcrumbs(
        &mut self,
        ui: &mut Ui,
        target: &PortalTabTarget,
    ) -> bool {
        let mut hierarchy = self.portal_hierarchy(target).into_iter();
        let Some(home) = hierarchy.next().filter(|_| hierarchy.len() > 0) else {
            return false;
        };
        let parent = Frame::new()
            .inner_margin(Margin {
                left: 12,
                top: 2,
                ..Margin::ZERO
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Keep Home reachable when the rest of the hierarchy overflows.
                    let open_home = self.portal_breadcrumb(
                        ui,
                        &home,
                        self.portal_tab_title(&home).as_ref(),
                        false,
                    );
                    ui.label("›");
                    let parent = scroll_breadcrumbs(ui, "portal-hierarchy", |ui| {
                        let mut parent = None;
                        for (position, scope) in hierarchy.enumerate() {
                            if position > 0 {
                                ui.label("›");
                            }
                            let title = match &scope {
                                PortalTabTarget::ScopeView { view, .. } => {
                                    Cow::Borrowed(view.label())
                                }
                                PortalTabTarget::EntraGroupSection { section, .. } => {
                                    Cow::Borrowed(section.title())
                                }
                                PortalTabTarget::EntraUserSection { section, .. } => {
                                    Cow::Borrowed(section.title())
                                }
                                PortalTabTarget::AzureDevOpsOrganization {
                                    view: PortalAzureDevOpsOrganizationView::Projects,
                                    ..
                                } => Cow::Borrowed("Projects"),
                                PortalTabTarget::AzureDevOpsOrganization {
                                    view: PortalAzureDevOpsOrganizationView::Users,
                                    ..
                                } => Cow::Borrowed("Users"),
                                PortalTabTarget::AzureDevOpsProject { view, .. }
                                    if *view != PortalAzureDevOpsProjectView::Overview =>
                                {
                                    Cow::Borrowed(view.label())
                                }
                                _ => self.portal_tab_title(&scope),
                            };
                            if let Some(open) =
                                self.portal_breadcrumb(ui, &scope, title.as_ref(), &scope == target)
                            {
                                parent = Some(open);
                            }
                        }
                        parent
                    });
                    open_home.or(parent)
                })
                .inner
            })
            .inner;
        parent.is_some_and(|open| open.apply(self))
    }

    fn portal_breadcrumb(
        &self,
        ui: &mut Ui,
        target: &PortalTabTarget,
        title: &str,
        current: bool,
    ) -> Option<PortalTabOpen> {
        const ICON_SIZE: f32 = 14.0;
        const ICON_GAP: f32 = 4.0;
        let icon =
            Image::new(portal_target_icon(self, target)).fit_to_exact_size(Vec2::splat(ICON_SIZE));
        if current {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = ICON_GAP;
                ui.add(icon);
                ui.strong(title);
            });
            None
        } else {
            let response = ui
                .add(
                    Button::new((icon, title))
                        .small()
                        .frame(false)
                        .gap(ICON_GAP)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            PortalTabOpen::from_response(&response, || target.clone())
        }
    }
}

fn scroll_breadcrumbs<R>(ui: &mut Ui, id_salt: &str, breadcrumbs: impl FnOnce(&mut Ui) -> R) -> R {
    // The enclosing horizontal row gives this scrollbar its own local style.
    ui.spacing_mut().scroll = ScrollStyle {
        bar_width: 4.0,
        bar_inner_margin: 3.0,
        ..ScrollStyle::solid()
    };
    ScrollArea::horizontal()
        .id_salt(id_salt)
        .stick_to_right(true)
        .show(ui, |ui| ui.horizontal(breadcrumbs).inner)
        .inner
}
