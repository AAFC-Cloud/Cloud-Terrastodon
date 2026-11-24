use crate::app::MyApp;
use crate::entra_application_registrations_ui::show_app_registration;
use crate::entra_application_registrations_ui::show_app_registrations;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_value::ReflectedValue;
use crate::resource_detail_header::resource_detail_header;
use crate::scope_iam_thumbnails::load_queued_iam_thumbnails;
use crate::scope_iam_thumbnails::scope_iam_thumbnail;
use crate::scope_iam_thumbnails::with_iam_thumbnails;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CornerRadius;
use eframe::egui::Frame;
use eframe::egui::Grid;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::ImageSource;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use std::sync::Arc;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn show_entra_directory(app: &mut MyApp, ui: &mut Ui, target: &PortalTabTarget) {
    let page = target.page();
    let kind = match target {
        PortalTabTarget::EntraUser(_) => Some("User"),
        PortalTabTarget::EntraGroup(_) => Some("Group"),
        PortalTabTarget::EntraAppRegistration(_) => Some("App registration"),
        _ => None,
    };
    if let Some(kind) = kind {
        let title = app.portal_tab_title(target);
        if resource_detail_header(
            ui,
            &title,
            kind,
            page.title(),
            page_icon(page),
            &PortalTabTarget::Page(page),
        )
        .is_some_and(|open| open.apply(app))
        {
            return;
        }
    } else {
        Frame::NONE
            .inner_margin(Margin::symmetric(16, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(Image::new(page_icon(page)).fit_to_exact_size(Vec2::splat(30.0)));
                    ui.label(RichText::new(page.title()).strong().size(22.0));
                });
            });
    }

    let expanded = app.resource_navigation_expanded;
    let destination = Panel::left(ui.make_persistent_id("entra-directory-navigation"))
        .exact_size(if expanded { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "directory navigation",
            );
            let mut next = None;
            for (section, label) in [
                (PortalPage::Entra, "Overview"),
                (PortalPage::EntraUsers, "Users"),
                (PortalPage::EntraGroups, "Groups"),
                (PortalPage::EntraAppRegistrations, "App registrations"),
            ] {
                if let Some(open) = navigation_tile(
                    ui,
                    label,
                    page_icon(section),
                    section == page,
                    expanded,
                    || PortalTabTarget::Page(section),
                ) {
                    next = Some(open);
                }
            }
            next
        })
        .inner;
    if destination.is_some_and(|open| open.apply(app)) {
        return;
    }

    Frame::NONE
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            if app.show_azure_authentication_required(ui) {
                return;
            }
            app.load_entra_directory(target, false);
            match *target {
                PortalTabTarget::Page(PortalPage::Entra) => show_directory_overview(app, ui),
                PortalTabTarget::Page(PortalPage::EntraUsers) => show_users(app, ui),
                PortalTabTarget::Page(PortalPage::EntraGroups) => show_groups(app, ui),
                PortalTabTarget::Page(PortalPage::EntraAppRegistrations) => {
                    show_app_registrations(app, ui)
                }
                PortalTabTarget::EntraUser(id) => show_user(app, ui, id),
                PortalTabTarget::EntraGroup(id) => show_group(app, ui, id),
                PortalTabTarget::EntraAppRegistration(id) => show_app_registration(app, ui, id),
                _ => {}
            }
        });
}

fn show_directory_overview(app: &mut MyApp, ui: &mut Ui) {
    if let Some(authentication) = app.ready_azure_authentication() {
        ui.horizontal(|ui| {
            ui.label("Tenant ID");
            let id = authentication.tenant_id.to_string();
            ui.monospace(&id);
            if ui.small_button("Copy ID").clicked() {
                ui.ctx().copy_text(id);
            }
        });
    }
    ui.add_space(12.0);
    for (label, page) in [
        ("Browse users", PortalPage::EntraUsers),
        ("Browse groups", PortalPage::EntraGroups),
        (
            "Browse app registrations",
            PortalPage::EntraAppRegistrations,
        ),
    ] {
        if let Some(action) =
            PortalTabOpen::from_response(&ui.link(label), || PortalTabTarget::Page(page))
        {
            action.apply(app);
        }
    }
}

fn show_users(app: &mut MyApp, ui: &mut Ui) {
    let Some(users) = loaded(
        ui,
        &app.azure.entra_directory.users,
        "Could not load users.",
    ) else {
        return;
    };
    let columns: &[ReflectedTableColumn] = if app.egui_config.demo_mode {
        &[ReflectedTableColumn::new("Object ID", "id")
            .with_link("id")
            .with_icon(user_thumbnail)]
    } else {
        &[
            ReflectedTableColumn::new("Display name", "display_name")
                .with_width(260.0)
                .with_link("id")
                .with_icon(user_thumbnail)
                .with_secondary_text("id"),
            ReflectedTableColumn::new("User principal name", "user_principal_name")
                .with_width(245.0),
            ReflectedTableColumn::new("Job title", "job_title").groupable(),
        ]
    };
    ui.weak(format!("{} users", users.len()));
    let id = Id::new(("entra-users", app.azure.account_generation));
    let target = with_iam_thumbnails(
        &app.azure.scope_iam.thumbnails,
        app.egui_config.demo_mode,
        ui,
        |ui| {
            ReflectedTable::new(id, columns).show_explorable(
                ui,
                users.iter().map(Arc::as_ref),
                |user| id.with(user.id),
                &mut app.azure.checkboxes,
            )
        },
    );
    load_queued_iam_thumbnails(app, ui);
    if let Some(target) = target {
        target.apply(app);
    }
}

