use crate::app::MyApp;
use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_picker::AzureAccountPicker;
use crate::azure_account_picker_action::AzureAccountPickerAction;
use crate::entra_directory_ui::loaded;
use crate::icons::HELIX_ICON;
use crate::icons::HELIX_TICKETS_ICON;
use crate::icons::HELIX_WORK_ORDER_ICON;
use crate::portal_helix_server_form::parse_innovation_suite_url;
use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_helix_ticket_console::show_ticket_console_criteria;
use crate::portal_helix_ticket_console::show_ticket_console_table;
use crate::portal_helix_ticket_console::show_ticket_details;
use crate::portal_helix_work::PortalHelixWork;
use crate::portal_helix_work_orders_ui::show_work_orders;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_value::ReflectedValue;
use crate::resource_detail_header::resource_detail_header;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CentralPanel;
use eframe::egui::ComboBox;
use eframe::egui::Frame;
use eframe::egui::Grid;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;

pub(crate) fn show_helix_server(
    app: &mut MyApp,
    ui: &mut Ui,
    name: &BmcHelixServerName,
    view: &PortalHelixServerView,
) {
    use PortalHelixServerView::Overview;
    use PortalHelixServerView::Ticket;
    use PortalHelixServerView::Tickets;
    use PortalHelixServerView::WorkOrder;
    use PortalHelixServerView::WorkOrders;

    let (title, kind, inventory) = match view {
        WorkOrder(id) => (id.to_string(), "Helix work order", WorkOrders.target(name)),
        Ticket(id) => {
            let target = view.clone().target(name);
            let title = app
                .helix_ticket_display_id(app.tabs.active_id(), &target)
                .unwrap_or_else(|| id.as_str());
            (title.to_owned(), "Helix ticket", Tickets.target(name))
        }
        _ => (
            name.to_string(),
            "Helix server",
            PortalTabTarget::Page(PortalPage::HelixServers),
        ),
    };
    if resource_detail_header(
        ui,
        &title,
        kind,
        inventory.title().as_ref(),
        view.icon(),
        &inventory,
    )
    .is_some_and(|open| open.apply(app))
    {
        return;
    }
    let expanded = app.resource_navigation_expanded;
    let open = Panel::left(ui.make_persistent_id("helix-server-navigation"))
        .exact_size(if expanded { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "Helix server navigation",
            );
            let overview = navigation_tile(
                ui,
                "Overview",
                HELIX_ICON,
                *view == Overview,
                expanded,
                || Overview.target(name),
            );
            let work_orders = navigation_tile(
                ui,
                "Work orders",
                HELIX_WORK_ORDER_ICON,
                *view == WorkOrders,
                expanded,
                || WorkOrders.target(name),
            );
            let tickets = navigation_tile(
                ui,
                "Tickets",
                HELIX_TICKETS_ICON,
                *view == Tickets,
                expanded,
                || Tickets.target(name),
            );
            overview.or(work_orders).or(tickets)
        })
        .inner;
    if let Some(open) = open {
        match open {
            PortalTabOpen::Current(target) if !matches!(view, WorkOrder(_) | Ticket(_)) => {
                app.tabs.replace_current(target);
                app.record_portal_visit();
                app.after_history_navigation();
                ui.ctx().request_repaint();
                return;
            }
            open => {
                if open.apply(app) {
                    return;
                }
            }
        }
    }
    if matches!(app.helix.servers, Loadable::NotLoaded) {
        app.refresh_helix_catalogue();
    }
    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 8)))
        .show(ui, |ui| {
            app.helix.show_catalogue_error(ui);
            let Some(catalogue) =
                loaded(ui, &app.helix.servers, "Could not load the saved servers.")
            else {
                return;
            };
            let Ok(server) = catalogue.get(Some(name)) else {
                ui.label("This Helix server is no longer registered.");
                return;
            };
            if *view == Overview {
                let is_default = catalogue.default.as_ref() == Some(name);
                show_overview(app, ui, &server, is_default);
            } else {
                show_helix_requests(app, ui, &server, view);
            }
        });
}

fn show_overview(app: &mut MyApp, ui: &mut Ui, server: &BmcHelixServer, is_default: bool) {
    Grid::new("helix-server-overview").show(ui, |ui| {
        ui.label("AR System URL");
        ui.hyperlink(server.url.to_string());
        ui.end_row();
        ui.label("Innovation Suite URL");
        ui.horizontal(|ui| {
            ui.hyperlink(server.innovation_suite_origin().to_string());
            if server.innovation_suite_url.is_none() {
                ui.label("(same as AR System)");
            }
        });
        ui.end_row();
    });
    ui.horizontal_wrapped(|ui| {
        if is_default {
            if ui.button("Clear default server").clicked() {
                app.modify_helix_catalogue("Clear default Helix server", |store| {
                    store.clear_default().map(drop)
                });
            }
        } else if ui.button("Set as default server").clicked() {
            let name = server.name.clone();
            app.modify_helix_catalogue("Set default Helix server", move |store| {
                store.set_default(&name).map(drop)
            });
        }
        show_innovation_suite_editor(app, ui, server);
        ui.menu_button("Remove server", |ui| {
            if ui.button("Confirm removal").clicked() {
                let name = server.name.clone();
                app.modify_helix_catalogue("Remove Helix server", move |store| {
                    store.remove(&name).map(drop)
                });
                ui.close();
            }
        });
    });
    if ui.button("Add Helix account").clicked() {
        app.account_manager.open_add_helix(&server.url);
        app.navigate(PortalPage::Accounts);
    }
}

