use eframe::egui::Id;

/// The distinct identity and display label of a group in a reflected table.
pub(crate) struct ReflectedTableGroup {
    pub id: Id,
    pub label: String,
}
