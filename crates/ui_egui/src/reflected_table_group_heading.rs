use crate::reflected_table_group::ReflectedTableGroup;
use crate::table_row_background::TableRowBackground;
use eframe::egui::Button;
use eframe::egui::Id;
use eframe::egui::RichText;
use eframe::egui::Ui;
use egui_extras::TableRow;
use std::ops::Range;

/// The header row and visible descendants for one group in a reflected table.
pub(crate) struct ReflectedTableGroupHeading {
    pub group: ReflectedTableGroup,
    pub display_index: usize,
    pub visible_rows: Range<usize>,
    pub count: usize,
    pub expanded: bool,
}

impl ReflectedTableGroupHeading {
    pub fn expansion_id(table_id: Id, group_id: Id) -> Id {
        table_id.with(("group-expansion", group_id))
    }

    pub fn show(
        &self,
        row: &mut TableRow<'_, '_>,
        background: &TableRowBackground,
        table_id: Id,
        selection_enabled: bool,
    ) -> bool {
        let mut toggled = false;
        if selection_enabled {
            background.first_cell(row, |_| false);
            row.col(|ui| toggled = self.show_toggle(ui, table_id));
        } else {
            background.first_cell(row, |ui| {
                toggled = self.show_toggle(ui, table_id);
                false
            });
        }
        toggled
    }

    fn show_toggle(&self, ui: &mut Ui, table_id: Id) -> bool {
        let (arrow, action) = if self.expanded {
            ("▾", "Collapse")
        } else {
            ("▸", "Expand")
        };
        let label = format!("{} ({})", self.group.label, self.count);
        let button = Button::new(RichText::new(format!("{arrow} {label}")).strong())
            .frame(false)
            .truncate();
        let clicked = ui
            .add(button)
            .on_hover_text(format!("{action} {label}"))
            .clicked();
        if clicked {
            let id = Self::expansion_id(table_id, self.group.id);
            ui.data_mut(|data| data.insert_temp(id, !self.expanded));
            ui.ctx().request_repaint();
        }
        clicked
    }
}
