use bevy::input::mouse::MouseButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct CameraDragPanMovementPlugin;
impl Plugin for CameraDragPanMovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Anchor>();
        app.add_systems(Update, mouse_drag_update);
    }
}

#[derive(Reflect)]
struct Anchor {
    drag_start_world_position: Vec2,
}

#[derive(Resource, Reflect, Default)]
struct MouseDragState {
    anchor: Option<Anchor>,
    is_dragging: bool,
}

#[allow(clippy::type_complexity)]
fn mouse_drag_update(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_drag_state: ResMut<MouseDragState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut Transform)>,
) {
    let (camera, camera_global_transform, mut camera_transform) = camera_query.single();
    let window = window_query.single();

    // drag start and end logic
    for event in mouse_button_input_events.read() {
        if event.button == MouseButton::Left {
            mouse_drag_state.is_dragging = event.state.is_pressed();
            if mouse_drag_state.is_dragging {
                // begin dragging
                if let Some(screen_position) = window.cursor_position() {
                    if let Some(world_position) = camera
                        .viewport_to_world(camera_global_transform, screen_position)
                        .map(|ray| ray.origin.truncate())
                    {
                        mouse_drag_state.anchor = Some(Anchor {
                            drag_start_world_position: world_position,
                        });
                    }
                }
            } else {
                // finish dragging
                mouse_drag_state.anchor = None;
            }
        }
    }

    if mouse_drag_state.is_dragging {
        // perform drag update
        if let Some(anchor) = &mouse_drag_state.anchor {
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
                    mouse_drag_state.anchor = Some(Anchor {
                        drag_start_world_position: anchor.drag_start_world_position,
                    });
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
    }
}
