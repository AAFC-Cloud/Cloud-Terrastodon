use crate::entra_directory_ui::copyable_id;
use crate::icons::HELIX_TICKETS_ICON;
use crate::portal_helix_ticket_row::PortalHelixTicketRow;
use crate::portal_tab_open::PortalTabOpen;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicket;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Grid;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::Margin;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use std::collections::HashMap;

/// Show the query criteria attached to the Shared Ticket Console request.
pub(crate) fn show_ticket_console_criteria(ui: &mut Ui) {
    ui.heading("Shared Ticket Console");
    ui.horizontal_wrapped(|ui| {
        for criterion in ["All open", "My support groups", "Assigned"] {
            Frame::new()
                .fill(Color32::from_rgb(225, 239, 248))
                .corner_radius(10)
                .inner_margin(Margin::symmetric(10, 4))
                .show(ui, |ui| {
                    ui.label(criterion);
                });
        }
    });
}

pub(crate) fn show_ticket_console_table(
    ui: &mut Ui,
    server: &BmcHelixServerName,
    tickets: &[BmcHelixTicket],
) -> Option<PortalTabOpen> {
    let columns = [
        ReflectedTableColumn::new("Ticket", "display_id")
            .with_width(220.0)
            .with_secondary_text("id")
            .with_identifier("id")
            .with_link("target")
            .with_icon(|_, _| Some(HELIX_TICKETS_ICON)),
        ReflectedTableColumn::new("Title", "title"),
        ReflectedTableColumn::new("Status", "status")
            .with_width(160.0)
            .groupable(),
        ReflectedTableColumn::new("Assigned to", "assignee")
            .with_width(170.0)
            .groupable(),
        ReflectedTableColumn::new("Created (UTC)", "created_at").with_width(190.0),
        ReflectedTableColumn::new("Last activity (UTC)", "last_activity_at").with_width(190.0),
    ];
    let rows: Vec<_> = tickets
        .iter()
        .map(|ticket| PortalHelixTicketRow::new(server, ticket))
        .collect();
    ReflectedTable::new("helix-tickets", &columns)
        .without_selection()
        .with_search_hint("Search loaded tickets")
        .with_result_count()
        .show_explorable(ui, &rows, |ticket| Id::new(&ticket.id), &mut HashMap::new())
}

pub(crate) fn show_ticket_details(ui: &mut Ui, ticket: &BmcHelixTicket) {
    ScrollArea::vertical().show(ui, |ui| {
        Grid::new("helix-ticket-details").show(ui, |ui| {
            copyable_id(ui, "Record ID", &ticket.id);
            for (name, value) in [
                ("Title", ticket.title.as_deref()),
                ("Status", ticket.status.as_deref()),
                ("Assigned to", ticket.assignee.as_deref()),
            ] {
                ui.label(name);
                ui.label(value.unwrap_or("—"));
                ui.end_row();
            }
            for (name, timestamp) in [
                ("Created (UTC)", ticket.created_at),
                ("Last activity (UTC)", ticket.last_activity_at),
            ] {
                ui.label(name);
                ui.label(timestamp.map_or_else(|| "—".to_owned(), |value| value.to_string()));
                ui.end_row();
            }
        });
        ui.separator();
        ui.strong("Description");
        if let Some(description) = &ticket.description {
            ui.add(Label::new(description).wrap().selectable(true));
        } else {
            ui.weak("No detailed description available.");
        }
        if !ticket.fields.is_empty() {
            ui.collapsing(format!("Fields ({})", ticket.fields.len()), |ui| {
                Grid::new("helix-ticket-fields").show(ui, |ui| {
                    for (id, value) in &ticket.fields {
                        ui.monospace(id.to_string());
                        ui.label(value.as_ref());
                        ui.end_row();
                    }
                });
            });
        }
    });
}
