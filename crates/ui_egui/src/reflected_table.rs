use crate::inventory_table::show_inventory_table;
use crate::portal_tab_target::PortalTabTarget;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_table_group::ReflectedTableGroup;
use crate::reflected_table_group_heading::ReflectedTableGroupHeading;
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
        self.show_rows(ui, rows, row_id, checked_rows, None)
    }

    /// Render an expandable heading and its matching rows for each group.
    pub fn show_grouped<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
        group: impl Fn(&T) -> ReflectedTableGroup,
    ) -> Option<PortalTabTarget> {
        self.show_rows(ui, rows, row_id, checked_rows, Some(&group))
    }

    fn show_rows<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
        group: Option<&dyn Fn(&T) -> ReflectedTableGroup>,
    ) -> Option<PortalTabTarget> {
        if self.columns.is_empty() {
            return None;
        }
        let mut rows: Vec<_> = rows.into_iter().map(|row| (row_id(row), row)).collect();
        let table_id = ui.id().with(self.id);
        let headings =
            group.map_or_else(Vec::new, |group| group_rows(ui, table_id, &mut rows, group));
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
                        let field = self.columns[sort.column].field;
                        if headings.is_empty() {
                            sort_rows(&mut rows, field, sort.descending);
                        } else {
                            for heading in &headings {
                                sort_rows(
                                    &mut rows[heading.visible_rows.clone()],
                                    field,
                                    sort.descending,
                                );
                            }
                        }
                        let background = TableRowBackground::new(&mut body);
                        body.rows(29.0, rows.len() + headings.len(), |mut row| {
                            let display_index = row.index();
                            let headings_before = headings
                                .partition_point(|heading| heading.display_index < display_index);
                            if let Some(heading) = headings.get(headings_before)
                                && heading.display_index == display_index
                            {
                                heading.show(&mut row, &background, table_id);
                                return;
                            }
                            let index = display_index - headings_before;
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

fn group_rows<T>(
    ui: &Ui,
    table_id: Id,
    rows: &mut Vec<(Id, &T)>,
    group_for: &dyn Fn(&T) -> ReflectedTableGroup,
) -> Vec<ReflectedTableGroupHeading> {
    let mut groups = HashMap::new();
    for (id, value) in rows.drain(..) {
        let group = group_for(value);
        let (_, items) = groups
            .entry(group.id)
            .or_insert_with(|| (group, Vec::new()));
        items.push((id, value));
    }
    let mut groups: Vec<_> = groups.into_values().collect();
    groups.sort_by_cached_key(|(group, _)| (group.label.to_lowercase(), group.id.value()));
    let mut headings = Vec::with_capacity(groups.len());
    for (group, items) in groups {
        let start = rows.len();
        let count = items.len();
        let id = ReflectedTableGroupHeading::expansion_id(table_id, group.id);
        let expanded = ui.data(|data| data.get_temp::<bool>(id)).unwrap_or(true);
        if expanded {
            rows.extend(items);
        }
        headings.push(ReflectedTableGroupHeading {
            group,
            display_index: start + headings.len(),
            visible_rows: start..rows.len(),
            count,
            expanded,
        });
    }
    headings
}

fn sort_rows<T: Facet<'static>>(rows: &mut [(Id, &T)], field: &str, descending: bool) {
    rows.sort_by_cached_key(|(id, value)| {
        let key = ReflectedValue::new(*value)
            .column(field)
            .map(ReflectedValue::table_sort_key);
        (key, id.value())
    });
    if descending {
        rows.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;

    #[test]
    fn collapsing_a_group_preserves_its_heading_and_excludes_its_rows_from_selection() {
        let mut alpha_ids = [Id::new("alpha tenant one"), Id::new("alpha tenant two")];
        alpha_ids.sort_by_key(|id| id.value());
        let [collapsed, expanded] = alpha_ids;
        let zulu = Id::new("zulu");
        let source = [
            (zulu, "Zulu", "four"),
            (expanded, "Alpha", "three"),
            (collapsed, "Alpha", "one"),
            (collapsed, "Alpha", "two"),
        ];
        let mut rows: Vec<_> = source.iter().map(|row| (Id::new(row.2), row)).collect();
        let table_id = Id::new("test-grouping");
        let mut frame = Context::default().run_ui(Default::default(), |ui| {
            ui.data_mut(|data| {
                data.insert_temp(
                    ReflectedTableGroupHeading::expansion_id(table_id, collapsed),
                    false,
                );
            });
            let headings = group_rows(ui, table_id, &mut rows, &|row| ReflectedTableGroup {
                id: row.0,
                label: row.1.to_owned(),
            });
            assert_eq!(
                headings
                    .iter()
                    .map(|heading| (heading.group.id, heading.display_index, heading.count))
                    .collect::<Vec<_>>(),
                [(collapsed, 0, 2), (expanded, 1, 1), (zulu, 3, 1)]
            );
            assert_eq!(
                rows.iter().map(|(_, row)| row.2).collect::<Vec<_>>(),
                ["three", "four"]
            );
        });
        frame.textures_delta.clear();
    }
}
