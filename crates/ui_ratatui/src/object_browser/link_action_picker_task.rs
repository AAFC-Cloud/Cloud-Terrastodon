use cloud_terrastodon_user_input::{Choice, PickError, PickerTui, TerminalCoordinator};
use tokio::task::JoinHandle;

use super::pickers::LinkActionPicker;
use crate::object_explorer::FieldCandidateAction;

#[derive(Debug)]
pub(super) enum LinkActionPickerOutcome {
    Selected(FieldCandidateAction),
    Cancelled,
    Failed(String),
}

pub(super) struct LinkActionPickerTask {
    handle: JoinHandle<LinkActionPickerOutcome>,
}

impl LinkActionPickerTask {
    pub(super) fn spawn(picker: &LinkActionPicker) -> Result<Self, String> {
        let choices = picker
            .consequences()
            .iter()
            .map(|consequence| Choice {
                key: format!("{:?}\n{}", consequence.action(), consequence.description()),
                value: consequence.action(),
            })
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err("no compatible transfer actions are available".to_owned());
        }
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header("Move or Clone")
                .set_auto_accept(false)
                .terminal_coordinator(coordinator)
                .pick_one(choices)
                .await
            {
                Ok(action) => LinkActionPickerOutcome::Selected(action),
                Err(PickError::Cancelled) => LinkActionPickerOutcome::Cancelled,
                Err(error) => LinkActionPickerOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self { handle })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<LinkActionPickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("link action PickerTui task failed: {error}"))
    }
}
