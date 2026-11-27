use crate::inventory_table::show_inventory_footer;
use crate::inventory_table::show_inventory_table;
use crate::portal_filter_chip::portal_filter_chip;
use crate::portal_search::portal_search_with_id;
use crate::portal_tab_open::PortalTabOpen;
use crate::reflected_table_column::ReflectedTableColumn;
use crate::reflected_table_group::ReflectedTableGroup;
use crate::reflected_table_group_heading::ReflectedTableGroupHeading;
use crate::reflected_table_rows::ReflectedTableRows;
use crate::reflected_table_sort::ReflectedTableSort;
use crate::reflected_value::ReflectedValue;
use crate::table_row_background::TableRowBackground;
use eframe::egui::AsId;
use eframe::egui::Checkbox;
use eframe::egui::CornerRadius;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::ImageSource;
use eframe::egui::Label;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_extras::Column;
use facet::Facet;
use std::collections::HashMap;

/// A table whose columns look up display values and typed links with Facet.
pub(crate) struct ReflectedTable<'a> {
    id: Id,
    columns: &'a [ReflectedTableColumn],
    initial_sort: ReflectedTableSort,
    selection_enabled: bool,
    result_count: bool,
    status: Option<&'a str>,
    search_hint: &'static str,
}

impl<'a> ReflectedTable<'a> {
    pub fn new(id: impl AsId, columns: &'a [ReflectedTableColumn]) -> Self {
        Self {
            id: Id::new(id),
            columns,
            initial_sort: ReflectedTableSort::INITIAL,
            selection_enabled: true,
            result_count: false,
            status: None,
            search_hint: "Search results",
        }
    }

    pub fn initially_descending(mut self) -> Self {
        self.initial_sort.descending = true;
        self
    }

    /// Hide the checkboxes and disable selection gestures. Existing checked rows still shade.
    pub fn without_selection(mut self) -> Self {
        self.selection_enabled = false;
        self
    }

    /// Show the filtered count and the total supplied to the explorable table.
    pub fn with_result_count(mut self) -> Self {
        self.result_count = true;
        self
    }

    /// Reserve the result footer and show the current loading or lookup progress there.
    pub fn with_status(mut self, status: Option<&'a str>) -> Self {
        self.result_count = true;
        self.status = status;
        self
    }

    pub fn with_search_hint(mut self, hint: &'static str) -> Self {
        self.search_hint = hint;
        self
    }

    /// Read the table's current text filter for an optional upstream lookup.
    pub fn search_text(&self, ui: &Ui) -> String {
        let search_id = ui.id().with(self.id).with("search");
        ui.data(|data| data.get_temp::<String>(search_id).unwrap_or_default())
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
    ) -> Option<PortalTabOpen> {
        let rows = rows.into_iter().map(|row| (row_id(row), row)).collect();
        self.show_rows(ui, rows, checked_rows, None, None)
    }

