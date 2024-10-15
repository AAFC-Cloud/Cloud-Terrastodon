use avian2d::prelude::Physics;
use avian2d::prelude::PhysicsTime;
pub use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

pub struct PhysicsPausePlugin;

impl Plugin for PhysicsPausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_pause);
        app.add_systems(Startup, spawn_pause_keybind);
        app.add_plugins(InputManagerPlugin::<PauseAction>::default());
        app.init_state::<AppState>();
        app.add_systems(
            OnEnter(AppState::Paused),
            |mut time: ResMut<Time<Physics>>| time.pause(),
        );
        app.add_systems(
            OnExit(AppState::Paused),
            |mut time: ResMut<Time<Physics>>| time.unpause(),
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    Paused,
    #[default]
    Running,
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PauseAction {
    TogglePause,
}
impl Actionlike for PauseAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PauseAction::TogglePause => InputControlKind::Button,
        }
    }
}

fn spawn_pause_keybind(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause keybind"),
        InputManagerBundle::with_map(
            InputMap::default().with(PauseAction::TogglePause, KeyCode::KeyP),
        ),
    ));
}

fn toggle_pause(
    current_state: ResMut<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    actions: Query<&ActionState<PauseAction>>,
) {
    if actions
        .iter()
        .any(|a| a.just_pressed(&PauseAction::TogglePause))
    {
        let new_state = match current_state.get() {
            AppState::Paused => AppState::Running,
            AppState::Running => AppState::Paused,
        };
        next_state.set(new_state);
    }
}
