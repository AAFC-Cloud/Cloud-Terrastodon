use crate::portal_tab_group_id::PortalTabGroupId;
use eframe::egui::Color32;

/// Presentation shared by a contiguous set of tabs in the tab rail.
pub(crate) struct PortalTabGroup {
    pub id: PortalTabGroupId,
    pub name: String,
    pub color: Color32,
    pub collapsed: bool,
}

impl PortalTabGroup {
    pub const COLORS: [Color32; 8] = [
        Color32::from_rgb(21, 96, 175),
        Color32::from_rgb(118, 59, 168),
        Color32::from_rgb(147, 53, 107),
        Color32::from_rgb(167, 56, 47),
        Color32::from_rgb(152, 76, 3),
        Color32::from_rgb(129, 94, 9),
        Color32::from_rgb(4, 110, 100),
        Color32::from_rgb(71, 95, 112),
    ];

    pub fn new(id: PortalTabGroupId) -> Self {
        Self {
            id,
            name: format!("Group {}", id.0 + 1),
            color: Self::COLORS[(id.0 % Self::COLORS.len() as u64) as usize],
            collapsed: false,
        }
    }

    pub fn label(&self) -> &str {
        if self.name.trim().is_empty() {
            "Tab group"
        } else {
            &self.name
        }
    }
}
