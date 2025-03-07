use crate::az_cli::AzureCliResponse;
use crate::prelude::AzureManagementGroup;
use crate::prelude::AzureResourceGroup;
use crate::prelude::AzureScope;
use crate::subscriptions::AzureSubscription;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy::utils::hashbrown::HashMap;
use bevy_svg::prelude::Svg;
use cloud_terrastodon_core_azure::prelude::Fake;
use cloud_terrastodon_core_azure::prelude::ManagementGroupId;
use cloud_terrastodon_core_azure::prelude::ResourceGroupId;
use cloud_terrastodon_core_azure::prelude::RoleDefinition;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use cloud_terrastodon_core_azure::prelude::ThinRoleAssignment;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_graph_nodes_derive::derive_graph_node_icon_data;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::spawn_graph_node;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::GraphNodeIconData;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::IconHandle;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::SpawnGraphNodeEvent;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_follower_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_leader_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;

pub struct RoleAssignmentsPlugin;
impl Plugin for RoleAssignmentsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building RoleAssignmentsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureRoleAssignment>();
        app.register_type::<RoleAssignmentIconData>();
        app.init_resource::<RoleAssignmentIconData>();
        app.observe(join_on_follower_added(
            |ra: &AzureRoleAssignment, rg: &AzureResourceGroup| {
                ra.role_assignment.scope.expanded_form() == rg.id.expanded_form()
            },
        ));
        app.observe(join_on_follower_added(
            |ra: &AzureRoleAssignment, sub: &AzureSubscription| {
                ra.role_assignment.scope.expanded_form() == sub.id.expanded_form()
            },
        ));
        app.observe(join_on_follower_added(
            |ra: &AzureRoleAssignment, mg: &AzureManagementGroup| {
                ra.role_assignment.scope.expanded_form() == mg.id.expanded_form()
            },
        ));
        app.observe(join_on_leader_added(
            |rg: &AzureResourceGroup, ra: &AzureRoleAssignment| {
                ra.role_assignment.scope.expanded_form() == rg.id.expanded_form()
            },
        ));
        app.observe(join_on_leader_added(
            |sub: &AzureSubscription, ra: &AzureRoleAssignment| {
                ra.role_assignment.scope.expanded_form() == sub.id.expanded_form()
            },
        ));
        app.observe(join_on_leader_added(
            |mg: &AzureManagementGroup, ra: &AzureRoleAssignment| {
                ra.role_assignment.scope.expanded_form() == mg.id.expanded_form()
            },
        ));
    }
}

#[derive_graph_node_icon_data]
struct RoleAssignmentIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: IconHandle,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

#[derive(Debug, Reflect, Component)]
#[reflect(Default)]
pub struct AzureRoleAssignment {
    #[reflect(ignore)]
    pub role_assignment: ThinRoleAssignment,
    #[reflect(ignore)]
    pub role_definition: RoleDefinition,
}
impl Default for AzureRoleAssignment {
    fn default() -> Self {
        AzureRoleAssignment {
            role_assignment: Fake::fake(),
            role_definition: Fake::fake(),
        }
    }
}

fn setup(
    mut handles: ResMut<RoleAssignmentIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up resource group icon data lllll");
    handles.circle_icon = asset_server
        .load::<Svg>("textures/azure/RoleAssignments.svg")
        .into();
    handles.circle_material = materials.add(Color::from(BLACK));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 32.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 50.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<RoleAssignmentIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListAzureRoleAssignments {
            role_assignments,
            role_definitions,
        } = msg
        else {
            continue;
        };
        let role_definitions = role_definitions
            .iter()
            .map(|x| (&x.id, x))
            .collect::<HashMap<_, _>>();
        debug!("Received {} role assignments", role_assignments.len());
        for (i, role_assignment) in role_assignments.iter().enumerate() {
            // only display role assignments for ManagementGroups, Subscriptions, and ResourceGroups
            let should_show = ResourceGroupId::try_from_expanded(role_assignment.scope.expanded_form()).is_ok()
                || SubscriptionId::try_from_expanded(role_assignment.scope.expanded_form()).is_ok()
                || ManagementGroupId::try_from_expanded(role_assignment.scope.expanded_form()).is_ok();
            if !should_show {
                continue;
            }
            let Some(role_definition) = role_definitions.get(&role_assignment.role_definition_id)
            else {
                warn!("No role definition found for {role_assignment:?}");
                continue;
            };
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!(
                        "Role Assignment - {}",
                        role_assignment.id.short_form()
                    )),
                    text: role_definition.display_name.to_owned(),
                    translation: Vec3::new(0., i as f32 * 150., 0.),
                    top_extras: (
                        AzureRoleAssignment {
                            role_assignment: role_assignment.to_owned(),
                            role_definition: (**role_definition).to_owned(),
                        },
                        AzureScope {
                            scope: role_assignment.id.as_scope(),
                        },
                        OrganizableSecondary,
                        CollisionLayers::new(PhysLayer::Node, PhysLayer::Cursor),
                    ),
                    text_extras: (OnlyShowWhenHovered,),
                    circle_extras: (),
                    icon_extras: (),
                },
                icon_data.as_ref(),
                &mut commands,
            );
        }
    }
}
