use crate::app::MyApp;
use crate::icons::portal_target_icon;
use crate::icons::resource_icon;
use crate::portal_history_entry::PortalHistoryEntry;
use crate::portal_page::PortalPage;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_scope_view::PortalScopeView;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Atom;
use eframe::egui::Button;
use eframe::egui::Image;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use itertools::Itertools;
use jiff::Timestamp;
use jiff::tz::TimeZone;

impl MyApp {
    pub(crate) fn record_portal_visit(&mut self) {
        self.sync_azure_content();
        self.record_portal_visit_to(self.tabs.active().clone());
    }

    pub(crate) fn record_portal_visit_to(&mut self, target: PortalTabTarget) {
        if target == PortalTabTarget::Page(PortalPage::History) {
            return;
        }
        let resource_kind = match &target {
            PortalTabTarget::Resource(id)
            | PortalTabTarget::ScopeView {
                scope: PortalResourceScope::Resource(id),
                view: PortalScopeView::ActivityLog | PortalScopeView::AccessControl,
            }
            | PortalTabTarget::ResourceOutput {
                resource: PortalResourceOutputTarget::Scope(PortalResourceScope::Resource(id)),
                ..
            } => self
                .inventory_resource(id)
                .map(|resource| resource.kind.clone()),
            _ => None,
        };
        let title = self.portal_tab_title(&target).into_owned();
        let authentication = self
            .azure
            .authentication
            .filter(|_| PortalHistoryEntry::uses_azure_authentication(&target));
        self.portal_history.record(PortalHistoryEntry {
            target,
            title,
            visited_at: Timestamp::now(),
            resource_kind,
            authentication,
        });
    }

    pub(crate) fn show_portal_history(&mut self, ui: &mut Ui) {
        let entries = self.portal_history.visible(self.azure.authentication);
        if entries.is_empty() {
            ui.label("No pages visited yet.");
            return;
        }
        let target = ScrollArea::vertical()
            .id_salt("portal_history")
            .show(ui, |ui| {
                let mut target = None;
                let zone = TimeZone::system();
                for (day, entries) in &entries
                    .into_iter()
                    .map(|entry| (entry.visited_at.to_zoned_datetime(zone.clone()), entry))
                    .chunk_by(|(time, _)| time.date())
                {
                    ui.label(RichText::new(day.strftime("%A, %B %-d, %Y").to_string()).strong());
                    for (time, entry) in entries {
                        let icon = entry
                            .resource_kind
                            .as_ref()
                            .map_or_else(|| portal_target_icon(self, &entry.target), resource_icon);
                        let title = if self.egui_config.demo_mode && entry.title.contains('@') {
                            "Redacted title"
                        } else {
                            entry.title.as_str()
                        };
                        let button = Button::new((
                            Image::new(icon).fit_to_exact_size(Vec2::splat(16.0)),
                            title,
                            Atom::grow(),
                        ))
                        .right_text(RichText::new(time.strftime("%H:%M:%S").to_string()).weak())
                        .min_size(Vec2::new(ui.available_width(), 30.0))
                        .frame_when_inactive(false)
                        .truncate();
                        let response = ui.add(button).on_hover_text(title);
                        if let Some(action) =
                            PortalTabOpen::from_response(&response, || entry.target.clone())
                        {
                            target = Some(action);
                        }
                    }
                    ui.add_space(12.0);
                }
                target
            })
            .inner;
        if let Some(action) = target {
            action.apply(self);
        }
    }
}