    /// Render an expandable heading and its matching rows for each group.
    pub fn show_grouped<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
        group: impl Fn(&T) -> ReflectedTableGroup,
    ) -> Option<PortalTabOpen> {
        let rows = rows.into_iter().map(|row| (row_id(row), row)).collect();
        self.show_rows(ui, rows, checked_rows, Some(&group), None)
    }

    /// Reuse the row order while the source values and grouping remain stable.
    /// Callers clear the cached order when either input changes.
    pub fn show_cached<T: Facet<'static>>(
        &self,
        ui: &mut Ui,
        values: &[T],
        checked_rows: &mut HashMap<Id, bool>,
        order: &mut Option<ReflectedTableRows>,
        row_id: impl Fn(&T) -> Id,
        group: Option<&dyn Fn(&T) -> ReflectedTableGroup>,
    ) -> Option<PortalTabOpen> {
        if self.columns.is_empty() {
            return None;
        }
        let sort = self.sort(ui);
        if order.as_ref().is_some_and(|order| order.sort != sort) {
            *order = None;
        }
        let prepared = order.get_or_insert_with(|| {
            let mut rows: Vec<_> = values
                .iter()
                .enumerate()
                .map(|(index, value)| (row_id(value), index))
                .collect();
            let headings = self.prepare_rows(ui, &mut rows, sort, group, |index| &values[*index]);
            ReflectedTableRows {
                sort,
                indices: rows,
                headings,
            }
        });
        let (navigation, group_toggled) = self.draw_rows(
            ui,
            prepared.indices.len(),
            &|index| {
                let (id, index) = prepared.indices[index];
                (id, &values[index])
            },
            &prepared.headings,
            checked_rows,
            None,
        );
        if group_toggled {
            *order = None;
        }
        navigation
    }

    /// Explore an already loaded set of rows using Facet-backed search and grouping.
    pub fn show_explorable<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
    ) -> Option<PortalTabOpen> {
        self.show_explorable_rows(ui, rows, row_id, checked_rows, |_, _| false, None)
    }

    pub fn show_explorable_with_activation<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
        mut activate: impl FnMut(&T),
    ) -> Option<PortalTabOpen> {
        self.show_explorable_rows(
            ui,
            rows,
            row_id,
            checked_rows,
            |_, _| false,
            Some(&mut activate),
        )
    }

    pub fn show_explorable_matching<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
        extra_match: impl Fn(&T, &str) -> bool,
    ) -> Option<PortalTabOpen> {
        self.show_explorable_rows(ui, rows, row_id, checked_rows, extra_match, None)
    }

    fn show_explorable_rows<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        rows: impl IntoIterator<Item = &'row T>,
        row_id: impl Fn(&T) -> Id,
        checked_rows: &mut HashMap<Id, bool>,
        extra_match: impl Fn(&T, &str) -> bool,
        activate: Option<&mut dyn FnMut(&T)>,
    ) -> Option<PortalTabOpen> {
        let table_id = ui.id().with(self.id);
        let search_id = table_id.with("search");
        let grouping_id = table_id.with("grouping");
        let mut search = self.search_text(ui);
        let previous_grouping = ui.data(|data| data.get_temp::<&'static str>(grouping_id));
        let mut grouping = previous_grouping.filter(|field| {
            self.columns
                .iter()
                .any(|column| column.field == *field && column.can_group)
        });
        let search_changed = ui
            .horizontal_wrapped(|ui| {
                let changed = portal_search_with_id(
                    ui,
                    search_id.with("input"),
                    &mut search,
                    self.search_hint,
                    270.0,
                )
                .changed();
                if self.columns.iter().any(|column| column.can_group) {
                    let label = grouping
                        .and_then(|field| self.columns.iter().find(|column| column.field == field))
                        .map_or("None", |column| column.label);
                    ui.push_id(grouping_id, |ui| {
                        portal_filter_chip(ui, format!("Group by · {label}"), |ui| {
                            if ui.selectable_value(&mut grouping, None, "None").clicked() {
                                ui.close();
                            }
                            for column in self.columns.iter().filter(|column| column.can_group) {
                                if ui
                                    .selectable_value(
                                        &mut grouping,
                                        Some(column.field),
                                        column.label,
                                    )
                                    .clicked()
                                {
                                    ui.close();
                                }
                            }
                        });
                    });
                }
                changed
            })
            .inner;
        if search_changed || grouping != previous_grouping {
            self.reset_selection_anchor(ui);
            ui.data_mut(|data| {
                if search_changed {
                    data.insert_temp(search_id, search.clone());
                }
                if let Some(field) = grouping {
                    data.insert_temp(grouping_id, field);
                } else {
                    data.remove::<&'static str>(grouping_id);
                }
            });
        }
        let search = search.trim().to_lowercase();
        let mut total = 0;
        let rows: Vec<_> = rows
            .into_iter()
            .inspect(|_| total += 1)
            .filter(|row| self.matches_search(*row, &search) || extra_match(*row, &search))
            .map(|row| (row_id(row), row))
            .collect();
        if self.result_count {
            show_inventory_footer(ui, self.id, rows.len(), total, self.status);
        }
        let no_match = !self.result_count && !search.is_empty() && rows.is_empty();
        let navigation = if let Some(column) =
            grouping.and_then(|field| self.columns.iter().find(|column| column.field == field))
        {
            self.show_rows(
                ui,
                rows,
                checked_rows,
                Some(&|row| group_by_column(column, row)),
                activate,
            )
        } else {
            self.show_rows(ui, rows, checked_rows, None, activate)
        };
        if no_match {
            ui.label("No rows match the search.");
        }
        navigation
    }

    /// Match a lowercase substring in a row's visible values and linked identifiers.
    pub fn matches_search(&self, row: &impl Facet<'static>, search: &str) -> bool {
        if search.is_empty() {
            return true;
        }
        let row = ReflectedValue::new(row);
        self.columns.iter().any(|column| {
            std::iter::once(column.field)
                .chain(column.secondary_text)
                .chain(column.link_field.filter(|field| *field != column.field))
                .chain(
                    column
                        .identifier_field
                        .filter(|field| *field != column.field),
                )
                .filter_map(|field| row.column(field).and_then(ReflectedValue::table_text))
                .any(|value| value.to_lowercase().contains(search))
        })
    }

    fn show_rows<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        mut rows: Vec<(Id, &'row T)>,
        checked_rows: &mut HashMap<Id, bool>,
        group: Option<&dyn Fn(&T) -> ReflectedTableGroup>,
        activate: Option<&mut dyn FnMut(&T)>,
    ) -> Option<PortalTabOpen> {
        if self.columns.is_empty() {
            return None;
        }
        let headings = self.prepare_rows(ui, &mut rows, self.sort(ui), group, |row| *row);
        self.draw_rows(
            ui,
            rows.len(),
            &|index| rows[index],
            &headings,
            checked_rows,
            activate,
        )
        .0
    }

    fn sort(&self, ui: &Ui) -> ReflectedTableSort {
        let sort_id = ui.id().with(self.id).with("sort");
        ui.data(|data| data.get_temp::<ReflectedTableSort>(sort_id))
            .filter(|sort| sort.column < self.columns.len())
            .unwrap_or(self.initial_sort)
    }

    fn prepare_rows<'row, T: Facet<'static> + 'row, R>(
        &self,
        ui: &Ui,
        rows: &mut Vec<(Id, R)>,
        sort: ReflectedTableSort,
        group: Option<&dyn Fn(&T) -> ReflectedTableGroup>,
        value: impl Fn(&R) -> &'row T,
    ) -> Vec<ReflectedTableGroupHeading> {
        let table_id = ui.id().with(self.id);
        let headings = group.map_or_else(Vec::new, |group| {
            group_rows(ui, table_id, rows, &|row| group(value(row)))
        });
        let field = self.columns[sort.column].field;
        if headings.is_empty() {
            sort_rows(rows, field, sort.descending, value);
        } else {
            for heading in &headings {
                sort_rows(
                    &mut rows[heading.visible_rows.clone()],
                    field,
                    sort.descending,
                    &value,
                );
            }
        }
        headings
    }

    fn draw_rows<'row, T: Facet<'static> + 'row>(
        &self,
        ui: &mut Ui,
        row_count: usize,
        row_at: &dyn Fn(usize) -> (Id, &'row T),
        headings: &[ReflectedTableGroupHeading],
        checked_rows: &mut HashMap<Id, bool>,
        mut activate: Option<&mut dyn FnMut(&T)>,
    ) -> (Option<PortalTabOpen>, bool) {
        let table_id = ui.id().with(self.id);
        let sort_id = table_id.with("sort");
        let anchor_id = table_id.with("anchor");
        let mut sort = self.sort(ui);
        let min_width = if self.selection_enabled { 40.0 } else { 0.0 }
            + self
                .columns
                .iter()
                .map(ReflectedTableColumn::initial_width)
                .sum::<f32>();
        let height = ui.available_height();
        let row_height = if self
            .columns
            .iter()
            .any(|column| column.secondary_text.is_some())
        {
            40.0
        } else {
            29.0
        };
        let modifiers = ui.input(|input| input.modifiers);
        let mut navigation = None;
        let mut selection = None;
        let mut group_toggled = false;
        ScrollArea::horizontal().id_salt(self.id).show(ui, |ui| {
            ui.set_min_width(min_width);
            show_inventory_table(ui, |table| {
                let mut table = table
                    .id_salt(self.id)
                    .max_scroll_height((height - 16.0).max(0.0));
                if self.selection_enabled {
                    table = table.column(Column::exact(32.0));
                }
                for column in self.columns {
                    table = table.column(column.layout());
                }
                table
                    .header(28.0, |mut header| {
                        if self.selection_enabled {
                            header.col(|ui| {
                                let selected = if checked_rows.is_empty() {
                                    0
                                } else {
                                    (0..row_count)
                                        .filter(|&index| {
                                            checked_rows.get(&row_at(index).0) == Some(&true)
                                        })
                                        .count()
                                };
                                let mut all = row_count != 0 && selected == row_count;
                                let indeterminate = selected > 0 && !all;
                                if ui
                                    .add(
                                        Checkbox::without_text(&mut all)
                                            .indeterminate(indeterminate),
                                    )
                                    .on_hover_text("Select visible rows")
                                    .changed()
                                {
                                    for index in 0..row_count {
                                        checked_rows.insert(row_at(index).0, all);
                                    }
                                    ui.data_mut(|data| data.remove::<Id>(anchor_id));
                                }
                            });
                        }
                        for (index, column) in self.columns.iter().enumerate() {
                            header.col(|ui| {
                                sort.show_header(ui, column.label, index, sort_id);
                            });
                        }
                    })
                    .body(|mut body| {
                        let background = TableRowBackground::new(&mut body);
                        // egui renders only the viewport and its small boundary buffer.
                        body.rows(row_height, row_count + headings.len(), |mut row| {
                            let display_index = row.index();
                            let headings_before = headings
                                .partition_point(|heading| heading.display_index < display_index);
                            if let Some(heading) = headings.get(headings_before)
                                && heading.display_index == display_index
                            {
                                group_toggled |= heading.show(
                                    &mut row,
                                    &background,
                                    table_id,
                                    self.selection_enabled,
                                );
                                return;
                            }
                            let index = display_index - headings_before;
                            let (id, value) = row_at(index);
                            let reflected = ReflectedValue::new(value);
                            let mut checked = checked_rows.get(&id) == Some(&true);
                            if self.selection_enabled {
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
                            }
                            for (column_index, column) in self.columns.iter().enumerate() {
                                let mut show_cell = |ui: &mut Ui| {
                                    if let Some(icon) = column.icon(ui, reflected) {
                                        let size = if column.secondary_text.is_some() {
                                            24.0
                                        } else {
                                            16.0
                                        };
                                        let radius = if matches!(icon, ImageSource::Texture(_)) {
                                            CornerRadius::same(12)
                                        } else {
                                            CornerRadius::ZERO
                                        };
                                        ui.add(
                                            Image::new(icon)
                                                .fit_to_exact_size(Vec2::splat(size))
                                                .corner_radius(radius),
                                        );
                                    }
                                    let cell = reflected.column(column.field);
                                    let text = cell
                                        .and_then(ReflectedValue::table_text)
                                        .unwrap_or_else(|| "—".to_owned());
                                    let link =
                                        reflected.column(column.link_field.unwrap_or(column.field));
                                    let mut show_value = |ui: &mut Ui| {
                                        let target =
                                            link.and_then(ReflectedValue::navigation_target);
                                        if target.is_some()
                                            || (column.activates_row && activate.is_some())
                                        {
                                            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
                                            let response = ui.link(&text);
                                            if target.is_some()
                                                && PortalTabOpen::opens_in_background(&response)
                                            {
                                                navigation = target.map(PortalTabOpen::Background);
                                            } else if response.clicked() {
                                                if self.selection_enabled && modifiers.shift {
                                                    selection = Some((index, true));
                                                } else if self.selection_enabled
                                                    && (modifiers.command || modifiers.ctrl)
                                                {
                                                    selection = Some((index, !checked));
                                                } else if column.activates_row
                                                    && let Some(activate) = activate.as_deref_mut()
                                                {
                                                    activate(value);
                                                } else {
                                                    navigation = target.map(PortalTabOpen::Current);
                                                }
                                            }
                                            response
                                        } else {
                                            ui.add(Label::new(&text).truncate())
                                        }
                                    };
                                    let secondary_text = column
                                        .secondary_text
                                        .and_then(|field| reflected.column(field))
                                        .and_then(ReflectedValue::table_text)
                                        .filter(|secondary| secondary != &text);
                                    let response = if let Some(secondary_text) = secondary_text {
                                        ui.vertical(|ui| {
                                            ui.spacing_mut().item_spacing.y = 0.0;
                                            let response = show_value(ui);
                                            ui.add(
                                                Label::new(
                                                    RichText::new(&secondary_text).weak().small(),
                                                )
                                                .truncate(),
                                            )
                                            .on_hover_text(secondary_text);
                                            response
                                        })
                                        .inner
                                    } else {
                                        show_value(ui)
                                    };
                                    if let Some(identity) = column
                                        .identifier_field
                                        .or(column.link_field)
                                        .and_then(|field| reflected.column(field))
                                        .and_then(ReflectedValue::table_text)
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
                                };
                                if column_index == 0 && !self.selection_enabled {
                                    background.first_cell(&mut row, |ui| {
                                        show_cell(ui);
                                        checked
                                    });
                                } else {
                                    row.col(show_cell);
                                }
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
                .and_then(|id| (0..row_count).find(|&index| row_at(index).0 == id));
            let start = anchor.unwrap_or(index);
            for index in start.min(index)..=start.max(index) {
                checked_rows.insert(row_at(index).0, checked);
            }
            if anchor.is_none() {
                ui.data_mut(|data| data.insert_temp(anchor_id, row_at(index).0));
            }
            ui.ctx().request_repaint();
        }
        (navigation, group_toggled)
    }
}

fn group_by_column(
    column: &ReflectedTableColumn,
    row: &impl Facet<'static>,
) -> ReflectedTableGroup {
    let row = ReflectedValue::new(row);
    let label = row
        .column(column.field)
        .and_then(ReflectedValue::table_text);
    let identity = column
        .link_field
        .and_then(|field| row.column(field))
        .and_then(ReflectedValue::table_text);
    ReflectedTableGroup {
        id: Id::new((column.field, identity.as_deref().or(label.as_deref()))),
        label: label
            .or(identity)
            .unwrap_or_else(|| "Unspecified".to_owned()),
    }
}

fn group_rows<T>(
    ui: &Ui,
    table_id: Id,
    rows: &mut Vec<(Id, T)>,
    group_for: &dyn Fn(&T) -> ReflectedTableGroup,
) -> Vec<ReflectedTableGroupHeading> {
    let mut groups = HashMap::new();
    for (id, value) in rows.drain(..) {
        let group = group_for(&value);
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

fn sort_rows<'row, T: Facet<'static> + 'row, R>(
    rows: &mut [(Id, R)],
    field: &str,
    descending: bool,
    value: impl Fn(&R) -> &'row T,
) {
    rows.sort_by_cached_key(|(id, row)| {
        let key = ReflectedValue::new(value(row))
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
    use crate::portal_tab_target::PortalTabTarget;
    use cloud_terrastodon_azure::AzureResourceGroupId;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::FullOutput;
    use eframe::egui::Modifiers;
    use eframe::egui::PointerButton;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;

    #[derive(Facet)]
    struct Record {
        label: String,
        id: String,
        #[facet(sensitive)]
        credential: String,
    }

    #[test]
    fn search_includes_linked_identifiers_and_respects_facet_sensitivity() {
        let columns = [
            ReflectedTableColumn::new("Name", "label").with_link("id"),
            ReflectedTableColumn::new("Credential", "credential"),
        ];
        let table = ReflectedTable::new("search", &columns);
        let row = Record {
            label: "Sample Resource".into(),
            id: "Synthetic-42".into(),
            credential: "Private Value".into(),
        };
        assert!(table.matches_search(&row, "sample resource"));
        assert!(table.matches_search(&row, "synthetic-42"));
        assert!(!table.matches_search(&row, "private value"));
    }

    #[test]
    fn progress_and_zero_results_keep_search_focus_and_header_position() {
        let context = Context::default();
        let columns = [ReflectedTableColumn::new("Name", "label")];
        let rows = [Record {
            label: "Initial entry".into(),
            id: "initial".into(),
            credential: String::new(),
        }];
        let mut search = String::new();
        let mut draw = |status, events| {
            let mut frame = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 240.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let table = ReflectedTable::new("directory", &columns).with_status(status);
                    table.show_explorable(ui, &rows, |row| Id::new(&row.id), &mut HashMap::new());
                    search = table.search_text(ui);
                },
            );
            frame.textures_delta.clear();
            frame
        };
        let frame = draw(None, vec![]);
        let header_y = |frame: &FullOutput| {
            frame
                .shapes
                .iter()
                .find_map(|clipped| match &clipped.shape {
                    Shape::Text(shape) if shape.galley.text() == "Name ↑" => Some(shape.pos.y),
                    _ => None,
                })
                .unwrap()
        };
        let initial_header_y = header_y(&frame);
        let input = frame
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == "Search results" => {
                    Some(shape.visual_bounding_rect().center())
                }
                _ => None,
            })
            .unwrap();
        for pressed in [true, false] {
            draw(
                None,
                vec![
                    Event::PointerMoved(input),
                    Event::PointerButton {
                        pos: input,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            );
        }
        let focused = context.memory(|memory| memory.focused()).unwrap();
        for (status, letter) in [
            (Some("Loading directory entries"), "z"),
            (Some("Searching Microsoft Graph"), "a"),
            (None, "b"),
        ] {
            let frame = draw(status, vec![Event::Text(letter.to_owned())]);
            assert!(context.memory(|memory| memory.has_focus(focused)));
            assert_eq!(header_y(&frame), initial_header_y);
        }
        assert_eq!(search, "zab");
    }

    #[test]
    fn activating_a_filtered_sorted_row_uses_its_identity() {
        let context = Context::default();
        let columns = [ReflectedTableColumn::new("Name", "label").with_row_link()];
        let table = ReflectedTable::new("local-links", &columns)
            .without_selection()
            .with_result_count();
        let rows = ["Zulu project", "Outside search", "Alpha project"].map(|label| Record {
            label: label.into(),
            id: format!("id for {label}"),
            credential: String::new(),
        });
        let draw = |events, activated: &mut Option<String>| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 240.0))),
                events,
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                let search_id = ui.id().with(table.id).with("search");
                ui.data_mut(|data| data.insert_temp(search_id, "project".to_owned()));
                table.show_explorable_with_activation(
                    ui,
                    &rows,
                    |row| Id::new(&row.id),
                    &mut HashMap::new(),
                    |row| *activated = Some(row.id.clone()),
                );
            });
            output.textures_delta.clear();
            output
        };
        let mut activated = None;
        let frame = draw(vec![], &mut activated);
        let labels: Vec<_> = frame
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Text(shape) => Some((shape.galley.text(), shape.visual_bounding_rect())),
                _ => None,
            })
            .collect();
        assert!(labels.iter().all(|(label, _)| *label != "Outside search"));
        assert!(
            labels
                .iter()
                .any(|(label, _)| *label == "Showing 2 of 3 entries")
        );
        let row_pos = |name| {
            labels
                .iter()
                .find(|(label, _)| *label == name)
                .unwrap()
                .1
                .center()
        };
        let pos = row_pos("Alpha project");
        assert!(pos.y < row_pos("Zulu project").y);

        for pressed in [true, false] {
            draw(
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                &mut activated,
            );
        }
        assert_eq!(activated.as_deref(), Some("id for Alpha project"));
    }

    #[test]
    fn modified_clicking_a_typed_cell_opens_its_destination_in_the_background() -> eyre::Result<()>
    {
        #[derive(Facet)]
        struct ResourceRow {
            name: &'static str,
            id: AzureResourceGroupId,
        }

        let row = ResourceRow {
            name: "Example resource group",
            id: AzureResourceGroupId::try_new(
                "00000000-0000-0000-0000-000000000001",
                "example-group",
            )?,
        };
        let context = Context::default();
        let columns = [ReflectedTableColumn::new("Name", "name").with_link("id")];
        let table = ReflectedTable::new("linked-resources", &columns);
        let mut checked = HashMap::new();
        let mut draw = |events| {
            let mut destination = None;
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 240.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    destination = table.show(ui, [&row], |row| Id::new(&row.id), &mut checked);
                },
            );
            output.textures_delta.clear();
            (output, destination)
        };
        let (frame, _) = draw(vec![]);
        let pos = frame
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == row.name => {
                    Some(shape.visual_bounding_rect().center())
                }
                _ => None,
            })
            .unwrap();
        for (button, modifiers) in [
            (PointerButton::Middle, Modifiers::NONE),
            (PointerButton::Primary, Modifiers::CTRL),
            (PointerButton::Primary, Modifiers::COMMAND),
        ] {
            let click = |pressed| {
                vec![
                    Event::ModifiersChanged(modifiers),
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button,
                        pressed,
                        modifiers,
                    },
                ]
            };
            draw(click(true));
            let (_, destination) = draw(click(false));
            assert_eq!(
                destination,
                Some(PortalTabOpen::Background(PortalTabTarget::ResourceGroup(
                    row.id.clone()
                )))
            );
        }
        assert!(checked.is_empty());
        Ok(())
    }

    #[test]
    fn grouping_keeps_distinct_linked_identities_with_the_same_label() {
        let column = ReflectedTableColumn::new("Name", "label").with_link("id");
        let make_row = |id: &str| Record {
            label: "Sample".into(),
            id: id.into(),
            credential: String::new(),
        };
        let first = group_by_column(&column, &make_row("first"));
        let second = group_by_column(&column, &make_row("second"));
        assert_eq!(first.label, second.label);
        assert_ne!(first.id, second.id);
    }

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
