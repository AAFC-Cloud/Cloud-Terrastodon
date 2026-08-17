use cloud_terrastodon_user_input::{Choice, PickError, PickerTui, TerminalCoordinator};
use tokio::task::JoinHandle;

use super::pickers::{FieldValuePicker, ValuePickerChoice};
use crate::object_explorer::SlotId;

#[derive(Debug)]
pub(super) enum ValuePickerOutcome {
    Selected {
        destination: SlotId,
        field: usize,
        choice: ValuePickerChoice,
    },
    Cancelled,
    Failed(String),
}

pub(super) struct ValuePickerTask {
    handle: JoinHandle<ValuePickerOutcome>,
}

impl ValuePickerTask {
    pub(super) fn spawn(picker: &FieldValuePicker) -> Result<Self, String> {
        let choices = picker
            .rows()
            .iter()
            .map(|row| Choice {
                key: row.label().to_owned(),
                value: row.choice().clone(),
            })
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err("no compatible object choices are available".to_owned());
        }
        let destination = picker.destination();
        let field = picker.field();
        let header = format!("Pick Object for {}", picker.field_name());
        let coordinator = TerminalCoordinator::try_current().ok_or_else(|| {
            "a nested PickerTui needs the object browser's TerminalCoordinator".to_owned()
        })?;
        let handle = tokio::spawn(async move {
            match PickerTui::new()
                .set_header(header)
                .set_auto_accept(false)
                .terminal_coordinator(coordinator)
                .pick_one(choices)
                .await
            {
                Ok(choice) => ValuePickerOutcome::Selected {
                    destination,
                    field,
                    choice,
                },
                Err(PickError::Cancelled) => ValuePickerOutcome::Cancelled,
                Err(error) => ValuePickerOutcome::Failed(error.to_string()),
            }
        });
        Ok(Self { handle })
    }

    pub(super) fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    pub(super) async fn finish(self) -> Result<ValuePickerOutcome, String> {
        self.handle
            .await
            .map_err(|error| format!("value PickerTui task failed: {error}"))
    }
}
