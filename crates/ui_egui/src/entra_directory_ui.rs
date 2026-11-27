use crate::app::MyApp;
use crate::entra_application_registrations_ui::show_app_registration;
use crate::entra_application_registrations_ui::show_app_registrations;
use crate::entra_devops_permissions::show_entra_group_devops_permissions;
use crate::entra_directory_inventory::EntraDirectoryInventory;
use crate::entra_group_section::EntraGroupSection;
use crate::entra_group_sections_ui::show_entra_group_section;
use crate::entra_user_application_sections_ui::show_entra_user_application_assignments;
use crate::entra_user_application_sections_ui::show_entra_user_application_ownership;
use crate::entra_user_groups_ui::show_entra_user_group_memberships;
use crate::entra_user_groups_ui::show_entra_user_group_ownerships;
use crate::entra_user_permissions::show_entra_user_azure_roles;
use crate::entra_user_permissions::show_entra_user_devops_permissions;
use crate::entra_user_section::EntraUserSection;
use crate::entra_user_subject::EntraUserSubject;
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
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUser;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::CollapsingHeader;
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
use std::borrow::Cow;
use std::hash::Hash;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn show_entra_directory(app: &mut MyApp, ui: &mut Ui, target: &PortalTabTarget) {
    let (id, section) = match *target {
        PortalTabTarget::EntraGroup(id) => (id, EntraGroupSection::Overview),
        PortalTabTarget::EntraGroupSection { group_id, section } => (group_id, section),
        PortalTabTarget::EntraUser(id) => {
            return show_entra_user(
                app,
                ui,
                EntraUserSubject::Directory(id),
                EntraUserSection::Overview,
            );
        }
        PortalTabTarget::Page(PortalPage::EntraMyUser) => {
            return show_entra_user(
                app,
                ui,
                EntraUserSubject::SignedIn,
                EntraUserSection::Overview,
            );
        }
        PortalTabTarget::EntraUserSection { user, section } => {
            return show_entra_user(app, ui, user, section);
        }
        _ => return show_directory_page(app, ui, target),
    };
    let overview = PortalTabTarget::EntraGroup(id);
    app.load_entra_directory(&overview, false);
    let title = app.portal_tab_title(&overview);
    if resource_detail_header(
        ui,
        &title,
        "Group",
        "Groups",
        page_icon(PortalPage::EntraGroups),
        &PortalTabTarget::Page(PortalPage::EntraGroups),
    )
    .is_some_and(|open| open.apply(app))
    {
        return;
    }

    if show_directory_object_navigation(
        app,
        ui,
        EntraGroupSection::ALL.into_iter().map(|choice| {
            (
                choice.title(),
                choice.icon(),
                section == choice,
                choice.target(id),
            )
        }),
    ) {
        return;
    }

    Frame::NONE
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            if app.show_azure_authentication_required(ui) {
                return;
            }
            match section {
                EntraGroupSection::Overview | EntraGroupSection::Properties => {
                    show_group(app, ui, id, section);
                }
                EntraGroupSection::AzureDevOpsPermissions => {
                    show_entra_group_devops_permissions(app, ui, id);
                }
                _ => {
                    app.load_entra_group_section(id, section, false);
                    show_entra_group_section(app, ui, id, section);
                }
            }
        });
}

fn show_directory_object_navigation(
    app: &mut MyApp,
    ui: &mut Ui,
    sections: impl IntoIterator<Item = (&'static str, ImageSource<'static>, bool, PortalTabTarget)>,
) -> bool {
    let expanded = app.resource_navigation_expanded;
    let open = Panel::left(ui.make_persistent_id("entra-object-navigation"))
        .exact_size(if expanded { 228.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "resource navigation",
            );
            ScrollArea::vertical()
                .show(ui, |ui| {
                    let mut destination = None;
                    for (title, icon, current, target) in sections {
                        if let Some(open) =
                            navigation_tile(ui, title, icon, current, expanded, || target)
                        {
                            destination = Some(open);
                        }
                    }
                    destination
                })
                .inner
        })
        .inner;
    match open {
        Some(PortalTabOpen::Current(target)) => {
            app.tabs.replace_current(target);
            app.record_portal_visit();
            app.after_history_navigation();
            ui.ctx().request_repaint();
            true
        }
        Some(PortalTabOpen::Background(target)) => {
            app.open_background_tab(target);
            false
        }
        None => false,
    }
}

