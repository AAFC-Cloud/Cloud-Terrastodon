use crate::icons::page_icon;
use crate::inventory_table::show_inventory_table;
use crate::management_group_hierarchy::ManagementGroupHierarchy;
use crate::management_group_tree_row::ManagementGroupTreeRow;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use crate::table_row_background::TableRowBackground;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::Scope;
use eframe::egui::Button;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_extras::Column;

/// Show an expandable child hierarchy or a flat list of descendant subscriptions.
pub(crate) fn show_management_group_tree(
    ui: &mut Ui,
    hierarchy: &ManagementGroupHierarchy<'_>,
    id: &ManagementGroupId,
    subscriptions_only: bool,
    subscriptions_available: bool,
) -> Option<PortalTabTarget> {
    ui.push_id((id, subscriptions_only), |ui| {
        let tree_id = ui.id();
        let search_id = tree_id.with("search");
        let mut search: String = ui.data(|data| data.get_temp(search_id)).unwrap_or_default();
        if ui
            .add(
                TextEdit::singleline(&mut search)
                    .hint_text("Search by name or ID")
                    .desired_width(280.0),
            )
            .changed()
        {
            ui.data_mut(|data| data.insert_temp(search_id, search.clone()));
        }

        let query = search.trim().to_lowercase();
        let mut rows = Vec::new();
        append_rows(ui, hierarchy, id, &query, subscriptions_only, 0, &mut rows);
        if rows.is_empty() {
            ui.label(
                match (
                    query.is_empty(),
                    subscriptions_only,
                    subscriptions_available,
                ) {
                    (false, _, _) => "No results match your search.",
                    (true, true, _) => "No subscriptions are visible under this group.",
                    (true, false, true) => "No child groups or subscriptions are visible.",
                    (true, false, false) => "No child groups are visible in the loaded inventory.",
                },
            );
            return None;
        }

        let mut destination = None;
        ScrollArea::horizontal().show(ui, |ui| {
            show_inventory_table(ui, |table| {
                let table = table
                    .column(Column::initial(260.0).at_least(125.0).clip(true))
                    .column(Column::remainder().at_least(170.0).clip(true));
                let (table, headings) = if subscriptions_only {
                    (
                        table.column(Column::initial(170.0).at_least(100.0).clip(true)),
                        &["Name", "ID", "Parent group"][..],
                    )
                } else {
                    (
                        table
                            .column(Column::initial(120.0).at_least(100.0).clip(true))
                            .column(Column::initial(100.0).at_least(80.0).clip(true)),
                        &["Name", "ID", "Type", "Subscriptions"][..],
                    )
                };
                table
                    .header(28.0, |mut header| {
                        for heading in headings {
                            header.col(|ui| {
                                ui.strong(*heading);
                            });
                        }
                    })
                    .body(|mut body| {
                        let background = TableRowBackground::new(&mut body);
                        body.rows(30.0, rows.len(), |mut row| match &rows[row.index()] {
                            ManagementGroupTreeRow::Group {
                                group,
                                depth,
                                expanded,
                            } => {
                                background.first_cell(&mut row, |ui| {
                                    ui.add_space(*depth as f32 * 16.0);
                                    if let Some(open) = expanded {
                                        let toggle = ui
                                            .add_sized(
                                                [16.0, 24.0],
                                                Button::new(if *open { "▾" } else { "▸" })
                                                    .frame(false),
                                            )
                                            .on_hover_text(if *open {
                                                "Collapse group"
                                            } else {
                                                "Expand group"
                                            });
                                        if toggle.clicked() {
                                            ui.data_mut(|data| {
                                                data.insert_temp(
                                                    tree_id.with(group.name.as_str()),
                                                    !*open,
                                                );
                                            });
                                            ui.ctx().request_repaint();
                                        }
                                    } else {
                                        ui.add_space(16.0);
                                    }
                                    ui.add(
                                        Image::new(page_icon(PortalPage::ManagementGroups))
                                            .fit_to_exact_size(Vec2::splat(18.0)),
                                    );
                                    if ui
                                        .link(&group.display_name)
                                        .on_hover_text(&group.display_name)
                                        .clicked()
                                    {
                                        destination =
                                            Some(PortalTabTarget::ManagementGroup(group.id()));
                                    }
                                    false
                                });
                                row.col(|ui| {
                                    copyable_id(ui, &group.name, || group.id().to_string())
                                });
                                row.col(|ui| {
                                    ui.label("Management group");
                                });
                                row.col(|ui| {
                                    if subscriptions_available {
                                        ui.label(
                                            hierarchy.subscription_count(&group.id()).to_string(),
                                        );
                                    } else {
                                        ui.label("—");
                                    }
                                });
                            }
                            ManagementGroupTreeRow::Subscription {
                                subscription,
                                depth,
                            } => {
                                background.first_cell(&mut row, |ui| {
                                    if !subscriptions_only {
                                        ui.add_space(*depth as f32 * 16.0 + 16.0);
                                    }
                                    ui.add(
                                        Image::new(page_icon(PortalPage::Subscriptions))
                                            .fit_to_exact_size(Vec2::splat(18.0)),
                                    );
                                    if ui
                                        .link(subscription.name.as_str())
                                        .on_hover_text(subscription.name.as_str())
                                        .clicked()
                                    {
                                        destination =
                                            Some(PortalTabTarget::Subscription(subscription.id));
                                    }
                                    false
                                });
                                row.col(|ui| {
                                    copyable_id(ui, &subscription.id.to_string(), || {
                                        subscription.id.expanded_form()
                                    })
                                });
                                row.col(|ui| {
                                    if subscriptions_only {
                                        if let Some(parent) =
                                            subscription.management_group_ancestors_chain.first()
                                            && ui
                                                .link(&parent.display_name)
                                                .on_hover_text(&parent.display_name)
                                                .clicked()
                                        {
                                            destination =
                                                Some(PortalTabTarget::ManagementGroup(parent.id()));
                                        }
                                    } else {
                                        ui.label("Subscription");
                                    }
                                });
                                if !subscriptions_only {
                                    row.col(|_| {});
                                }
                            }
                        });
                    });
            });
        });
        destination
    })
    .inner
}

