use crate::app::MyApp;
use crate::portal_resource_scope::PortalResourceScope;
use crate::scope_tags_browser::apply_scope_tags;
use crate::scope_tags_browser::load_scope_tags;
use crate::scope_tags_browser::supports_tags;
use crate::scope_tags_editor::ScopeTagsEditor;
use crate::table_row_background::TableRowBackground;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Layout;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use egui_extras::Column;
use egui_extras::TableBuilder;
use teamy_tracing::gui::DiagnosticReportView;

pub(crate) fn scope_tags_ui(app: &mut MyApp, ui: &mut Ui, scope: &PortalResourceScope) {
    if !supports_tags(scope) {
        ui.label("Tags are unavailable for this scope.");
        return;
    }
    if app
        .azure
        .scope_tags
        .entries
        .get(scope)
        .is_none_or(|editor| matches!(editor.source, Loadable::NotLoaded))
    {
        load_scope_tags(app, scope, false);
    }
    let Some(editor) = app.azure.scope_tags.entries.get_mut(scope) else {
        return;
    };
    match &editor.source {
        Loadable::NotLoaded | Loadable::Loading { .. } => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading tags…");
            });
        }
        Loadable::Failed { error, .. } => {
            ui.label("Azure did not return tags for this resource.");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
        }
        Loadable::Loaded { .. } => {
            if ui.push_id(scope, |ui| show_tag_editor(ui, editor)).inner {
                apply_scope_tags(app, scope);
            }
        }
    }
}

fn show_tag_editor(ui: &mut Ui, editor: &mut ScopeTagsEditor) -> bool {
    let saving = editor.is_loading();
    ui.horizontal_wrapped(|ui| {
        ui.label("Azure stores tags as plain text.");
        ui.hyperlink_to(
            "Learn about Azure tags",
            "https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/tag-resources",
        );
    });
    ui.add_space(8.0);
    let focus_new_row = ui
        .horizontal(|ui| {
            let add = ui
                .add_enabled(
                    !saving && editor.rows.len() < ScopeTagsEditor::MAX_TAGS,
                    Button::new("Add tag"),
                )
                .clicked();
            if add {
                editor.add_row();
            }
            if ui
                .add_enabled(
                    !saving && !editor.rows.is_empty(),
                    Button::new("Remove all tags"),
                )
                .clicked()
            {
                editor.rows.clear();
            }
            add
        })
        .inner;
    let validation = editor.draft_error();
    if let Some(error) = &validation {
        ui.colored_label(ui.visuals().error_fg_color, error);
    }
    if let Loadable::Failed { error, .. } = &editor.write {
        ui.colored_label(
            ui.visuals().error_fg_color,
            "Could not apply tags. Your draft is still available.",
        );
        ui.collapsing("Error details", |ui| {
            DiagnosticReportView::new(error).show(ui);
        });
    }

    let dirty = editor.has_changes();
    let apply = Panel::bottom(ui.make_persistent_id("scope_tags_actions"))
        .exact_size(40.0)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let apply = ui
                    .add_enabled(
                        !saving && dirty && validation.is_none(),
                        Button::new(RichText::new("Apply").color(Color32::WHITE))
                            .fill(Color32::from_rgb(0, 120, 212)),
                    )
                    .clicked();
                let discard = ui
                    .add_enabled(
                        !saving && (dirty || matches!(editor.write, Loadable::Failed { .. })),
                        Button::new("Discard changes"),
                    )
                    .clicked();
                if discard {
                    editor.reset_draft();
                }
                if saving {
                    ui.spinner();
                    ui.label("Applying tags…");
                } else if !dirty && matches!(editor.write, Loadable::Loaded { .. }) {
                    ui.label("Tags updated.");
                }
                apply
            })
            .inner
        })
        .inner;

    if editor.rows.is_empty() {
        ui.add_space(10.0);
        ui.label(if editor.has_changes() {
            "Applying will remove every tag on this resource."
        } else {
            "This resource has no tags."
        });
        return apply;
    }

    let mut remove = None;
    let last = editor.rows.len() - 1;
    let table = TableBuilder::new(ui)
        .id_salt("tag_rows")
        .striped(false)
        .min_scrolled_height(0.0)
        .cell_layout(Layout::left_to_right(Align::Center))
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::exact(64.0));
    let table = if focus_new_row {
        table.scroll_to_row(last, None)
    } else {
        table
    };
    table
        .header(26.0, |mut header| {
            header.col(|ui| {
                ui.strong("Name");
            });
            header.col(|ui| {
                ui.strong("Value");
            });
            header.col(|_| {});
        })
        .body(|mut body| {
            let background = TableRowBackground::new(&mut body);
            body.rows(31.0, editor.rows.len(), |mut row| {
                let index = row.index();
                let draft = &mut editor.rows[index];
                background.first_cell(&mut row, |ui| {
                    let name = ui.add_enabled(
                        !saving,
                        TextEdit::singleline(&mut draft.name)
                            .id_salt(("tag_name", draft.id))
                            .hint_text("Tag name")
                            .desired_width(ui.available_width()),
                    );
                    if focus_new_row && index == last {
                        name.request_focus();
                    }
                    false
                });
                row.col(|ui| {
                    ui.add_enabled(
                        !saving,
                        TextEdit::singleline(&mut draft.value)
                            .id_salt(("tag_value", draft.id))
                            .hint_text("Tag value")
                            .desired_width(ui.available_width()),
                    );
                });
                row.col(|ui| {
                    if ui
                        .add_enabled(!saving, Button::new("Remove").frame_when_inactive(false))
                        .on_hover_text("Remove tag from draft")
                        .clicked()
                    {
                        remove = Some(index);
                    }
                });
            });
        });
    if let Some(index) = remove {
        editor.rows.remove(index);
    }
    apply
}