fn user_thumbnail(ui: &Ui, row: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    let id = *row.column("id")?.get::<EntraUserId>()?;
    scope_iam_thumbnail(ui, id).or_else(|| Some(page_icon(PortalPage::EntraUsers)))
}

fn show_groups(app: &mut MyApp, ui: &mut Ui) {
    let Some(groups) = loaded(
        ui,
        &app.azure.entra_directory.groups,
        "Could not load groups.",
    ) else {
        return;
    };
    let object_id = ReflectedTableColumn::new("Object ID", "id").with_width(260.0);
    let security = ReflectedTableColumn::new("Security enabled", "security_enabled")
        .with_width(120.0)
        .groupable();
    let visibility = ReflectedTableColumn::new("Visibility", "visibility")
        .with_width(120.0)
        .groupable();
    let columns: &[ReflectedTableColumn] = if app.egui_config.demo_mode {
        &[
            object_id.with_link("id").with_icon(group_icon),
            security,
            visibility,
        ]
    } else {
        &[
            ReflectedTableColumn::new("Display name", "display_name")
                .with_width(260.0)
                .with_link("id")
                .with_icon(group_icon),
            object_id,
            security,
            visibility,
            ReflectedTableColumn::new("Email", "mail"),
        ]
    };
    ui.weak(format!("{} groups", groups.len()));
    let id = Id::new(("entra-groups", app.azure.account_generation));
    if let Some(target) = ReflectedTable::new(id, columns).show_explorable(
        ui,
        groups.iter().map(Arc::as_ref),
        |group| id.with(group.id),
        &mut app.azure.checkboxes,
    ) {
        target.apply(app);
    }
}

fn group_icon(_: &Ui, _: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    Some(page_icon(PortalPage::EntraGroups))
}

fn show_user(app: &MyApp, ui: &mut Ui, id: EntraUserId) {
    let Some(response) = app.azure.entra_directory.user_details.get(&id) else {
        return;
    };
    let Some(user) = loaded(ui, response, "Could not load this user.") else {
        return;
    };
    with_iam_thumbnails(
        &app.azure.scope_iam.thumbnails,
        app.egui_config.demo_mode,
        ui,
        |ui| {
            let icon =
                scope_iam_thumbnail(ui, id).unwrap_or_else(|| page_icon(PortalPage::EntraUsers));
            ui.add(
                Image::new(icon)
                    .fit_to_exact_size(Vec2::splat(48.0))
                    .corner_radius(CornerRadius::same(24)),
            );
        },
    );
    load_queued_iam_thumbnails(app, ui);

    ScrollArea::vertical().show(ui, |ui| {
        Grid::new(("entra-user", id))
            .spacing([18.0, 6.0])
            .show(ui, |ui| {
                copyable_id(ui, "Object ID", id);
                if !app.egui_config.demo_mode {
                    property(
                        ui,
                        "User principal name",
                        Some(user.user_principal_name.as_str()),
                    );
                    property(ui, "Email", user.mail.as_deref());
                    property(ui, "Job title", user.job_title.as_deref());
                    property(ui, "Office", user.office_location.as_deref());
                }
            });
        if !app.egui_config.demo_mode {
            ui.collapsing("Graph response", |ui| {
                ReflectedValue::new(user.as_ref()).show(ui)
            });
        }
    });
}

fn show_group(app: &MyApp, ui: &mut Ui, id: EntraGroupId) {
    let Some(response) = app.azure.entra_directory.group_details.get(&id) else {
        return;
    };
    let Some(group) = loaded(ui, response, "Could not load this group.") else {
        return;
    };
    ScrollArea::vertical().show(ui, |ui| {
        Grid::new(("entra-group", id))
            .spacing([18.0, 6.0])
            .show(ui, |ui| {
                copyable_id(ui, "Object ID", id);
                property(ui, "Visibility", group.visibility.as_deref());
                property(
                    ui,
                    "Security enabled",
                    Some(if group.security_enabled { "Yes" } else { "No" }),
                );
                if !app.egui_config.demo_mode {
                    property(ui, "Email", group.mail.as_deref());
                    property(ui, "Description", group.description.as_deref());
                }
            });
        if !app.egui_config.demo_mode {
            ui.collapsing("Graph response", |ui| {
                ReflectedValue::new(group.as_ref()).show(ui)
            });
        }
    });
}

pub(crate) fn loaded<'a, T>(
    ui: &mut Ui,
    response: &'a Loadable<T>,
    failure: &str,
) -> Option<&'a T> {
    match response {
        Loadable::Loaded { value, .. } => Some(value),
        Loadable::Failed { error, .. } => {
            ui.label(failure);
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui)
            });
            None
        }
        Loadable::Loading { .. } | Loadable::NotLoaded => {
            ui.spinner();
            None
        }
    }
}

pub(crate) fn copyable_id(ui: &mut Ui, name: &str, id: impl ToString) {
    let id = id.to_string();
    ui.label(name);
    ui.horizontal_wrapped(|ui| {
        ui.monospace(&id);
        if ui
            .small_button("Copy ID")
            .on_hover_text(format!("Copy {name}"))
            .clicked()
        {
            ui.ctx().copy_text(id);
        }
    });
    ui.end_row();
}

fn property(ui: &mut Ui, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        ui.label(name);
        ui.label(value);
        ui.end_row();
    }
}
