use crate::reflected_table_group_heading::ReflectedTableGroupHeading;
use crate::reflected_table_sort::ReflectedTableSort;
use eframe::egui::Id;

/// Visible row indices and group headings for a stable source inventory.
pub(crate) struct ReflectedTableRows {
    pub sort: ReflectedTableSort,
    pub indices: Vec<(Id, usize)>,
    pub headings: Vec<ReflectedTableGroupHeading>,
}