fn append_rows<'a>(
    ui: &Ui,
    hierarchy: &'a ManagementGroupHierarchy<'_>,
    id: &ManagementGroupId,
    query: &str,
    subscriptions_only: bool,
    depth: usize,
    rows: &mut Vec<ManagementGroupTreeRow<'a>>,
) {
    let searching = !query.is_empty();
    let tree = !subscriptions_only && !searching;
    for group in hierarchy.child_groups(id) {
        let child_id = group.id();
        let expandable = tree
            && (hierarchy.child_groups(&child_id).next().is_some()
                || !hierarchy.subscriptions(&child_id).is_empty());
        let expanded = expandable.then(|| {
            ui.data(|data| data.get_temp(ui.id().with(group.name.as_str())))
                .unwrap_or(false)
        });
        if !subscriptions_only
            && (!searching || matches_query(&group.display_name, &child_id.to_string(), query))
        {
            rows.push(ManagementGroupTreeRow::Group {
                group,
                depth,
                expanded,
            });
        }
        if !tree || expanded == Some(true) {
            append_rows(
                ui,
                hierarchy,
                &child_id,
                query,
                subscriptions_only,
                if tree { depth + 1 } else { 0 },
                rows,
            );
        }
    }
    rows.extend(
        hierarchy
            .subscriptions(id)
            .iter()
            .filter_map(|&subscription| {
                (!searching
                    || matches_query(
                        subscription.name.as_str(),
                        &subscription.id.expanded_form(),
                        query,
                    ))
                .then_some(ManagementGroupTreeRow::Subscription {
                    subscription,
                    depth,
                })
            }),
    );
}

fn matches_query(name: &str, id: &str, query: &str) -> bool {
    name.to_lowercase().contains(query) || id.to_lowercase().contains(query)
}

fn copyable_id(ui: &mut Ui, value: &str, expanded: impl FnOnce() -> String) {
    ui.add(Label::new(value).truncate())
        .on_hover_text(value)
        .context_menu(|ui| {
            if ui.button("Copy resource ID").clicked() {
                ui.ctx().copy_text(expanded());
                ui.close();
            }
        });
}
