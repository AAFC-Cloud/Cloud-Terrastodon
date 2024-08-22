
use azure::prelude::ScopeImpl;
use azure::prelude::TestResourceId;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive;

pub struct ScopePlugin;
impl Plugin for ScopePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AzureScope>();
        app.register_type_data::<AzureScope, InspectorEguiImpl>();
    }
}


#[derive(Component, Reflect, Debug)]
#[reflect(Default)]
pub struct AzureScope {
    #[reflect(ignore)]
    pub scope: ScopeImpl,
}
impl Default for AzureScope {
    fn default() -> Self {
        Self { scope: ScopeImpl::TestResource(TestResourceId::new("bruh"))}
    }
}

impl InspectorPrimitive for AzureScope {
    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        options: &dyn std::any::Any,
        id: bevy_inspector_egui::egui::Id,
        env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        todo!()
    }

    fn ui_readonly(
        &self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        options: &dyn std::any::Any,
        id: bevy_inspector_egui::egui::Id,
        env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        todo!()
    }
}