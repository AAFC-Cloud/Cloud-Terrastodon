use eframe::egui::Button;
use eframe::egui::Id;
use eframe::egui::Ui;

/// The selected sort key and direction of a reflected inventory table.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReflectedTableSort {
    pub column: usize,
    pub descending: bool,
}

impl ReflectedTableSort {
    pub const INITIAL: Self = Self {
        column: 0,
        descending: false,
    };

    pub fn show_header(&mut self, ui: &mut Ui, label: &str, column: usize, sort_id: Id) {
        let title = format!("{label} {}", self.indicator(column));
        if ui.add(Button::new(title).frame(false).truncate()).clicked() {
            self.select(column);
            ui.data_mut(|data| data.insert_temp(sort_id, *self));
            ui.ctx().request_repaint();
        }
    }

    pub fn select(&mut self, column: usize) {
        self.descending = self.column == column && !self.descending;
        self.column = column;
    }

    pub fn indicator(self, column: usize) -> &'static str {
        if self.column != column {
            "↕"
        } else if self.descending {
            "↓"
        } else {
            "↑"
        }
    }
}