fn show_entra_user(
    app: &mut MyApp,
    ui: &mut Ui,
    subject: EntraUserSubject,
    section: EntraUserSection,
) {
    let overview = subject.overview();
    let requires_user_sign_in = subject == EntraUserSubject::SignedIn
        && app
            .ready_azure_authentication()
            .is_some_and(|authentication| {
                authentication.auth_source == AzureBearerAuthSource::WorkloadIdentity
            });
    if !requires_user_sign_in {
        app.load_entra_directory(&overview, false);
    }
    let my_user_name = app
        .azure
        .entra_directory
        .my_user
        .as_loaded()
        .map(|user| user.display_name.as_str())
        .filter(|name| !name.trim().is_empty() && !app.egui_config.demo_mode);
    let title = app.portal_tab_title(&overview);
    let name = match subject {
        EntraUserSubject::SignedIn => my_user_name.unwrap_or(&title),
        EntraUserSubject::Directory(_) => &title,
    };
    if resource_detail_header(
        ui,
        name,
        "User",
        "Users",
        page_icon(PortalPage::EntraUsers),
        &PortalTabTarget::Page(PortalPage::EntraUsers),
    )
    .is_some_and(|open| open.apply(app))
        || show_directory_object_navigation(
            app,
            ui,
            EntraUserSection::ALL.into_iter().map(|choice| {
                (
                    choice.title(),
                    choice.icon(),
                    section == choice,
                    choice.target(subject),
                )
            }),
        )
    {
        return;
    }
    Frame::NONE
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            if app.show_azure_authentication_required(ui) {
                return;
            }
            if requires_user_sign_in {
                ui.label("My User requires a user sign-in.");
                return;
            }
            let directory = &app.azure.entra_directory;
            if section == EntraUserSection::Overview {
                let response = match subject {
                    EntraUserSubject::Directory(id) => directory.user_details.get(&id),
                    EntraUserSubject::SignedIn => Some(&directory.my_user),
                };
                if let Some(user) =
                    response.and_then(|response| loaded(ui, response, "Could not load this user."))
                {
                    show_user_details(app, ui, user);
                }
                return;
            }
            let Some(id) = subject.user_id(directory) else {
                let _ = loaded(ui, &directory.my_user, "Could not load the signed-in user.");
                return;
            };
            match section {
                EntraUserSection::Overview => {}
                EntraUserSection::GroupMemberships => {
                    show_entra_user_group_memberships(app, ui, id)
                }
                EntraUserSection::GroupOwnerships => show_entra_user_group_ownerships(app, ui, id),
                EntraUserSection::ApplicationOwnership => {
                    show_entra_user_application_ownership(app, ui, id)
                }
                EntraUserSection::ApplicationAssignments => {
                    show_entra_user_application_assignments(app, ui, id)
                }
                EntraUserSection::AzureRoleAssignments => show_entra_user_azure_roles(app, ui, id),
                EntraUserSection::AzureDevOpsPermissions => {
                    show_entra_user_devops_permissions(app, ui, id)
                }
            }
        });
}

