use crate::icons::page_icon;
use crate::inventory_table::show_inventory_table;
use crate::management_group_hierarchy::ManagementGroupHierarchy;
use crate::management_group_tree_row::ManagementGroupTreeRow;
use crate::portal_page::PortalPage;
use crate::portal_search::portal_search;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table_sort::ReflectedTableSort;
use crate::table_row_background::TableRowBackground;
use cloud_terrastodon_azure::ManagementGroupId;
use cloud_terrastodon_azure::Scope;
use eframe::egui::Button;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::ScrollArea;
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
) -> Option<PortalTabOpen> {
    ui.push_id((id, subscriptions_only), |ui| {
        let tree_id = ui.id();
        let search_id = tree_id.with("search");
        let sort_id = tree_id.with("sort");
        let mut sort = ui
            .data(|data| data.get_temp::<ReflectedTableSort>(sort_id))
            .unwrap_or(ReflectedTableSort::INITIAL);
        let mut search: String = ui.data(|data| data.get_temp(search_id)).unwrap_or_default();
        if portal_search(ui, &mut search, "Search by name or ID", 280.0).changed() {
            ui.data_mut(|data| data.insert_temp(search_id, search.clone()));
        }

        let query = search.trim().to_lowercase();
        let mut rows = Vec::new();
        if subscriptions_only || !query.is_empty() {
            append_search_rows(hierarchy, id, &query, subscriptions_only, &mut rows);
            sort_rows(&mut rows, hierarchy, sort, subscriptions_only);
        } else {
            append_open_rows(ui, hierarchy, id, sort, 0, &mut rows);
        }
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
                        for (column, heading) in headings.iter().enumerate() {
                            header.col(|ui| {
                                sort.show_header(ui, heading, column, sort_id);
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
                                    if let Some(action) = PortalTabOpen::from_response(
                                        &ui.link(&group.display_name)
                                            .on_hover_text(&group.display_name),
                                        || PortalTabTarget::ManagementGroup(group.id()),
                                    ) {
                                        destination = Some(action);
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
                                    if let Some(action) = PortalTabOpen::from_response(
                                        &ui.link(subscription.name.as_str())
                                            .on_hover_text(subscription.name.as_str()),
                                        || PortalTabTarget::Subscription(subscription.id),
                                    ) {
                                        destination = Some(action);
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
                                            && let Some(action) = PortalTabOpen::from_response(
                                                &ui.link(&parent.display_name)
                                                    .on_hover_text(&parent.display_name),
                                                || PortalTabTarget::ManagementGroup(parent.id()),
                                            )
                                        {
                                            destination = Some(action);
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

/// Sort each level before visiting its expanded branches so subtrees stay together.
fn append_open_rows<'a>(
    ui: &Ui,
    hierarchy: &'a ManagementGroupHierarchy<'_>,
    id: &ManagementGroupId,
    sort: ReflectedTableSort,
    depth: usize,
    rows: &mut Vec<ManagementGroupTreeRow<'a>>,
) {
    let groups = hierarchy.child_groups(id).map(|group| {
        let child_id = group.id();
        let expandable = hierarchy.child_groups(&child_id).next().is_some()
            || !hierarchy.subscriptions(&child_id).is_empty();
        let expanded = expandable.then(|| {
            ui.data(|data| data.get_temp(ui.id().with(group.name.as_str())))
                .unwrap_or(false)
        });
        ManagementGroupTreeRow::Group {
            group,
            depth,
            expanded,
        }
    });
    let subscriptions = hierarchy.subscriptions(id).iter().map(|&subscription| {
        ManagementGroupTreeRow::Subscription {
            subscription,
            depth,
        }
    });
    let mut siblings = groups.chain(subscriptions).collect::<Vec<_>>();
    sort_rows(&mut siblings, hierarchy, sort, false);
    for sibling in siblings {
        let expanded_group = match &sibling {
            ManagementGroupTreeRow::Group {
                group,
                expanded: Some(true),
                ..
            } => Some(*group),
            _ => None,
        };
        rows.push(sibling);
        if let Some(group) = expanded_group {
            append_open_rows(ui, hierarchy, &group.id(), sort, depth + 1, rows);
        }
    }
}

fn append_search_rows<'a>(
    hierarchy: &'a ManagementGroupHierarchy<'_>,
    id: &ManagementGroupId,
    query: &str,
    subscriptions_only: bool,
    rows: &mut Vec<ManagementGroupTreeRow<'a>>,
) {
    for group in hierarchy.child_groups(id) {
        let child_id = group.id();
        if !subscriptions_only && matches_query(&group.display_name, &child_id.to_string(), query) {
            rows.push(ManagementGroupTreeRow::Group {
                group,
                depth: 0,
                expanded: None,
            });
        }
        append_search_rows(hierarchy, &child_id, query, subscriptions_only, rows);
    }
    rows.extend(
        hierarchy
            .subscriptions(id)
            .iter()
            .filter_map(|&subscription| {
                (query.is_empty()
                    || matches_query(
                        subscription.name.as_str(),
                        &subscription.id.expanded_form(),
                        query,
                    ))
                .then_some(ManagementGroupTreeRow::Subscription {
                    subscription,
                    depth: 0,
                })
            }),
    );
}

fn sort_rows(
    rows: &mut [ManagementGroupTreeRow<'_>],
    hierarchy: &ManagementGroupHierarchy<'_>,
    sort: ReflectedTableSort,
    subscriptions_only: bool,
) {
    match sort.column {
        1 => rows.sort_by_cached_key(|row| row.identifier().to_lowercase()),
        2 if subscriptions_only => {
            rows.sort_by_cached_key(|row| row.parent_name().to_lowercase());
        }
        2 => rows.sort_by_key(|row| matches!(row, ManagementGroupTreeRow::Subscription { .. })),
        3 => rows.sort_by_cached_key(|row| match row {
            ManagementGroupTreeRow::Group { group, .. } => {
                Some(hierarchy.subscription_count(&group.id()))
            }
            ManagementGroupTreeRow::Subscription { .. } => None,
        }),
        _ => rows.sort_by_cached_key(|row| row.name().to_lowercase()),
    }
    if sort.descending {
        rows.reverse();
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_azure::ManagementGroup;
    use cloud_terrastodon_azure::ManagementGroupAncestorsChain;
    use cloud_terrastodon_azure::ManagementGroupAncestorsChainEntry;
    use eframe::egui::Context;
    use facet_reflect::Partial;
    use std::sync::Arc;

    fn group(name: &str, parent: &str) -> eyre::Result<Arc<ManagementGroup>> {
        let ancestors = vec![ManagementGroupAncestorsChainEntry {
            name: parent.into(),
            display_name: parent.into(),
        }];
        Ok(Arc::new(ManagementGroup {
            id: ManagementGroupId::from_name(name),
            display_name: name.into(),
            tenant_id: "00000000-0000-0000-0000-000000000001".parse()?,
            management_group_ancestors_chain: Partial::alloc::<ManagementGroupAncestorsChain>()?
                .begin_inner()?
                .set(ancestors)?
                .end()?
                .build()?
                .materialize()?,
        }))
    }

    #[test]
    fn descending_sort_keeps_an_expanded_child_with_its_parent() -> eyre::Result<()> {
        let groups = [
            group("Bravo", "Root")?,
            group("Zulu", "Root")?,
            group("Alpha", "Zulu")?,
        ];
        let hierarchy = ManagementGroupHierarchy::new(&groups, &[]);
        let mut rows = Vec::new();
        let mut output = Context::default().run_ui(Default::default(), |ui| {
            ui.data_mut(|data| data.insert_temp(ui.id().with("Zulu"), true));
            append_open_rows(
                ui,
                &hierarchy,
                &ManagementGroupId::from_name("Root"),
                ReflectedTableSort {
                    column: 0,
                    descending: true,
                },
                0,
                &mut rows,
            );
        });
        output.textures_delta.clear();
        let names = rows
            .iter()
            .map(ManagementGroupTreeRow::name)
            .collect::<Vec<_>>();
        assert_eq!(names, ["Zulu", "Alpha", "Bravo"]);
        Ok(())
    }
}
