use azure::prelude::Scope;
use azure::prelude::ScopeImpl;
use azure::prelude::ScopeImplKind;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::utils::HashSet;
use bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl;
use bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive;

use crate::prelude::AzureScope;

pub struct ScopeTrackingPlugin;
impl Plugin for ScopeTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AzureEntities>();
        app.register_type::<AzureEntities>();
        app.register_type_data::<AzureEntities, InspectorEguiImpl>();
        app.observe(on_scope_added);
        app.observe(on_scope_removed);
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AzureEntities {
    #[reflect(ignore)]
    scope_to_entities: HashMap<ScopeImpl, HashSet<Entity>>,
    #[reflect(ignore)]
    entity_to_scopes: HashMap<Entity, HashSet<ScopeImpl>>,
}

impl InspectorPrimitive for AzureEntities {
    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        options: &dyn std::any::Any,
        id: bevy_inspector_egui::egui::Id,
        env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) -> bool {
        self.ui_readonly(ui, options, id, env);
        false
    }

    fn ui_readonly(
        &self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: &dyn std::any::Any,
        _id: bevy_inspector_egui::egui::Id,
        _env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
    ) {
        let mut counts: HashMap<ScopeImplKind, HashSet<&ScopeImpl>> = Default::default();
        for scope in self.scope_to_entities.keys() {
            counts
                .entry(scope.kind())
                .or_insert_with(HashSet::default)
                .insert(scope);
        }
        for (kind, values) in counts {
            ui.collapsing(format!("{:?} ({})", kind, values.len()), |ui| {
                for value in values {
                    ui.label(value.short_form());
                }
            });
        }
    }
}

impl AzureEntities {
    pub fn get_entities_for_scope(&self, scope: &ScopeImpl) -> Option<&HashSet<Entity>> {
        self.scope_to_entities.get(scope)
    }
    pub fn get_scopes_for_entity(&self, entity: Entity) -> Option<&HashSet<ScopeImpl>> {
        self.entity_to_scopes.get(&entity)
    }
    pub fn track_scope_entity(&mut self, scope: &ScopeImpl, entity: Entity) {
        // Update scope_to_entities map
        self.scope_to_entities
            .entry(scope.to_owned())
            .or_insert_with(Default::default)
            .insert(entity);

        // Update entity_to_scopes map
        self.entity_to_scopes
            .entry(entity)
            .or_insert_with(Default::default)
            .insert(scope.to_owned());

        debug!("Tracking {entity:?} with scope {scope:?}");
    }
    pub fn untrack_scope_entity(&mut self, scope: &ScopeImpl, entity: Entity) {
        // Update scope_to_entities map
        if let Some(entities) = self.scope_to_entities.get_mut(scope) {
            entities.retain(|&e| e != entity);
            if entities.is_empty() {
                self.scope_to_entities.remove(scope);
            }
        }

        // Update entity_to_scopes map
        if let Some(scopes) = self.entity_to_scopes.get_mut(&entity) {
            scopes.retain(|s| s != scope);
            if scopes.is_empty() {
                self.entity_to_scopes.remove(&entity);
            }
        }
        debug!("No longer tracking {entity:?} with scope {scope:?}");
    }
}

fn on_scope_added(
    trigger: Trigger<OnAdd, AzureScope>,
    query: Query<&AzureScope>,
    mut azure: ResMut<AzureEntities>,
) {
    let entity = trigger.entity();
    let Ok(scope) = query.get(entity) else {
        warn!("Scope was added but couldn't be found: {entity:?}");
        return;
    };
    azure.track_scope_entity(&scope.scope, entity);
}
fn on_scope_removed(
    trigger: Trigger<OnRemove, AzureScope>,
    query: Query<&AzureScope>,
    mut azure: ResMut<AzureEntities>,
) {
    let entity = trigger.entity();
    let Ok(scope) = query.get(entity) else {
        warn!("Scope was added but couldn't be found: {entity:?}");
        return;
    };
    azure.untrack_scope_entity(&scope.scope, entity);
}
