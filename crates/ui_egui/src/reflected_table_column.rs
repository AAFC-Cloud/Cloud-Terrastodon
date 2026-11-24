use crate::reflected_value::ReflectedValue;
use eframe::egui::ImageSource;
use eframe::egui::Ui;
use egui_extras::Column;

/// The display and navigation fields for a visible Facet table column.
pub(crate) struct ReflectedTableColumn {
    pub label: &'static str,
    pub field: &'static str,
    pub link_field: Option<&'static str>,
    pub identifier_field: Option<&'static str>,
    pub activates_row: bool,
    pub secondary_text: Option<&'static str>,
    pub can_group: bool,
    icon: Option<fn(&Ui, ReflectedValue<'_>) -> Option<ImageSource<'static>>>,
    width: Option<f32>,
}

impl ReflectedTableColumn {
    pub const fn new(label: &'static str, field: &'static str) -> Self {
        Self {
            label,
            field,
            link_field: None,
            identifier_field: None,
            activates_row: false,
            secondary_text: None,
            can_group: false,
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

    /// Show and copy a separate identifier without changing cell navigation.
    pub const fn with_identifier(mut self, field: &'static str) -> Self {
        self.identifier_field = Some(field);
        self
    }

    /// Activate the row through the caller's local UI action.
    pub const fn with_row_link(mut self) -> Self {
        self.activates_row = true;
        self
    }

    pub const fn with_icon(
        mut self,
        icon: fn(&Ui, ReflectedValue<'_>) -> Option<ImageSource<'static>>,
    ) -> Self {
        self.icon = Some(icon);
        self
    }

    pub const fn with_secondary_text(mut self, field: &'static str) -> Self {
        self.secondary_text = Some(field);
        self
    }

    /// Offer this field in the table's Group by menu.
    /// A linked field supplies the group identity when one is available.
    pub const fn groupable(mut self) -> Self {
        self.can_group = true;
        self
    }

    pub fn icon(&self, ui: &Ui, row: ReflectedValue<'_>) -> Option<ImageSource<'static>> {
        self.icon?(ui, row)
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
