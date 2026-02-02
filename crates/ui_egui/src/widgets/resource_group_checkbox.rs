use crate::app::MyApp;
use crate::icons::RESOURCE_GROUP_ICON;
use cloud_terrastodon_azure::prelude::ResourceGroup;
use eframe::egui::Ui;
use tracing::debug;

pub fn draw_resource_group_checkbox(
    app: &mut MyApp,
    ui: &mut Ui,
    resource_group: &ResourceGroup,
) {
    ui.horizontal(|ui| {
        let checked = app.checkbox_for(&resource_group.id);
        if ui.image(RESOURCE_GROUP_ICON).clicked() {
            debug!("Clicked on resource_group icon");
            *checked ^= true;
        }

        ui.checkbox(checked, resource_group.name.as_str());
    });
}
