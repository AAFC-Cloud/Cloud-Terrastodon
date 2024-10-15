


use avian2d::prelude::Collider;
use avian2d::prelude::FixedJoint;
use avian2d::prelude::Joint;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::MassPropertiesBundle;
use avian2d::prelude::RigidBody;
use bevy::prelude::*;
use cloud_terrastodon_visualizer_physics_plugin::prelude::CustomLinearDamping;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::prelude::KeyboardVirtualDPad;
use leafwing_input_manager::prelude::MouseScrollAxis;
use leafwing_input_manager::InputManagerBundle;

use crate::camera_types::CameraAction;
use crate::camera_types::CameraMotion;
use crate::camera_types::JointGizmosCamera;
use crate::camera_types::MyRenderLayers;
use crate::camera_types::PrimaryCamera;

pub struct SpawnCameraPlugin;
impl Plugin for SpawnCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);

    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_axis(CameraAction::Zoom, MouseScrollAxis::Y)
        .with_dual_axis(CameraAction::Pan, KeyboardVirtualDPad::WASD)
        .with(CameraAction::Sprint, KeyCode::ShiftLeft);

    let primary_camera_id = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                projection: OrthographicProjection {
                    scale: 10.0,
                    far: 1000.,
                    near: -1000.,
                    ..default()
                },
                ..default()
            },
            CameraMotion::default(),
            RigidBody::Kinematic,
            LinearVelocity::default(),
            CustomLinearDamping::default(),
            PrimaryCamera,
            MyRenderLayers::Primary.layer(),
            Name::new("Primary Camera"),
        ))
        .insert(InputManagerBundle::with_map(input_map))
        .id();

    let joint_gizmos_camera_id = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 1,
                    ..default()
                },
                projection: OrthographicProjection {
                    scale: 10.0,
                    far: 1000.,
                    near: -1000.,
                    ..default()
                },
                ..default()
            },
            JointGizmosCamera,
            MyRenderLayers::JointGizmos.layer(),
            RigidBody::Dynamic,
            MassPropertiesBundle::new_computed(&Collider::rectangle(1., 1.), 1.0),
            Name::new("Joint Gizmos Camera"),
        ))
        .id();

    commands.spawn(FixedJoint::new(primary_camera_id, joint_gizmos_camera_id));
}

