use bevy::input::mouse::MouseButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::prelude::PrimaryCamera;

pub struct CameraDragPanMovementPlugin;
impl Plugin for CameraDragPanMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Anchor>();
        app.init_resource::<MouseDragState>();
        app.add_systems(PreUpdate, mouse_drag_update);
    }
}

#[derive(Reflect)]
struct Anchor {
    drag_start_world_position: Vec2,
}

#[derive(Resource, Reflect, Default)]
enum MouseDragState {
    Dragging {
        anchor: Anchor,
    },
    #[default]
    Idle,
}

#[allow(clippy::type_complexity)]
fn mouse_drag_update(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_drag_state: ResMut<MouseDragState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut Transform), With<PrimaryCamera>>,
) {
    let (camera, camera_global_transform, mut camera_transform) = camera_query.single_mut();
    let window = window_query.single();

    // Begin and end logic
    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Middle {
            // Compute new drag state
            match event.state.is_pressed() {
                true => {
                    if let Some(screen_position) = window.cursor_position() {
                        if let Some(world_position) = camera
                            .viewport_to_world(camera_global_transform, screen_position)
                            .map(|ray| ray.origin.truncate())
                        {
                            *mouse_drag_state = MouseDragState::Dragging {
                                anchor: Anchor {
                                    drag_start_world_position: world_position,
                                },
                            }
                        }
                    }
                }
                false => *mouse_drag_state = MouseDragState::Idle,
            };
        }
    }

    // perform drag update
    let MouseDragState::Dragging { ref mut anchor } = *mouse_drag_state else {
        return;
    };

    if let Some(current_screen_position) = window.cursor_position() {
        // mouse is inside the window, convert to world coords
        if let Some(current_world_position) = camera
            .viewport_to_world(camera_global_transform, current_screen_position)
            .map(|ray| ray.origin.truncate())
        {
            // calculate delta from the anchor
            let delta = anchor.drag_start_world_position - current_world_position;
            camera_transform.translation += delta.extend(0.0);

            // track info needed to keep alignment with starting point
            *anchor = Anchor {
                drag_start_world_position: anchor.drag_start_world_position,
            };
        }
    } else {
        // cursor is outside the window, use delta to approximate mouse position
        let mut delta = mouse_motion_events
            .read()
            .fold(Vec2::ZERO, |acc, event| acc + event.delta);
        delta.x *= -1.0;
        camera_transform.translation += delta.extend(0.0);
    }
}