fn show_innovation_suite_editor(app: &mut MyApp, ui: &mut Ui, server: &BmcHelixServer) {
    let id = ui.make_persistent_id(("helix-innovation-suite-origin", &server.name));
    ui.menu_button("Edit Innovation Suite URL", |ui| {
        let (mut draft, mut error) = ui
            .data(|data| data.get_temp::<(String, Option<String>)>(id))
            .unwrap_or_else(|| {
                (
                    server
                        .innovation_suite_url
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string),
                    None,
                )
            });
        let label = ui.label("Innovation Suite HTTPS origin");
        let edit = ui.text_edit_singleline(&mut draft).labelled_by(label.id);
        if edit.changed() {
            error = None;
        }
        if ui.button("Save URL").clicked() {
            match parse_innovation_suite_url(&draft) {
                Ok(url) => {
                    let name = server.name.clone();
                    app.modify_helix_catalogue("Update Helix Innovation Suite URL", move |store| {
                        store.set_innovation_suite_url(&name, url).map(drop)
                    });
                    ui.data_mut(|data| data.remove::<(String, Option<String>)>(id));
                    ui.close();
                    return;
                }
                Err(report) => error = Some(format!("{report:#}")),
            }
        }
        if let Some(error) = &error {
            ui.label(error);
        }
        ui.data_mut(|data| data.insert_temp(id, (draft, error)));
    })
    .response
    .on_hover_text("Leave blank to use the AR System URL.");
}

fn show_helix_requests(
    app: &mut MyApp,
    ui: &mut Ui,
    server: &BmcHelixServer,
    view: &PortalHelixServerView,
) {
    if *view == PortalHelixServerView::Tickets {
        show_ticket_console_criteria(ui);
    }
    let tab = app.tabs.active_id();
    if app.account_operation.is_some() {
        ui.spinner();
        return;
    }
    let selected = app
        .account_picker
        .catalogue
        .as_ref()
        .ok()
        .and_then(|catalogue| app.helix.selected_account(tab, &server.url, catalogue));
    let selected_id = selected.map(|account| account.id);
    let mut next = selected_id;
    let (manage, reload) = ui
        .horizontal(|ui| {
            ui.label("Account");
            ComboBox::from_id_salt(("helix-account", &server.name))
                .selected_text(selected.map_or("Choose account", |account| {
                    AzureAccountPicker::display_alias(
                        account.alias.as_str(),
                        app.egui_config.demo_mode,
                    )
                }))
                .show_ui(ui, |ui| {
                    if let Ok(catalogue) = &app.account_picker.catalogue {
                        for account in catalogue.accounts.iter().filter(|account| {
                            matches!(
                                account.namespace,
                                CloudTerrastodonAccountNamespace::BmcHelix { .. }
                            )
                        }) {
                            let alias = AzureAccountPicker::display_alias(
                                account.alias.as_str(),
                                app.egui_config.demo_mode,
                            );
                            let session = catalogue.bmc_helix_session(account.id, &server.url);
                            ui.selectable_value(
                                &mut next,
                                Some(account.id),
                                format!(
                                    "{alias} · {}",
                                    AzureAccountDetails::session_label(session)
                                ),
                            );
                        }
                    }
                });
            (
                ui.button("Manage account").clicked(),
                ui.button("Reload accounts").clicked(),
            )
        })
        .inner;
    if let Some(account) = next.filter(|next| Some(*next) != selected_id) {
        app.helix.select_account(tab, &server.url, account);
    }
    if reload {
        app.handle_account_picker_action(AzureAccountPickerAction::Reload, ui.ctx());
        return;
    }
    let ready = app.ready_helix_account(tab, &server.url);
    if manage {
        if let Some(account) = next {
            app.account_manager
                .open_helix_account(account, server.url.clone());
        } else {
            app.account_manager.open_add_helix(&server.url);
        }
        app.navigate(PortalPage::Accounts);
        return;
    }
    let Some(account) = ready else {
        if next.is_some() {
            ui.label("Use Manage account to sign in to this server.");
        }
        return;
    };
    if matches!(
        app.helix.work(tab, server, account, view),
        Loadable::NotLoaded
    ) {
        app.load_helix_work(server, account, view, false);
    }
    if *view == PortalHelixServerView::WorkOrders {
        show_work_orders(app, ui, server, account);
        return;
    }
    let Some(work) = loaded(
        ui,
        app.helix.work(tab, server, account, view),
        match view {
            PortalHelixServerView::Tickets | PortalHelixServerView::Ticket(_) => {
                "Could not load tickets."
            }
            _ => "Could not load work orders.",
        },
    ) else {
        return;
    };
    match work {
        PortalHelixWork::List(_) => {}
        PortalHelixWork::Detail(order) => {
            ScrollArea::vertical().show(ui, |ui| ReflectedValue::new(order.as_ref()).show(ui));
        }
        PortalHelixWork::Tickets(tickets) => {
            if let PortalHelixServerView::Ticket(id) = view {
                if let Some(ticket) = tickets.iter().find(|ticket| ticket.id == *id) {
                    show_ticket_details(ui, ticket);
                } else {
                    ui.label(
                        "This ticket is no longer in the loaded Shared Ticket Console results.",
                    );
                }
            } else if let Some(open) = ui
                .push_id((&server.name, account), |ui| {
                    show_ticket_console_table(ui, &server.name, tickets)
                })
                .inner
            {
                open.apply(app);
            }
        }
    }
}
