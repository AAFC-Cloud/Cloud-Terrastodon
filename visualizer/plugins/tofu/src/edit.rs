use bevy::prelude::*;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::Hovered;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputControlKind;
use leafwing_input_manager::InputManagerBundle;

use crate::import_blocks_plugin::TofuImportBlock;
use crate::tofu_worker_plugin::TofuEvent;

pub struct EditPlugin;

impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<EditAction>::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, handle_edit_action);
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug, Reflect)]
pub enum EditAction {
    JumpToDefinition,
}
impl Actionlike for EditAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            EditAction::JumpToDefinition => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Edit Actions"),
        InputManagerBundle::with_map(
            InputMap::default().with(EditAction::JumpToDefinition, KeyCode::KeyE),
        ),
    ));
}

fn handle_edit_action(
    actions_query: Query<&ActionState<EditAction>>,
    hovered_query: Query<&TofuImportBlock, With<Hovered>>,
    mut events: EventWriter<TofuEvent>,
) {
    let Ok(actions) = actions_query.get_single() else {
        warn!("Could not find actions");
        return;
    };
    if actions.just_pressed(&EditAction::JumpToDefinition) {
        let found = hovered_query.iter().len();
        info!("Edit pressed on {} entities", found);
        for hovered in hovered_query.iter() {
            let path = hovered.file_path.to_owned();
            let line_col = hovered.line_col;
            events.send(TofuEvent::Open(path, line_col.0));
        }
    }
}
