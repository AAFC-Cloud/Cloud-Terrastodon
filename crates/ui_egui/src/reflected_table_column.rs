use crate::reflected_value::ReflectedValue;
use eframe::egui::ImageSource;
use egui_extras::Column;

/// The display and navigation fields for a visible Facet table column.
pub(crate) struct ReflectedTableColumn {
    pub label: &'static str,
    pub field: &'static str,
    pub link_field: Option<&'static str>,
    pub icon: Option<fn(ReflectedValue<'_>) -> Option<ImageSource<'static>>>,
    width: Option<f32>,
}

impl ReflectedTableColumn {
    pub const fn new(label: &'static str, field: &'static str) -> Self {
        Self {
            label,
            field,
            link_field: None,
            icon: None,
            width: None,
        }
    }

    pub const fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub const fn with_link(mut self, field: &'static str) -> Self {
        self.link_field = Some(field);
        self
    }

    pub const fn with_icon(
        mut self,
        icon: fn(ReflectedValue<'_>) -> Option<ImageSource<'static>>,
    ) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn layout(&self) -> Column {
        self.width
            .map_or_else(Column::remainder, Column::initial)
            .at_least(96.0)
            .clip(true)
    }

    pub fn initial_width(&self) -> f32 {
        self.width.unwrap_or(170.0)
    }
}
