use crate::app::MyApp;
use crate::inventory_table::show_inventory_table;
use crate::portal_storage_audit_row::PortalStorageAuditRow;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table_sort::ReflectedTableSort;
use crate::table_row_background::TableRowBackground;
use crate::workers::load_subscriptions::load_subscriptions;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure_cli::AzureStorageAccountAuditFinding;
use cloud_terrastodon_azure_cli::AzureStorageAccountAuditReport;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Button;
use eframe::egui::Id;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use egui_extras::Column;
use std::borrow::Cow;
use std::collections::HashMap;
use teamy_tracing::gui::escape_display;

pub(crate) fn show_storage_audit_findings(
    app: &mut MyApp,
    ui: &mut Ui,
    report: &AzureStorageAccountAuditReport,
    result_id: Id,
) {
    let confirmed = report
        .findings
        .iter()
        .filter(|finding| container_count(finding).is_some())
        .count();
    ui.horizontal_wrapped(|ui| {
        ui.strong(format!(
            "{} storage accounts observed",
            report.accounts_observed
        ));
        ui.label(format!("{confirmed} with unused anonymous access"));
        ui.label(format!(
            "{} inconclusive",
            report.findings.len() - confirmed
        ));
    });
    if report.findings.is_empty() {
        ui.label("No unused account-level anonymous blob access found.");
        return;
    }

    if matches!(app.azure.subscriptions, Loadable::NotLoaded) {
        load_subscriptions(app);
    }
    let subscription_names: HashMap<_, _> = app
        .azure
        .subscriptions
        .as_loaded()
        .into_iter()
        .flat_map(|subscriptions| subscriptions.iter())
        .map(|subscription| (subscription.id, subscription.name.as_str()))
        .collect();
    let table_id = result_id.with("findings");
    let sort_id = table_id.with("sort");
    let mut sort = ui
        .data(|data| data.get_temp::<ReflectedTableSort>(sort_id))
        .unwrap_or(ReflectedTableSort::INITIAL);
    let mut findings: Vec<_> = report
        .findings
        .iter()
        .map(|finding| {
            let id = finding.account().id.resource_group_id.subscription_id;
            let label = subscription_names
                .get(&id)
                .map_or_else(|| Cow::Owned(id.to_string()), |name| Cow::Borrowed(*name));
            (finding, label)
        })
        .collect();
    match sort.column {
        1 => findings.sort_by_key(|(finding, _)| verdict(finding)),
        2 => findings.sort_by_key(|(finding, _)| container_count(finding)),
        3 => findings.sort_by_cached_key(|(finding, _)| {
            finding
                .account()
                .id
                .resource_group_id
                .resource_group_name
                .as_str()
                .to_lowercase()
        }),
        4 => findings.sort_by_cached_key(|(_, subscription)| subscription.to_lowercase()),
        _ => findings
            .sort_by_cached_key(|(finding, _)| finding.account().name.as_str().to_lowercase()),
    }
    if sort.descending {
        findings.reverse();
    }

    let mut rows = Vec::with_capacity(findings.len());
    for (finding, subscription_name) in findings {
        let expansion_id = table_id.with(("expanded", &finding.account().id));
        let expanded = ui.data(|data| data.get_temp::<bool>(expansion_id).unwrap_or_default());
        rows.push(PortalStorageAuditRow::Finding {
            finding,
            subscription_name,
            expanded,
        });
        if expanded {
            PortalStorageAuditRow::append_evidence(&mut rows, finding);
        }
    }

    let height = ui.available_height();
    let mut navigation = None;
    ScrollArea::horizontal().id_salt(table_id).show(ui, |ui| {
        show_inventory_table(ui, |table| {
            table
                .id_salt(table_id)
                .max_scroll_height((height - 16.0).max(0.0))
                .column(Column::initial(230.0).at_least(120.0).clip(true))
                .column(Column::initial(240.0).at_least(130.0).clip(true))
                .column(Column::initial(125.0).at_least(110.0).clip(true))
                .column(Column::initial(200.0).at_least(110.0).clip(true))
                .column(Column::remainder().at_least(160.0).clip(true))
                .header(28.0, |mut header| {
                    for (index, label) in [
                        "Storage account",
                        "Verdict",
                        "Container count",
                        "Resource group",
                        "Subscription",
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        header.col(|ui| sort.show_header(ui, label, index, sort_id));
                    }
                })
                .body(|mut body| {
                    let background = TableRowBackground::new(&mut body);
                    body.rows(29.0, rows.len(), |mut row| match &rows[row.index()] {
                        PortalStorageAuditRow::Finding {
                            finding,
                            subscription_name,
                            expanded,
                        } => {
                            let account = finding.account();
                            let group = &account.id.resource_group_id;
                            background.first_cell(&mut row, |ui| {
                                let (arrow, action) = if *expanded {
                                    ("▾", "Collapse finding")
                                } else {
                                    ("▸", "Expand finding")
                                };
                                if ui
                                    .add(Button::new(arrow).frame(false))
                                    .on_hover_text(action)
                                    .clicked()
                                {
                                    ui.data_mut(|data| {
                                        data.insert_temp(
                                            table_id.with(("expanded", &account.id)),
                                            !*expanded,
                                        )
                                    });
                                    ui.ctx().request_repaint();
                                }
                                navigation = audit_link(
                                    ui,
                                    account.name.as_str(),
                                    account.id.expanded_form(),
                                    || PortalTabTarget::StorageAccount(account.id.clone()),
                                )
                                .or(navigation.take());
                                false
                            });
                            let count = container_count(finding);
                            row.col(|ui| {
                                let label = verdict(finding);
                                let text = RichText::new(label);
                                let text = if count.is_some() {
                                    text.color(ui.visuals().warn_fg_color)
                                } else {
                                    text.weak()
                                };
                                ui.add(Label::new(text).truncate()).on_hover_text(label);
                            });
                            row.col(|ui| {
                                if let Some(count) = count {
                                    ui.label(count.to_string());
                                } else {
                                    ui.weak("—").on_hover_text("Container count unavailable");
                                }
                            });
                            row.col(|ui| {
                                navigation = audit_link(
                                    ui,
                                    group.resource_group_name.as_str(),
                                    group.expanded_form(),
                                    || PortalTabTarget::ResourceGroup(group.clone()),
                                )
                                .or(navigation.take());
                            });
                            row.col(|ui| {
                                let id = group.subscription_id;
                                navigation =
                                    audit_link(ui, subscription_name, id.expanded_form(), || {
                                        PortalTabTarget::Subscription(id)
                                    })
                                    .or(navigation.take());
                            });
                        }
                        PortalStorageAuditRow::Evidence {
                            label,
                            value,
                            container_id,
                        } => {
                            background.first_cell(&mut row, |ui| {
                                ui.add_space(22.0);
                                let text = escape_display(label);
                                ui.add(Label::new(RichText::new(&text).weak()).truncate())
                                    .on_hover_text(text);
                                false
                            });
                            row.col(|ui| {
                                if let Some(id) = container_id {
                                    navigation = audit_link(ui, value, id.to_string(), || {
                                        PortalTabTarget::StorageContainer((*id).clone())
                                    })
                                    .or(navigation.take());
                                } else {
                                    let text = escape_display(value);
                                    ui.add(Label::new(&text).truncate()).on_hover_text(text);
                                }
                            });
                        }
                    });
                });
        });
    });
    if let Some(open) = navigation {
        open.apply(app);
    }
}

fn verdict(finding: &AzureStorageAccountAuditFinding) -> &'static str {
    match finding {
        AzureStorageAccountAuditFinding::UnusedAnonymousAccess { .. } => "Unused anonymous access",
        AzureStorageAccountAuditFinding::Inconclusive { .. } => "Inconclusive",
    }
}

fn container_count(finding: &AzureStorageAccountAuditFinding) -> Option<usize> {
    match finding {
        AzureStorageAccountAuditFinding::UnusedAnonymousAccess { containers, .. } => {
            Some(containers.len())
        }
        AzureStorageAccountAuditFinding::Inconclusive { .. } => None,
    }
}

fn audit_link(
    ui: &mut Ui,
    label: &str,
    identifier: String,
    target: impl FnOnce() -> PortalTabTarget,
) -> Option<PortalTabOpen> {
    ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
    let link = ui
        .link(escape_display(label))
        .on_hover_text(escape_display(&identifier));
    PortalTabOpen::from_response(&link, target)
}