fn show_directory_page(app: &mut MyApp, ui: &mut Ui, target: &PortalTabTarget) {
    let page = target.page();
    let kind = match target {
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
                (PortalPage::EntraMyUser, "My User"),
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
    app.azure
        .entra_directory
        .users
        .receive_pages(|user| user.id);
    let inventory = &app.azure.entra_directory.users;
    show_inventory_errors(ui, inventory, "Could not load users.");
    let progress = inventory_progress(inventory);
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
    let id = Id::new(("entra-users", app.azure.account_generation));
    let table = ReflectedTable::new(id, columns)
        .with_search_hint("Search users by name or ID")
        .with_status(progress.as_deref());
    let (target, term) = with_iam_thumbnails(
        &app.azure.scope_iam.thumbnails,
        app.egui_config.demo_mode,
        ui,
        |ui| {
            let target = table.show_explorable_matching(
                ui,
                inventory.rows(|user| user.id),
                |user| id.with(user.id),
                &mut app.azure.checkboxes,
                |user, search| {
                    !app.egui_config.demo_mode && inventory.lookup_matched(search, user.id)
                },
            );
            (target, table.search_text(ui))
        },
    );
    app.search_entra_directory(PortalPage::EntraUsers, &term, ui.ctx());
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
    app.azure
        .entra_directory
        .groups
        .receive_pages(|group| group.id);
    let inventory = &app.azure.entra_directory.groups;
    show_inventory_errors(ui, inventory, "Could not load groups.");
    let progress = inventory_progress(inventory);
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
    let id = Id::new(("entra-groups", app.azure.account_generation));
    let table = ReflectedTable::new(id, columns)
        .with_search_hint("Search groups by name or ID")
        .with_status(progress.as_deref());
    let target = table.show_explorable_matching(
        ui,
        inventory.rows(|group| group.id),
        |group| id.with(group.id),
        &mut app.azure.checkboxes,
        |group, search| !app.egui_config.demo_mode && inventory.lookup_matched(search, group.id),
    );
    app.search_entra_directory(PortalPage::EntraGroups, &table.search_text(ui), ui.ctx());
    if let Some(target) = target {
        target.apply(app);
    }
}

pub(crate) fn inventory_progress<T, I: Copy + Eq + Hash>(
    inventory: &EntraDirectoryInventory<T, I>,
) -> Option<Cow<'static, str>> {
    if matches!(inventory.lookup, Loadable::Loading { .. }) {
        return Some(Cow::Borrowed("Searching Microsoft Graph"));
    }
    match inventory.inventory {
        Loadable::Loading { .. } | Loadable::NotLoaded => {
            let count = inventory.primary_rows().len();
            let progress = if let Some(total) = inventory.total {
                format!("Loaded {count} of {total} directory entries")
            } else {
                format!("Loading directory entries ({count} so far)")
            };
            Some(Cow::Owned(progress))
        }
        _ => None,
    }
}

pub(crate) fn show_inventory_errors<T, I>(
    ui: &mut Ui,
    inventory: &EntraDirectoryInventory<T, I>,
    failure: &str,
) {
    if matches!(inventory.inventory, Loadable::Failed { .. }) {
        loaded(ui, &inventory.inventory, failure);
    }
    if matches!(inventory.lookup, Loadable::Failed { .. }) {
        loaded(ui, &inventory.lookup, "Microsoft Graph lookup failed.");
    }
}

fn group_icon(_: &Ui, _: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
    Some(page_icon(PortalPage::EntraGroups))
}

fn show_user_details(app: &MyApp, ui: &mut Ui, user: &EntraUser) {
    let id = user.id;
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
            ui.collapsing("Graph response", |ui| ReflectedValue::new(user).show(ui));
        }
    });
}

fn show_group(app: &MyApp, ui: &mut Ui, id: EntraGroupId, section: EntraGroupSection) {
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
        if section == EntraGroupSection::Properties && !app.egui_config.demo_mode {
            ui.separator();
            ReflectedValue::new(group.as_ref()).show(ui);
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
            CollapsingHeader::new("Error details")
                .id_salt(failure)
                .show(ui, |ui| DiagnosticReportView::new(error).show(ui));
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
