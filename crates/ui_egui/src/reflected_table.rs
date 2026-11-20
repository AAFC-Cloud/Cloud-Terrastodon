use crate::inventory_table::show_inventory_table;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_table_sort::ReflectedTableSort;
use crate::reflected_value::ReflectedValue;
use crate::table_row_background::TableRowBackground;
use eframe::egui::AsId;
use eframe::egui::Button;
use eframe::egui::Checkbox;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_extras::Column;
use facet::Facet;
use std::collections::HashMap;

/// A selectable table whose columns look up display values and typed links with Facet.
pub(crate) struct ReflectedTable<'a> {
    id: Id,
    columns: &'a [ReflectedTableColumn],
    initial_sort: ReflectedTableSort,
}

impl<'a> ReflectedTable<'a> {
    pub fn new(id: impl AsId, columns: &'a [ReflectedTableColumn]) -> Self {
        Self {
            id: Id::new(id),
            columns,
            initial_sort: ReflectedTableSort::INITIAL,
        }
    }

    pub fn initially_descending(mut self) -> Self {
        self.initial_sort.descending = true;
        self
    }

    pub fn reset_selection_anchor(&self, ui: &Ui) {
        ui.data_mut(|data| data.remove::<Id>(ui.id().with(self.id).with("anchor")));
    }

    pub fn show<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
    ) -> Option<PortalTabTarget> {
        if self.columns.is_empty() {
            return None;
        }
        let mut rows: Vec<_> = rows.into_iter().map(|row| (row_id(row), row)).collect();
        let table_id = ui.id().with(self.id);
        let sort_id = table_id.with("sort");
        let anchor_id = table_id.with("anchor");
        let mut sort = ui
            .data(|data| data.get_temp::<ReflectedTableSort>(sort_id))
            .filter(|sort| sort.column < self.columns.len())
            .unwrap_or(self.initial_sort);
        let min_width = 40.0
            + self
                .columns
                .iter()
                .map(ReflectedTableColumn::initial_width)
                .sum::<f32>();
        let height = ui.available_height();
        let modifiers = ui.input(|input| input.modifiers);
        let mut navigation = None;
        let mut selection = None;
        ScrollArea::horizontal().id_salt(self.id).show(ui, |ui| {
            ui.set_min_width(min_width);
            show_inventory_table(ui, |table| {
                let mut table = table
                    .id_salt(self.id)
                    .min_scrolled_height(0.0)
                    .max_scroll_height((height - 16.0).max(0.0))
                    .column(Column::exact(32.0));
                for column in self.columns {
                    table = table.column(column.layout());
                }
                table
                    .header(28.0, |mut header| {
                        header.col(|ui| {
                            let selected = rows
                                .iter()
                                .filter(|(id, _)| checked_rows.get(id) == Some(&true))
                                .count();
                            let mut all = !rows.is_empty() && selected == rows.len();
                            let indeterminate = selected > 0 && !all;
                            if ui
                                .add(Checkbox::without_text(&mut all).indeterminate(indeterminate))
                                .on_hover_text("Select visible rows")
                                .changed()
                            {
                                for (id, _) in &rows {
                                    checked_rows.insert(*id, all);
                                }
                                ui.data_mut(|data| data.remove::<Id>(anchor_id));
                            }
                        });
                        for (index, column) in self.columns.iter().enumerate() {
                            header.col(|ui| {
                                let title = format!("{} {}", column.label, sort.indicator(index));
                                if ui.add(Button::new(title).frame(false).truncate()).clicked() {
                                    sort.select(index);
                                    ui.data_mut(|data| data.insert_temp(sort_id, sort));
                                }
                            });
                        }
                    })
                    .body(|mut body| {
                        let key = self.columns[sort.column].field;
                        rows.sort_by_cached_key(|(id, value)| {
                            let key = ReflectedValue::new(*value)
                                .column(key)
                                .map(ReflectedValue::table_sort_key);
                            (key, id.value())
                        });
                        if sort.descending {
                            rows.reverse();
                        }
                        let background = TableRowBackground::new(&mut body);
                        body.rows(29.0, rows.len(), |mut row| {
                            let index = row.index();
                            let (id, value) = rows[index];
                            let reflected = ReflectedValue::new(value);
                            let mut checked = checked_rows.get(&id) == Some(&true);
                            background.first_cell(&mut row, |ui| {
                                if ui
                                    .add(Checkbox::without_text(&mut checked))
                                    .on_hover_text("Select row")
                                    .changed()
                                {
                                    selection = Some((index, checked));
                                }
                                checked
                            });
                            for column in self.columns {
                                row.col(|ui| {
                                    if let Some(icon) = column.icon.and_then(|icon| icon(reflected))
                                    {
                                        ui.add(
                                            Image::new(icon).fit_to_exact_size(Vec2::splat(16.0)),
                                        );
                                    }
                                    let cell = reflected.column(column.field);
                                    let text = cell
                                        .and_then(ReflectedValue::table_text)
                                        .unwrap_or_else(|| "—".to_owned());
                                    let link =
                                        reflected.column(column.link_field.unwrap_or(column.field));
                                    let response = if let Some(target) =
                                        link.and_then(ReflectedValue::navigation_target)
                                    {
                                        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                                        let response = ui.link(&text);
                                        if response.clicked() {
                                            if modifiers.shift {
                                                selection = Some((index, true));
                                            } else if modifiers.command || modifiers.ctrl {
                                                selection = Some((index, !checked));
                                            } else {
                                                navigation = Some(target);
                                            }
                                        }
                                        response
                                    } else {
                                        ui.add(Label::new(&text).truncate())
                                    };
                                    if let Some(identity) = column
                                        .link_field
                                        .and_then(|_| link.and_then(ReflectedValue::table_text))
                                    {
                                        response.on_hover_text(&identity).context_menu(|ui| {
                                            if ui.button("Copy identifier").clicked() {
                                                ui.ctx().copy_text(identity);
                                                ui.close();
                                            }
                                            if ui.button("Copy value").clicked() {
                                                ui.ctx().copy_text(text);
                                                ui.close();
                                            }
                                        });
                                    } else {
                                        response.on_hover_text(text);
                                    }
                                });
                            }
                        });
                    });
            });
        });

        if let Some((index, checked)) = selection {
            let anchor = modifiers
                .shift
                .then(|| ui.data(|data| data.get_temp::<Id>(anchor_id)))
                .flatten()
                .and_then(|id| rows.iter().position(|(other, _)| *other == id));
            let start = anchor.unwrap_or(index);
            for (id, _) in &rows[start.min(index)..=start.max(index)] {
                checked_rows.insert(*id, checked);
            }
            if anchor.is_none() {
                ui.data_mut(|data| data.insert_temp(anchor_id, rows[index].0));
            }
            ui.ctx().request_repaint();
        }
        navigation
    }
}
